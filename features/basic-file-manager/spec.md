# Specification - Basic Terminal File Manager

## Goals
Develop a simple, elegant terminal-based file manager for macOS with dual-pane layout, local filesystem integration, visual directory navigation, and path configuration.

## Requirements

### 1. User Interface (UI)
- **ASCII Art Banner**: Displayed at the top header, reading `CookieCommander`.
- **Dual Panes**: Main section split vertically into Left and Right panes.
- **Active Pane Visual Feedback**: The border of the active pane should be colored differently (e.g. green or cyan) to clearly highlight which pane has input focus.
- **File Listings**: Both panes show a list of directory items (folders and files). Directories should be visually distinguishable (e.g. colored or prefixed). Shows file sizes and mod dates where possible.
- **Status bar**: Footer displaying key helper mappings (e.g. `Tab: Switch | Esc/q: Quit | g: Go to path | Enter: Open`).
- **Path Entry Overlay / Input**: A pop-up prompt at the bottom of the screen when entering a new directory path manually.

### 2. Navigation & Actions
- **Pane Navigation**: Use `Up` and `Down` arrow keys to scroll through entries in the active pane.
- **Pane Toggle**: Use `Tab` key to swap keyboard focus between Left and Right panes.
- **Enter Directory**: Press `Enter` on a directory to navigate into it.
- **Parent Directory**: Press `Enter` on the first entry `..` (if present) to move up to the parent directory.
- **Custom Path**: Press `g` to prompt path input. Enter path and press `Enter` to navigate that pane to the path. Press `Esc` to cancel.
- **Quit**: Press `q` or `Esc` (in normal mode) to exit the application and cleanly restore the terminal settings.

### 3. VFS Integration
- Use a `LocalVfs` implementing the existing `Vfs` trait to query real filesystem paths.
