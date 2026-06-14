# Design Document - Text File Viewer

## Architecture & Data Flow

The Text File Viewer integrates directly with the existing dual-pane terminal layout, using a new input mode `InputMode::FileViewer`.

```
+-------------------------------------------------------+
|                      main.rs                          |
|  (polls PgUp/PgDn/Up/Down/Esc/q/Enter key events)     |
+--------------------------+----------------------------+
                           |
                           v
+--------------------------+----------------------------+
|                  state/manager.rs                     |
|  - Switch mode to FileViewer                          |
|  - Load text_extensions configuration                  |
|  - Track scroll_offset and load file lines            |
+--------------------------+----------------------------+
                           |
                           v
+--------------------------+----------------------------+
|                    ui/render.rs                       |
|  - Renders fullscreen file viewer overlay             |
|  - Computes visible lines and pagination              |
|  - Displays scroll percentage and filename            |
+--------------------------+----------------------------+
                           |
                           v
+--------------------------+----------------------------+
|                      vfs/ traits & local              |
|  - Add `read_file` method to Vfs trait                |
|  - Implement `read_file` in LocalVfs                  |
+-------------------------------------------------------+
```

## Detailed Component Specifications

### 1. Configuration (`AppConfig`)
We will introduce `AppConfig` to hold configurations, including the extensible list of text extensions.
It will be stored in `config.json` in the current working directory.
If `config.json` is missing, the application will serialize a default configuration to `config.json` containing:
`["txt", "md", "rs", "cs", "py", "java", "c", "cpp", "h", "csproj", "json", "toml", "js", "ts", "html", "css", "yaml", "yml", "sh", "bat"]`.

### 2. VFS Extension (`src/vfs/traits.rs` & `src/vfs/local.rs`)
- **`Vfs` trait**:
  ```rust
  async fn read_file(&self, path: &str) -> Result<String>;
  ```
- **`LocalVfs` implementation**:
  Reads the file content via `tokio::fs::read`. Converts bytes to a string using lossy UTF-8 conversion (`String::from_utf8_lossy`) to prevent crashes on binary or malformed files.

### 3. State Additions (`src/state/manager.rs`)
- **`InputMode`**: Add `InputMode::FileViewer`.
- **`FileViewerState`**:
  ```rust
  #[derive(Debug, Clone)]
  pub struct FileViewerState {
      pub file_path: String,
      pub file_name: String,
      pub lines: Vec<String>,
      pub scroll_offset: usize,
  }
  ```
- **`AppStateManager`**:
  - `pub file_viewer: Option<FileViewerState>`
  - `pub config: AppConfig`
  - Modify `handle_enter` to check if the selected entry is a file. If it's a file:
    - Extract extension.
    - Check if extension is in `config.text_extensions` (case-insensitively).
    - If yes, read file contents using `vfs.read_file`, split into lines, instantiate `FileViewerState`, transition `mode` to `InputMode::FileViewer`.
    - If no, set a status message "Unsupported file type" (or ignore).

- **Navigation Methods**:
  - `scroll_viewer_up(lines: usize)`: Decreses `scroll_offset` by `lines`, clamped to 0.
  - `scroll_viewer_down(lines: usize, visible_height: usize)`: Increases `scroll_offset` by `lines`, clamped to `lines.len() - visible_height`.

### 4. UI Layer (`src/ui/render.rs`)
- In `render()`, check if `state.mode == InputMode::FileViewer`. If so, render the file viewer modal using `Clear` to overlay the dual-pane view.
- The viewer block will display the filename in the title, and scroll information (e.g., `Line X-Y of Z (P%)`) in the bottom border.
- The file content is rendered inside the block using a `Paragraph`.

### 5. Keyboard Event Loop (`src/main.rs`)
Handle key codes when `state.mode == InputMode::FileViewer`:
- `Up`: Scroll up 1 line.
- `Down`: Scroll down 1 line.
- `PageUp`: Scroll up by a page (viewer height - 2).
- `PageDown`: Scroll down by a page (viewer height - 2).
- `Esc` or `q`: Exit viewer (sets mode back to `InputMode::Normal` and clears `file_viewer`).
