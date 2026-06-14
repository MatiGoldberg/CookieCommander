# Design Document - File Editing Integration

This document outlines the architectural changes required to support editing text files directly from CookieCommander.

## Architecture & Components

### 1. Editor Extensibility
To make the design extensible for future viewers/editors, we introduce a `SupportedEditor` abstraction. This abstraction specifies:
- The editor's internal identifier (e.g., `"vscode"`).
- Strategies to locate the editor's executable on the host system.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedEditor {
    VsCode,
}

impl SupportedEditor {
    pub fn id(&self) -> &'static str;
    pub fn find_path(&self) -> Option<String>;
}
```

### 2. Configuration Cache
We modify `AppConfig` inside `src/state/manager.rs` to include a map of cached editor executables:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub text_extensions: HashSet<String>,
    #[serde(default)]
    pub editors: std::collections::HashMap<String, String>,
}
```

### 3. Startup Detection
In `AppStateManager::init`:
1. Check if `vscode` path is defined in `config.json`.
2. If the path is not cached or if it doesn't point to an existing executable:
   - Run the search strategy for VS Code (`SupportedEditor::VsCode.find_path()`).
   - If found, update `config.editors` and rewrite `config.json`.

### 4. Spawning the Process
In `AppStateManager`, we implement `open_in_editor(&mut self)`:
- Retrieve `"vscode"` path from config.
- If not found or invalid, set `self.status_message = Some("Error: No editor is defined".to_string())`.
- If found, run:
  ```rust
  std::process::Command::new(editor_path)
      .arg(&viewer.file_path)
      .spawn();
  ```
- Set `self.status_message = Some(format!("Opened in VS Code: {}", file_name))`.

### 5. Keyboard Handling & UI
- In `src/main.rs`, capture `KeyCode::Char('e')` when `InputMode::FileViewer` is active, and trigger `state.open_in_editor()`.
- In `src/ui/render.rs`, add `e: Edit` to the help prompt inside the file viewer footer block.
