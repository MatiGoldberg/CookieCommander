# Implementation Plan - Text File Viewer

## Milestone 1: VFS Extension & Configuration Loading

### TDD Checkpoint
1. Add tests in `src/vfs/local.rs` verifying that `read_file` correctly reads text files, and handles lossy UTF-8 conversion for non-UTF8 characters.
2. Verify that `AppConfig` loads correctly from a JSON file, and writes a default `config.json` file if it doesn't exist.

### Implementation
- Modify `src/vfs/traits.rs` to add `read_file` to the `Vfs` trait.
- Implement `read_file` in `src/vfs/local.rs`.
- Add `serde` and `serde_json` to `Cargo.toml` dependencies.
- Define `AppConfig` structure in a new module or directly inside `src/state/manager.rs` (or `src/config.rs`).
- Update `AppStateManager::init` to load or generate the configuration file.

---

## Milestone 2: State Management (TDD)

### TDD Checkpoint
1. Add tests in `src/state/manager.rs` checking:
   - File extensions check (case-insensitive) against configured extensions.
   - Transitioning to `InputMode::FileViewer` when pressing `Enter` on a supported file.
   - Correctly populating `FileViewerState` with filename, path, and split lines.
   - Viewer scroll bounds (scrolling up/down/page-up/page-down does not overflow/underflow bounds).
   - Pressing `Esc` or `q` transitions back to `InputMode::Normal`.

### Implementation
- Add `FileViewerState` struct.
- Add `InputMode::FileViewer` enum variant.
- Modify `AppStateManager::handle_enter` to handle file viewing.
- Add scrolling methods (`scroll_viewer_up`, `scroll_viewer_down`) to `AppStateManager`.

---

## Milestone 3: UI Rendering

### Implementation
- Implement `render_file_viewer` inside `src/ui/render.rs`.
- Draw a bordered box with filename and scroll status.
- Render text contents using a `Paragraph` with line-by-line slicing to support large files without `u16` overflow or performance issues.

---

## Milestone 4: Integration & Manual Verification

### Implementation
- Update `src/main.rs` event loop to dispatch keyboard inputs (Up, Down, PgUp, PgDn, Esc, q) when in `InputMode::FileViewer`.
- Clean, build, and run tests. Verify that everything works.
