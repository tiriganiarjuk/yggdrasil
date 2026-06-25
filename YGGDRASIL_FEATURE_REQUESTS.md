# YGGDRASIL_FEATURE_REQUESTS

Feature ideas under discussion. Not yet prioritized or scheduled.

---

## FR-1: Cross-App Navigation via Clickable Paths

**Origin:** Datagram payloads often contain file paths (e.g., compaction summaries, exchange diffs). These should be clickable.

**Behavior inside Yggdrasil:**
- Data files (.json, .yaml, .toml, .toon, .schema.json) — switch to Kvasir tab, load the file
- Code files (.py, .rs, .svelte, .ts) — `open_in_editor`
- Everything else — system default (`open`)

**Behavior standalone:**
- `open_in_editor` for code
- `open -a "kvasir"` for data files if Kvasir.app is installed
- System `open` as fallback

**Mechanism:**
- Views accept an optional `navigate` callback prop (parallel to `commands`)
- Hlidskjalf calls `navigate?.("kvasir", { file: "/path/to/file" })`
- Yggdrasil receives it, switches tab, passes file path to KvasirView via prop
- KvasirView watches the prop and loads the file
- File-type routing is a pure function (extension in, action out) — lives in the view, not the wrapper
- Target views accept typed props for incoming navigation: `openFile?: string` for Kvasir

**Key constraint:** Yggdrasil remains a thin wrapper. It doesn't know how Kvasir loads a file — it just passes a typed payload through. No app logic in the wrapper.

**Synergy with FR-2:** Datagrams can include `{ file: "/path/to.jsonl", line: 47 }`. Click in Hlidskjalf → Kvasir opens the JSONL browser at entry 47. Compaction finishes, datagram arrives, click the link, read the record in YAML.

---

## FR-2: JSONL Viewer in Kvasir — IMPLEMENTED

**Status:** Complete. Implemented as `JsonlViewer.svelte` with `read_jsonl_info`, `read_jsonl_entry`, and `export_entry_as` Rust commands. Single-entry record browser with scrubber, keyboard navigation (arrow keys + Home/End), format conversion (JSON/YAML/TOML/TOON/RON/XML/MD), and export to temp file.

---

## FR-3: "Open as Format" in Data Viewer

**Origin:** Kvasir already converts between JSON/YAML/TOML/TOON and shows the result. But sometimes you want to take that conversion into your editor — view a JSON config, realize TOML is more readable, open the TOML version in Zed.

**Behavior:**
- Viewing any data file in Kvasir with a format selected (JSON, YAML, TOML, TOON)
- "Open in Editor" exports the file in the **currently selected format** to a temp file, opens in Zed
- Not a permanent conversion — a temp file for reading/editing
- Temp file name: `kvasir-{filename}.{format}` in `/tmp/`

**Implementation:**
- Rust: new command takes file content + target format, writes temp file, calls `open_in_editor`
- Svelte: the existing "Open in Editor" button becomes format-aware when in data view — opens the converted version, not the original
- Reuses `convert_to_all_formats` (already exists) + `open_in_editor` (already exists)
- Shares the same temp-file-and-open mechanism as FR-2's JSONL export

---

## FR-4: Shared Format Conversion Crate — IMPLEMENTED

**Status:** Complete. `format_core` crate in nornir (`core/format_core/`) provides parsing and serialization for JSON, YAML, TOML, TOON, RON, XML, and Markdown. Uses `serde_json::Value` as universal intermediate representation. `error_core` provides `FormatError` with per-format parse variants. `kvasir_core` is a thin consumer via cross-repo workspace dependency.

---

## FR-5: Lightweight Table Viewer — IMPLEMENTED

**Status:** Complete. `TableViewer.svelte` with `read_table` and `export_table_csv` Rust commands. Supports CSV, TSV, and Parquet. Click-to-sort columns (asc/desc/unsorted cycle), text filter across all columns, row count, export sorted/filtered data as CSV. SQLite support not yet implemented.

---

## FR-6: Theme Switching — IMPLEMENTED

**Status:** Complete. 4 themes in `ui/css/tokens.css`: dark (default), light, warm-dark, cool-dark. `ThemeSwitcher` component in shared UI, persists via `localStorage`. `body[data-theme]` attribute + CSS selectors. All 5 apps inherit themes via shared `ui/` package. Polarity-aware contrast ratios (7-10:1 dark, 14-16:1 light). `SettingsBar` provides quick access in all apps via `ContainerLayout`.

---

## FR-7: Markdown Reading Experience

**Origin:** No existing markdown viewer is satisfying. Kvasir already renders markdown (MarkdownPreview.svelte) — the goal is to make it a genuinely great reading experience, especially for long and dense documentation.

**Reading controls (always visible toolbar):**
- **Theme quick-switch**: 3-5 favorite color theme buttons (not a dropdown — one click). Ties into FR-6 themes but could also have markdown-specific overrides (reading bg slightly different from app bg).
- **Font size slider**: continuous or stepped (12-24px range). Persists across sessions.
- **Font combination selector**: dropdown or slider for 3-5 curated pairings (e.g., heading font + body font). Examples:
  - System sans / monospace (clean, default)
  - Serif body / sans headings (book-like, easy on eyes for long reads)
  - Monospace everything (technical docs)
- **Heading separators**: selectable styles for visual breaks between sections — horizontal rules, colored bars, extra whitespace. Helps navigation in long dense docs where headings blur together.

**Auto-generated TOC sidebar:**
- Parse headings (h1-h6) from rendered markdown, display as navigable tree
- Click to jump to section
- Highlight current section on scroll (intersection observer)
- Indentation reflects heading depth
- Collapsible for deeply nested docs
- Toggle on/off — some docs are short enough not to need it

**Reading aids for dense docs:**
- Adjustable paragraph spacing (compact vs airy)
- Optional line-height control
- Focus mode: dim everything except the current section

**What it is NOT:**
- Not an editor — no editing, no split pane, no live preview of edits
- Not a full theming engine — curated presets, not infinite customization

**Implementation:**
- MarkdownPreview.svelte already exists — extend it with a toolbar
- Reading preferences stored in `localStorage`, applied as CSS custom properties
- Font combinations as predefined CSS class sets
- Heading separators as CSS `::after` pseudo-elements on heading tags
- The controls are markdown-specific, not app-wide (though theme buttons sync with FR-6)
