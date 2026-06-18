# Plan: Copy and Delete Features

This document outlines the milestones and implementation plan for selection, deletion, and copying.

## Milestones

### Milestone 1: Extend VFS Trait & LocalVfs Implementation
- Add `create_dir`, `remove_file`, `remove_dir_all`, and `copy_file` to `Vfs` trait.
- Implement them in `LocalVfs` using `tokio::fs`.
- Add integration/unit tests for the new `LocalVfs` methods.

### Milestone 2: Selection State & Toggling
- Add `selections` HashSet to `PaneState`.
- Implement `toggle_selection` in `PaneState`.
- Add unit tests for `toggle_selection` in `PaneState`.
- Ensure selections are cleared when navigation occurs (i.e. inside `handle_enter` and `commit_go_to_path`).

### Milestone 3: UI Rendering of Selections and Deletion Popup
- Update `render_pane` to draw selection indicator and highlight style for selected files.
- Design and implement the delete confirmation popup dialog.

### Milestone 4: Deletion Logic & Integration
- Implement deletion handling in `AppStateManager`.
- Create recursive delete function calling the new Vfs methods.
- Connect keybindings (`Delete` / `d`) to open the deletion confirmation dialog.
- Connect `y`/`n`/`Esc` keys to perform or cancel deletion.

### Milestone 5: Copy Logic & Integration
- Implement recursive copying logic in `AppStateManager` / helper module using Vfs primitives.
- Connect keybindings (`F5` / `c`) to trigger the copy.
- Refresh panes after the copy completes.
