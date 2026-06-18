# Spec: Copy and Delete Features

This document outlines the product requirements and user stories for the file selection, deletion, and copying features in CookieCommander.

## User Stories & Functional Requirements

### 1. File & Folder Selection
- **Toggle Selection**: The user can press `Space` on any file or folder in the active pane to select or unselect it.
- **Exclusion**: The special entry `..` (parent directory navigation) cannot be selected.
- **Visual Feedback**: Selected items are highlighted in the UI. 
  - We will display an indicator (e.g., `[x]` or a star `*` or a colored checkmark) next to selected items to make them stand out, even when the cursor moves away.
- **Selection State Persistence**: Selection is maintained as long as the user stays in the same directory. If they navigate to another directory, the selection for that pane is cleared.

### 2. Deletion Feature
- **Triggers**: Pressing the `Delete` key (or back-up key `d` if `Delete` is not captured on some terminals) triggers the deletion flow.
- **Deletion Targets**: 
  - If there are selected items in the active pane, those items are targeted for deletion.
  - If no items are selected, the item currently under the cursor (excluding `..`) is targeted.
- **Confirmation Prompt**:
  - The application enters a new mode: `InputMode::DeleteConfirm`.
  - A centered pop-up asks: `"Are you sure you want to delete the selected item(s)?"` (or similar clear message).
  - The user can confirm by pressing `y`/`Y` or cancel by pressing `n`/`N`/`Esc`.
- **Action**: On confirmation, all target files/folders are deleted recursively from the filesystem. The pane is then refreshed.

### 3. Copying Feature
- **Triggers**: Pressing `F5` (or `c` as a backup/alternative shortcut) triggers the copy action.
- **Copying Targets**:
  - If there are selected items in the active pane, those items are copied.
  - If no items are selected, the item currently under the cursor (excluding `..`) is copied.
- **Behavior**:
  - Files and folders are copied from the active pane's current directory to the inactive/other pane's current directory.
  - Directories must be copied recursively.
  - After copying, the target/other pane is refreshed to show the newly copied files.
