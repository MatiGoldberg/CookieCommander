# Design: Copy and Delete Features

This document outlines the architectural changes required to implement file/folder selection, deletion, and copying in CookieCommander.

## 1. Virtual File System (`Vfs`) API Changes

To support filesystem mutation, we will extend the `Vfs` trait in `src/vfs/traits.rs` with the following methods:

```rust
#[async_trait]
pub trait Vfs: Send + Sync {
    // Existing methods...
    
    /// Creates a directory.
    async fn create_dir(&self, path: &str) -> Result<()>;

    /// Deletes a file.
    async fn remove_file(&self, path: &str) -> Result<()>;

    /// Deletes a directory and all its contents recursively.
    async fn remove_dir_all(&self, path: &str) -> Result<()>;

    /// Copies a single file from one path to another.
    async fn copy_file(&self, from: &str, to: &str) -> Result<()>;
}
```

We will implement these in `LocalVfs` (`src/vfs/local.rs`) using `tokio::fs` functions:
- `tokio::fs::create_dir`
- `tokio::fs::remove_file`
- `tokio::fs::remove_dir_all`
- `tokio::fs::copy`

And we will mock them in `MockVfs` for unit testing.

## 2. Selection State in `PaneState`

We will add a selection tracker to `PaneState` in `src/state/pane.rs`:

```rust
pub struct PaneState {
    pub current_path: String,
    pub entries: Vec<FileMetadata>,
    pub selected_index: usize,
    /// Set of selected entry names (excluding "..")
    pub selections: std::collections::HashSet<String>,
}
```

We will add helper functions:
- `pub fn toggle_selection(&mut self)`: Toggles selection of the entry at `selected_index`.
- `pub fn clear_selections(&mut self)`: Clears selection set.

## 3. UI Improvements for Selected Items

In `src/ui/render.rs`:
- Check if `entry.name` is in `pane.selections`.
- If selected, prepend a checkmark symbol `✔ ` (or `* `) and render the entry name with a highlighted style (e.g., yellow/cyan foreground).
- Render `InputMode::DeleteConfirm` using a popup similar to the save prompt.

## 4. Key Binding Hooks

In `src/main.rs` and `AppStateManager`:
- `Space` key -> Toggle selection.
- `Delete` key / `d` -> Go to `InputMode::DeleteConfirm`.
- `F5` / `c` -> Run `copy_selected(&vfs)`.
- Confirm delete dialog `y`/`n`/`Esc` -> Action / Cancel.
