use directory_read_io::{DirectoryEntry, EntryKind, ListOptions};
use serde::Serialize;
use std::path::Path;

/// Open a file in the system editor (Zed via macOS `open -a`).
pub fn open_in_editor(path: &str, _line: usize) -> Result<(), String> {
    std::process::Command::new("open")
        .args(["-a", "Zed", path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// File Tree
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub extension: Option<String>,
    pub size_bytes: u64,
}

/// A directory's own entries, directories first, through nornir's one directory
/// walker. An entry that cannot be read fails the listing and is named in the
/// error. The directory may be reached through a symlink; the walk follows none.
pub fn list_directory(directory: &str, show_hidden: bool) -> Result<Vec<FileTreeEntry>, String> {
    let shown_under = Path::new(directory);
    let root = shown_under.canonicalize().map_err(|e| format!("Cannot list '{}': {}", directory, e))?;
    let options = ListOptions { max_depth: Some(1), include_hidden: show_hidden, ..ListOptions::default() };
    let listing = directory_read_io::list_directory(&root.to_string_lossy(), options).map_err(|e| e.to_string())?;

    let mut entries: Vec<FileTreeEntry> =
        listing.entries.iter().map(|listed| tree_entry(shown_under, listed)).collect();

    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

/// One listed entry as the tree shows it, under the directory as the user reached
/// it. A symlink is shown as what it points to, so it opens or expands like its
/// target; one that points nowhere is shown as an empty file.
fn tree_entry(shown_under: &Path, listed: &DirectoryEntry) -> FileTreeEntry {
    let path = shown_under.join(&listed.path);
    let (is_dir, size_bytes) = match listed.kind {
        EntryKind::Directory => (true, 0),
        EntryKind::File => (false, listed.size.unwrap_or(0)),
        EntryKind::Symlink => match std::fs::metadata(&path) {
            Ok(target) if target.is_dir() => (true, 0),
            Ok(target) => (false, target.len()),
            Err(_) => (false, 0),
        },
    };
    let extension = if is_dir { None } else { path.extension().map(|e| e.to_string_lossy().to_string()) };
    FileTreeEntry {
        name: listed.path.clone(),
        path: path.to_string_lossy().to_string(),
        is_dir,
        extension,
        size_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::path::PathBuf;

    fn temp_tree(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("common_core_{test_name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub/inner")).unwrap();
        std::fs::write(dir.join("sub/inner/deep.txt"), "deep").unwrap();
        std::fs::write(dir.join("Beta.md"), "abc").unwrap();
        std::fs::write(dir.join("alpha.toml"), "").unwrap();
        std::fs::write(dir.join(".hidden"), "x").unwrap();
        dir
    }

    fn names(entries: &[FileTreeEntry]) -> Vec<&str> {
        entries.iter().map(|entry| entry.name.as_str()).collect()
    }

    #[test]
    fn lists_a_directory_s_own_entries_directories_first() {
        let dir = temp_tree("own");
        let entries = list_directory(dir.to_str().unwrap(), false).unwrap();
        assert_eq!(names(&entries), ["sub", "alpha.toml", "Beta.md"]);
        assert!(entries[0].is_dir);
        assert_eq!(entries[2].size_bytes, 3);
        assert_eq!(entries[2].extension.as_deref(), Some("md"));
        assert_eq!(entries[2].path, dir.join("Beta.md").to_string_lossy());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hidden_entries_are_listed_only_when_asked() {
        let dir = temp_tree("hidden");
        assert!(names(&list_directory(dir.to_str().unwrap(), true).unwrap()).contains(&".hidden"));
        assert!(!names(&list_directory(dir.to_str().unwrap(), false).unwrap()).contains(&".hidden"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_symlink_is_shown_as_what_it_points_to_and_can_be_listed() {
        let dir = temp_tree("symlink");
        symlink(dir.join("sub"), dir.join("link_to_sub")).unwrap();
        symlink(dir.join("Beta.md"), dir.join("link_to_file")).unwrap();
        let entries = list_directory(dir.to_str().unwrap(), false).unwrap();
        let link_to_sub = entries.iter().find(|entry| entry.name == "link_to_sub").unwrap();
        let link_to_file = entries.iter().find(|entry| entry.name == "link_to_file").unwrap();
        assert!(link_to_sub.is_dir);
        assert!(!link_to_file.is_dir);
        assert_eq!(link_to_file.size_bytes, 3);

        let through_link = list_directory(&link_to_sub.path, false).unwrap();
        assert_eq!(names(&through_link), ["inner"]);
        assert_eq!(through_link[0].path, dir.join("link_to_sub/inner").to_string_lossy());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_directory_and_an_unreadable_one_are_errors_naming_them() {
        let dir = temp_tree("errors");
        let missing = dir.join("absent");
        assert!(list_directory(missing.to_str().unwrap(), false).unwrap_err().contains("absent"));

        let locked = dir.join("sub");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
        let refused = list_directory(locked.to_str().unwrap(), false);
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(refused.unwrap_err().contains("sub"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
