# Specification - File Editing Integration

## Goal
Integrate an edit feature into the text file viewer of CookieCommander, allowing users to press `e` to quickly open the active file in Visual Studio Code.

## Requirements

### 1. Startup Editor Detection & Configuration Caching
- On startup, CookieCommander will scan for Visual Studio Code (VS Code) executable path.
- Check standard locations on macOS:
  - Inside the user's environment `PATH` (looking for executable `code`).
  - `/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code`.
  - `~/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code` (inside `$HOME`).
- The config file schema will be extended to support caching editor paths under `"editors"` object:
  ```json
  {
    "text_extensions": [...],
    "editors": {
      "vscode": "/path/to/code"
    }
  }
  ```
- If the path is found and not already cached (or cached path is no longer valid/exists), the detected path is written back to `config.json`.
- The architecture must be extensible so that support for other editors/viewers can be added in the future.

### 2. Launch Editor on Keypress
- When the file viewer is open (`InputMode::FileViewer`), pressing `e` will attempt to open the file in VS Code.
- Launch the VS Code process in the background asynchronously/non-blocking so the terminal UI remains responsive.
- If VS Code is not installed or not found, the status message is set to `"Error: No editor is defined"`.
- If successful, a status message indicating the editor opened the file will be shown.

### 3. UI Updates
- The file viewer help bar will be updated to display `e: Edit` alongside the existing controls.
