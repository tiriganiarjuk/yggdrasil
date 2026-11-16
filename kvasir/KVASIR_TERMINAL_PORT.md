# Kvasir: Terminal Port

## Goal

A terminal version of Kvasir that renders a file properly inside a terminal
pane — specifically a `herdr` pane beside a Claude Code session — instead of
the plain text `bat` gives. Not a priority; a tool worth building later.

The trigger: viewing a markdown roadmap next to a session, with its headings,
tables and emoji rendered, and its in-document links (`[4](#s4)`) followable so
a blocker is one keypress away.

## Provenance

The findings below come from a read-only evaluation of the code. Each claim
carries a file and line; verify against the source before building on it.

---

## What Kvasir is today

Desktop app (Tauri + Svelte 5) for inspecting a workspace.

| Mode | What it does | Where |
|---|---|---|
| Code | highlighted source (highlight.js) | `src/lib/KvasirView.svelte:7`, `:399-412` |
| Preview | `.md` via `marked`, YAML frontmatter turned into a fenced block | `KvasirView.svelte:9`, `:414-428` |
| Data | one file shown as JSON/YAML/TOML/TOON/RON/XML/Markdown, each with a cl100k token count | `core/kvasir_core/src/lib.rs:109-173`, `:340-345` |
| Inspect | schema analysis of `.schema.json` | `schema-inspect.ts` (837 lines, `analyzeSchema` at `:722`) |
| JSONL | one entry at a time, arrow keys | `JsonlViewer.svelte:160-168` |
| Table | CSV/TSV/Parquet, 100k-row cap, sort and filter, export to `<stem>_resorted.csv` | `lib.rs:63`, `:293-320`; `TableViewer.svelte:27-50` |

Mode list at `KvasirView.svelte:95-105`; default mode at `:265-267` (schema →
Inspect, `.md` → Preview, else Code).

Three layers:

> **`kvasir_core`** — pure Rust, no Tauri dependency (`core/kvasir_core/Cargo.toml`). File reading, language and format detection, conversion, token counts, JSONL and table reading.

> **Tauri shell** — 1:1 pass-throughs (`src-tauri/src/lib.rs:10-63`) plus macOS file-open handling (`:88-101`).

> **Svelte front end** — all presentation: markdown, highlighting, schema inspection, table sort/filter, JSONL navigation, folder picker (`KvasirView.svelte:134`), and the `~/ai` browsing boundary (`ui/components/ContainerLayout.svelte:155`).

**The key fact for a port: `kvasir_core` renders nothing.** Markdown,
highlighting and schema analysis must be rebuilt for the terminal.

## Reuse

**As-is:** every public function in `kvasir_core/src/lib.rs` — `list_directory`,
`read_file`, `convert_to_all_formats`, `detect_data_format`, the JSONL readers,
`read_table`, `export_table_csv`. Plain structs and strings in and out.
Caveats: `open_in_editor` is hard-wired to Zed on macOS
(`core/common_core/src/lib.rs:4-7`); `export_entry_as` writes to the temp
directory (`lib.rs:269-275`).

**Rebuilt in Rust:** markdown rendering and frontmatter, highlighting, schema
inspector, mode selection, table sort/filter, JSONL navigation, the `~/ai`
boundary, OS file-open handling.

**Placement rule:** display logic stays out of `*_core` crates (yggdrasil
CLAUDE.md, "Putting Display Logic in Core Crates"). The terminal renderer
belongs in the new crate.

**Offline buildability (inferred, untested):** `~/.cargo/registry` already
holds `ratatui` 0.29/0.30, `crossterm` 0.28/0.29, `pulldown-cmark` 0.10/0.12,
`syntect` 5.3. nornir's `cc_launch` is an existing ratatui app to copy
patterns from (`nornir/Cargo.toml:174-175` pins `ratatui = "0.30"`,
`crossterm = "0.29"`; UI in `nornir/cli/cc_launch/src/tui.rs`).

---

## Approaches

> **A. `kvasir-tui` — full ratatui app.** New workspace member on `kvasir_core`: file tree, mode bar (C/P/D/I/J/T), pulldown-cmark → styled ratatui text, syntect highlighting, table and JSONL navigation. Largest build. Loses: real heading sizes and fonts, inline images (unless herdr passes kitty/sixel through — unknown), raw HTML in markdown, mouse-first sorting, native folder picker, OS file associations, Yggdrasil tab hosting. Unknowns: port or drop the schema inspector; syntect grammar coverage for Svelte/TOON (inferred: probably missing); wide tables in a narrow pane.

> **B. One-file rendered viewer + herdr wrapper.** `kvasir-tui <path>`: no tree, opens one file in its default mode, scrolls, quits on `q`. A shell function splits a herdr pane and runs it there. Reuses `read_file`, `detect_data_format`, `convert_to_all_formats`. Small, and it is the first slice of A.

> **C. Existing Svelte front end in a terminal browser.** An HTTP shim exposing `kvasir_core`, plus an `invoke` replacement. Keeps almost everything, including the schema inspector. Loses the Tauri-only pieces (dialog plugin, file-open events, `homeDir`). Large unknowns: every `invoke` call site needs a transport swap, and how a terminal browser performs inside herdr is unverified.

**Lean: B, built as the start of A.** It solves the immediate need in little
code, respects the core/shell split, and grows into the other modes without
rework. Defer the schema inspector.

### What B must include from the start

> **In-document link navigation.** Following `#anchor` links is the reason for the port, and ratatui provides no link handling — the viewer has to track link targets and jump to them itself.

> **The herdr wrapper can capture the new pane.** `herdr pane split --current --direction right --no-focus --cwd <dir>` prints JSON containing `result.pane.pane_id`; `herdr pane run <pane_id> <command>` then starts the viewer in it.

---

## Open questions — for the owner

1. **Scope** — a quick one-file viewer, or full parity with the desktop app (tree, Data/JSONL/Table/Inspect)?
2. **Schema inspector** — needed in the terminal? It is an 837-line TypeScript port.
3. **Browsing boundary** — keep the `~/ai` / `~/.claude` restriction?
4. **Location** — a member of the yggdrasil workspace, or next to `cc_launch` in nornir `cli/`?
5. **Interaction** — interactive ratatui viewer, or one-shot ANSI output through `less -R`? (Link-following needs the interactive one.)
6. **herdr hookup** — a keybinding or `herdr integration` hook, or a shell function? Does herdr pass kitty/sixel graphics through?
7. **Open in Editor** — hard-wired to Zed today; in a terminal, open `$EDITOR` in another pane instead?
