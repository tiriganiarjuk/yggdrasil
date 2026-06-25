# Kvasir - Workspace Inspector

**Kvasir** (Norse: the wisest being, whose knowledge was distilled into the mead of poetry) is a desktop workspace inspector for exploring codebases with syntax highlighting, format conversion, schema inspection, JSONL browsing, and tabular data viewing.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Kvasir (Tauri App)                            │
├──────────────────────────────────────────────────────────────────────┤
│  Frontend (Svelte 5)              │  Backend (Rust)                  │
│  ─────────────────────            │  ────────────────                │
│  • Directory tree view            │  • list_directory()              │
│  • Code viewer (highlight.js)     │  • read_file()                   │
│  • Format converter (7 formats)   │  • convert_to_all_formats()      │
│  • JSONL entry browser            │  • detect_data_format()          │
│  • Table viewer (CSV/TSV/Parquet) │  • read_jsonl_info/entry()       │
│  • Schema inspector               │  • export_entry_as()             │
│  • Markdown preview (rendered)    │  • read_table()                  │
│  • Wrap mode cycling              │  • export_table_csv()            │
│  • Dotfile toggle                 │  • open_in_editor()              │
│  • OS file association handler    │  • get_pending_file()            │
└──────────────────────────────────────────────────────────────────────┘
```

## Tech Stack

- **Tauri 2.x**: Rust backend + system webview
- **Svelte 5**: Frontend with runes ($state, $derived)
- **@yggdrasil/ui**: Shared component library (ContainerLayout, Button, ToggleGroup, etc.)
- **highlight.js**: Syntax highlighting
- **format_core** (nornir): Format parsing/serialization (JSON/YAML/TOML/TOON/XML/Markdown)
- **tiktoken-rs**: Real BPE token counting (cl100k)
- **csv/parquet/arrow**: Tabular data reading

## Key Files

```
kvasir/
├── src/
│   └── lib/
│       ├── KvasirView.svelte       # Main UI (git add -f required)
│       ├── JsonlViewer.svelte      # JSONL entry-by-entry browser
│       ├── TableViewer.svelte      # CSV/TSV/Parquet table viewer
│       ├── FormatControls.svelte   # Format toggle with token stats
│       ├── MarkdownPreview.svelte  # Rendered markdown display
│       ├── SchemaInspector.svelte  # JSON Schema analysis renderer
│       ├── schema-inspect.ts       # Pure schema analysis functions
│       └── kvasir-types.ts         # TypeScript type definitions
│   └── routes/
│       └── +page.svelte            # Thin wrapper: <KvasirView />
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                 # kvasir_lib::run()
│   │   └── lib.rs                  # Tauri wrappers + OS file-open handler
│   ├── Cargo.toml                  # deps: kvasir_core, common_core, tauri
│   └── capabilities/
│       └── default.json            # Tauri permissions
├── package.json
└── OUTLINE.md
```

## Commands

| Standalone | Yggdrasil (prefixed) | Description |
|-----------|---------------------|-------------|
| `list_directory` | `kvas_list_directory` | List directory contents |
| `read_file` | `kvas_read_file` | Read file with language detection |
| `open_in_editor` | `kvas_open_in_editor` | Open file in Zed |
| `convert_to_all_formats` | `kvas_convert_to_all_formats` | Convert between JSON/YAML/TOML/TOON/RON/XML/MD |
| `detect_data_format` | `kvas_detect_data_format` | Detect data format from file extension |
| `read_jsonl_info` | `kvas_read_jsonl_info` | Get JSONL file entry count and size |
| `read_jsonl_entry` | `kvas_read_jsonl_entry` | Read single JSONL entry by index |
| `export_entry_as` | `kvas_export_entry_as` | Export entry to temp file in chosen format |
| `read_table` | `kvas_read_table` | Read CSV/TSV/Parquet as table data |
| `export_table_csv` | `kvas_export_table_csv` | Export sorted/filtered table as CSV |
| `get_pending_file` | — | Get file path from OS file-open event |

## Format Conversion

Supports 8 formats via `format_core` (nornir) + `ron`:

| Format | Parse | Serialize | Notes |
|--------|-------|-----------|-------|
| JSON | yes | yes | Pretty-printed |
| YAML | yes | yes | |
| TOML | yes | yes | Nulls stripped before serialization |
| TOON | yes | yes | |
| RON | yes | yes | Rusty Object Notation |
| XML | yes | yes | `@` prefix for attributes, `#text` for text content |
| Markdown | — | yes | One-way: rendered as readable document |

Token counts use real BPE tokenization (cl100k via tiktoken-rs), not byte-length estimates.

## View Modes

| Mode | Trigger | Description |
|------|---------|-------------|
| Code (C) | Default for most files | Syntax-highlighted source with line numbers |
| Preview (P) | `.md` files | Rendered markdown with frontmatter support |
| Data (D) | Data files (json/yaml/toml/etc) | Format conversion with token comparison |
| Inspect (I) | `.schema.json` files | JSON Schema structure analysis |
| JSONL (J) | `.jsonl` files | Entry-by-entry browser with scrubber |
| Table (T) | `.csv`/`.tsv`/`.parquet` files | Sortable, filterable table view |

## Related Apps

- **Hlidskjalf**: Agent monitor
- **Svalinn**: Code quality viewer
- **Ratatoskr**: Graph viewer
- **@yggdrasil/ui**: Shared components
