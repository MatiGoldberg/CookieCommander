# Design: Prettify JSON Feature

## 1. VFS Extension
We need to add a `write_file` operation to our Virtual File System (`Vfs`) trait to persist the formatted JSON when the user chooses to save.

- **File**: [traits.rs](file:///Users/matigoldberg/Code/CookieCommander/src/vfs/traits.rs)
  ```rust
  #[async_trait]
  pub trait Vfs: Send + Sync {
      // ...
      async fn write_file(&self, path: &str, content: &str) -> Result<()>;
  }
  ```
- **File**: [local.rs](file:///Users/matigoldberg/Code/CookieCommander/src/vfs/local.rs)
  ```rust
  #[async_trait]
  impl Vfs for LocalVfs {
      // ...
      async fn write_file(&self, path: &str, content: &str) -> Result<()> {
          tokio::fs::write(std::path::Path::new(path), content).await?;
          Ok(())
      }
  }
  ```

---

## 2. Prettifier Framework
To ensure extensibility, we define a core `Prettifier` trait in a new module `src/prettify.rs`. In the future, we can add XML, HTML, or other formatters here.

- **File**: `src/prettify.rs` [NEW]
  ```rust
  use anyhow::Result;

  pub trait Prettifier: Send + Sync {
      /// Checks if the content is in a state that can be formatted by this prettifier.
      fn can_prettify(&self, content: &str) -> bool;
      
      /// Reformats the content.
      fn prettify(&self, content: &str) -> Result<String>;
  }

  pub struct JsonPrettifier;

  impl Prettifier for JsonPrettifier {
      fn can_prettify(&self, content: &str) -> bool {
          // JSON must be minified (all on one line) to trigger the option
          let trimmed = content.trim_end();
          trimmed.lines().count() <= 1 && !trimmed.is_empty()
      }

      fn prettify(&self, content: &str) -> Result<String> {
          let val: serde_json::Value = serde_json::from_str(content)?;
          let pretty = serde_json::to_string_pretty(&val)?;
          Ok(pretty)
      }
  }

  pub fn get_prettifier(extension: &str) -> Option<Box<dyn Prettifier>> {
      match extension.to_lowercase().as_str() {
          "json" => Some(Box::new(JsonPrettifier)),
          _ => None,
      }
  }
  ```

---

## 3. Configuration & State Changes
We add configuration settings for size limits and update the application state machine to manage dirty/modified files and the save confirmation prompt.

- **File**: [manager.rs](file:///Users/matigoldberg/Code/CookieCommander/src/state/manager.rs)
  - **`AppConfig`**:
    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AppConfig {
        pub text_extensions: HashSet<String>,
        #[serde(default = "default_max_prettify_size_kb")]
        pub max_prettify_size_kb: usize,
    }
    
    fn default_max_prettify_size_kb() -> usize { 512 }
    ```
  - **`InputMode`**:
    ```rust
    pub enum InputMode {
        Normal,
        GoToPath,
        FileViewer,
        FileViewerSavePrompt,
    }
    ```
  - **`FileViewerState`**:
    ```rust
    pub struct FileViewerState {
        pub file_path: String,
        pub file_name: String,
        pub lines: Vec<String>,
        pub scroll_offset: usize,
        pub is_dirty: bool,
    }
    ```
  - **`AppStateManager` methods**:
    - `pub fn prettify_current_file(&mut self) -> Result<()>`: Joins viewer lines, verifies size against `config.max_prettify_size_kb`, retrieves the prettifier, formats the text, updates viewer lines, and marks `is_dirty = true`.
    - `pub async fn save_viewer_content(&mut self, vfs: &dyn Vfs) -> Result<()>`: Writes `lines.join("\n")` back to disk, clears `is_dirty`, and updates status.

---

## 4. UI Rendering
We draw the save confirmation dialog popup when the user is in `FileViewerSavePrompt` mode, and we add key hints to the footer of the file viewer.

- **File**: [render.rs](file:///Users/matigoldberg/Code/CookieCommander/src/ui/render.rs)
  - Render a confirmation popup similar to `render_go_to_popup` when mode is `FileViewerSavePrompt`.
  - Update `render_file_viewer` help footer to show `"p: Prettify"` when viewing a candidate JSON file.

---

## 5. Main Loop Integration
Intercept keys inside `main.rs` to handle transitions:
- If in `FileViewer` and user presses `p`, trigger prettification.
- If in `FileViewer` and user presses `Esc` or `q`, if the viewer is dirty, set mode to `FileViewerSavePrompt`. Else, close the viewer.
- If in `FileViewerSavePrompt`:
  - `y` / `Y` -> save and close.
  - `n` / `N` -> discard and close.
  - `Esc` -> return to `FileViewer` mode.
