# Design - File/Folder Creation and Rename/Move

## Architecture Details

### 1. VFS Abstraction (`src/vfs`)
- Add `rename` to `Vfs` trait in `src/vfs/traits.rs`:
  ```rust
  async fn rename(&self, from: &str, to: &str) -> Result<()>;
  ```
- Implement `rename` in `LocalVfs` in `src/vfs/local.rs` using `tokio::fs::rename`.

### 2. State Management (`src/state/manager.rs`)
- Expand `InputMode` enum:
  ```rust
  pub enum InputMode {
      Normal,
      GoToPath,
      FileViewer,
      FileViewerSavePrompt,
      DeleteConfirm,
      CreateFolder,
      CreateFile,
      RenameOrMove,
  }
  ```
- Add `pub rename_target: Option<String>` to `AppStateManager` to store the original path when renaming/moving a single item.
- Implement helper methods on `AppStateManager`:
  - `start_create_folder(&mut self)`
  - `commit_create_folder(&mut self, vfs: &dyn Vfs) -> Result<()>`
  - `start_create_file(&mut self)`
  - `commit_create_file(&mut self, vfs: &dyn Vfs) -> Result<()>`
  - `start_rename_or_move(&mut self)`
  - `commit_rename_or_move(&mut self, vfs: &dyn Vfs) -> Result<()>`

### 3. Rendering (`src/ui/render.rs`)
- Add rendering helpers for the new modal input popups:
  - `render_create_folder_popup`
  - `render_create_file_popup`
  - `render_rename_or_move_popup`

### 4. Event Processing (`src/main.rs`)
- Route `KeyCode::F(7)` to `start_create_folder` in normal mode.
- Route `KeyCode::Char('n')` to `start_create_file` in normal mode.
- Route `KeyCode::F(6)` to `start_rename_or_move` in normal mode.
- In `InputMode::CreateFolder`, `InputMode::CreateFile`, and `InputMode::RenameOrMove` modes:
  - `Enter` triggers the commit methods.
  - `Esc` cancels the input.
  - Alphanumeric, punctuation, space, backspace keys update `input_buffer`.
