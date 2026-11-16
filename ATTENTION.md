# ATTENTION — Svalinn's saga fix is committed and not deployed

`fee338d` repoints `run_saga` at `~/ai/tools/bin/saga` and reads sidecars through `saga_core`'s
`SanityReport` and `qa_path` (`core/svalinn_core`). The deployed Svalinn app still runs the
previous build until `deploy_apps.sh` runs — it rebuilds all five Tauri apps, so run it with the
machine cleared.

Two things in `core/svalinn_core/src/lib.rs`, found alongside:

- `run_saga` returns a `SagaResult` (success, file count, issue count, output) that the Svelte
  side discards — the button only waits for saga to finish, then refreshes — and
  `parse_saga_output` expects the Python saga's `Analyzed…` line, which the Rust saga never
  prints. Dead weight in a live function.
- `sidecar_status` recognises only `.py`, so Rust and Svelte files with sidecars show as having
  none in the tree view.

Delete this file when every item above is done.
