# Plan - File/Folder Creation and Rename/Move

## Milestones

### Milestone 1: Extend VFS with Rename Method
- Add `rename` to `Vfs` trait and `LocalVfs`.
- Implement unit tests for `rename` in `local.rs`.

### Milestone 2: Implement File & Folder Creation State Logic
- Update `InputMode` and `AppStateManager`.
- Implement state transitions and creation logic (`start_create_folder`, `commit_create_folder`, `start_create_file`, `commit_create_file`).
- Implement unit tests verifying creation logic and path validation.

### Milestone 3: Implement Rename / Move State Logic
- Implement state transitions and rename/move logic (`start_rename_or_move`, `commit_rename_or_move`).
- Implement unit tests verifying rename/move logic (single-item rename/move, multi-item move).

### Milestone 4: UI Popups and Key Routing
- Implement TUI popups in `src/ui/render.rs` for CreateFolder, CreateFile, and RenameOrMove.
- Route key events in `src/main.rs`.
- Conduct manual and automated verification.
