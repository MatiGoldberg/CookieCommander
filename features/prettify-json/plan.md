# Plan: Prettify JSON Feature

This plan outlines the milestones for implementing the Prettify JSON feature in CookieCommander using a Test-Driven Development (TDD) approach.

---

## Milestone 1: VFS Extension
- [ ] Add `write_file` method to `Vfs` trait in `src/vfs/traits.rs`.
- [ ] Implement `write_file` in `LocalVfs` in `src/vfs/local.rs`.
- [ ] Add unit test for `LocalVfs::write_file`.
- [ ] Verify existing code compiles and passes tests.

## Milestone 2: Prettifier Framework
- [ ] Create `src/prettify.rs` implementing `Prettifier` and `JsonPrettifier`.
- [ ] Write unit tests for `JsonPrettifier` to check candidate files (1-line check) and prettification of minified JSON.
- [ ] Expose `prettify` module in `src/main.rs`.

## Milestone 3: Configuration & State Updates
- [ ] Update `AppConfig` in `src/state/manager.rs` to include `max_prettify_size_kb` defaulting to `512`.
- [ ] Add `FileViewerSavePrompt` to `InputMode` and `is_dirty: bool` to `FileViewerState`.
- [ ] Implement `AppStateManager::prettify_current_file` (including size check, prettifier retrieval, and error state tracking).
- [ ] Implement `AppStateManager::save_viewer_content` utilizing `Vfs::write_file`.
- [ ] Write unit tests in `src/state/manager.rs` for config defaults, JSON prettify command, size constraint validation, and save function.

## Milestone 4: Main Loop & Key Handling
- [ ] Update key handler in `src/main.rs` for `InputMode::FileViewer` to process `p` (format).
- [ ] Update key handler in `src/main.rs` for `InputMode::FileViewer` to intercept `Esc`/`q` and open `FileViewerSavePrompt` if file is dirty.
- [ ] Add key handler in `src/main.rs` for `InputMode::FileViewerSavePrompt`:
  - `y` / `Y` -> Save and close.
  - `n` / `N` -> Discard changes and close.
  - `Esc` -> Go back to file viewing.

## Milestone 5: UI Rendering
- [ ] Update `src/ui/render.rs` to draw `FileViewerSavePrompt` as a centered confirmation popup.
- [ ] Update `render_file_viewer` help footer to display key hints.
- [ ] Run comprehensive cargo build and verify zero warnings/errors.
- [ ] Run full test suite to guarantee functionality.
