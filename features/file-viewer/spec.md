# Specification - Text File Viewer

## Goal
Provide a text file viewer that allows users to quickly inspect the contents of text-based files (markdown, source code, JSON, etc.) directly inside the file manager by pressing `Enter` on the file.

## Requirements

### 1. Configuration & Extensibility
- The list of file extensions allowed to be opened in the file viewer must be configurable and extensible.
- Configuration will be loaded from a JSON file (e.g. `config.json` in the current working directory).
- If the configuration file is missing, the application should generate a default config file with a robust list of common text file extensions:
  - Markdown (`md`)
  - Rust (`rs`)
  - C# (`cs`, `csproj`)
  - Python (`py`)
  - Java (`java`)
  - C/C++ (`c`, `cpp`, `h`, `hpp`)
  - JSON (`json`)
  - TOML (`toml`)
  - Text (`txt`)
  - JavaScript / TypeScript (`js`, `ts`, `jsx`, `tsx`)
  - Web (`html`, `css`)
  - Shell scripts (`sh`, `bat`)
  - Configs (`yaml`, `yml`, `ini`)
- Files without an extension or with an extension not matching the configured list will not trigger the viewer.

### 2. Enter on Text File
- When a user selects a file in the active pane and presses `Enter`:
  - If the file's extension matches the configured text extensions, open the file in the viewer.
  - If the extension does not match, display a status message indicating the file type is unsupported or not configurable.
- If the selected item is a directory, the existing behavior remains (i.e. navigate into the directory).

### 3. File Viewer UI & Interaction
- The File Viewer should be rendered as a full-screen overlay or a large centered modal.
- It must display:
  - The path or name of the file in the border title.
  - The actual file lines/content.
  - Clear keyboard helper cues (e.g., `Esc / q: Close | Up/Down: Scroll line | PgUp/PgDn: Scroll page`).
- Interactions:
  - `Up Arrow`: Scroll up 1 line.
  - `Down Arrow`: Scroll down 1 line.
  - `Page Up` (or code counterpart): Scroll up by one page.
  - `Page Down` (or code counterpart): Scroll down by one page.
  - `Esc` or `q`: Close the viewer and return to normal pane navigation.
- If the file is empty or cannot be read, display an appropriate error/empty message in the viewer.
