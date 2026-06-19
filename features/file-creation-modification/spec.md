# Specification - File/Folder Creation and Rename/Move

## Goal
Implement features in CookieCommander to create new empty files or folders, and rename or move existing files/folders.

## Requirements

### 1. Folder Creation (F7)
- Pressing `F7` in normal mode pops up a modal window/input box.
- The input box lets the user write the name of the new folder.
- Validation:
  - The name must not be empty.
  - The name must not contain illegal path characters.
  - The folder must not already exist in the current directory.
- On pressing `Enter`:
  - If valid, create the folder using `Vfs::create_dir`.
  - Refresh the current pane.
  - Clear the input buffer and return to normal mode.
  - Show a status message confirming success.
- On pressing `Esc`:
  - Cancel the creation and return to normal mode.

### 2. File Creation (n)
- Pressing `n` in normal mode pops up a modal window/input box.
- The input box lets the user write the name of the new file (including extension).
- Validation:
  - The name must not be empty.
  - The file must not already exist in the current directory.
- On pressing `Enter`:
  - If valid, create an empty file using `Vfs::write_file` with empty contents.
  - Refresh the current pane.
  - Clear the input buffer and return to normal mode.
  - Show a status message confirming success.
- On pressing `Esc`:
  - Cancel the creation and return to normal mode.

### 3. Rename or Move (F6)
- Pressing `F6` handles two scenarios based on selection:
  - **Single Item (No selection)**: Rename or move the item under the cursor.
    - If the entry is `..`, do nothing or show an error.
    - Pop up a dialog. Pre-populate the input box with the item's current full path.
    - The user can edit the path to either rename the file (change the name at the end) or move it (change the directory path).
    - If the user enters a relative path, resolve it relative to the current pane's directory.
  - **Multiple Items (Active selections)**: Move all selected items to a target directory.
    - Pop up a dialog. Pre-populate the input box with the other pane's current directory path.
    - The user can edit the target directory.
    - On pressing `Enter`, move each selected item to the target directory.
- Validation:
  - The target path must not be identical to the source path.
  - Target parent directories must exist.
- On pressing `Enter`:
  - Execute the rename/move operation using `Vfs::rename`.
  - Refresh both panes.
  - Clear the input buffer/selections and return to normal mode.
- On pressing `Esc`:
  - Cancel and return to normal mode.
