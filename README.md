# Utagoe

Utagoe is a local-first music listening system that combines direct playback, provider-backed cloud statistics sync, acquisition, and a music knowledge graph.

## Workspace

- `apps/utagoe-app`: Tauri + Solid desktop shell.
- `crates/libutagoe`: headless application core and orchestration layer.
- `crates/saisei`: local audio playback runtime.
- `crates/wakatta`: music knowledge base and relationship graph.
- `crates/utagoe-provider-protocol`: bincode-based provider RPC types.
- `crates/utagoe-provider-runtime`: WASM provider runtime boundary.
- `crates/utagoe-acquisition`: local file acquisition pipeline boundary.
