# Implementation Plan - Basic Terminal File Manager

## Milestone 1: Local VFS Implementation

### TDD Checkpoint
1. Add tests in `src/vfs/local.rs` (using `#[cfg(test)]`) that read a temporary workspace directory structure and confirm metadata/read_dir outputs.
2. Confirm the parent directory entry `..` is simulated or handled when reading folder items.

### Implementation
- Create `src/vfs/local.rs`.
- Wire it up in `src/vfs/mod.rs`.
- Ensure all tests pass.

---

## Milestone 2: State Management (TDD)

### TDD Checkpoint
1. Write functional/unit tests in `src/state/pane.rs` and `src/state/manager.rs` for:
   - Pane navigation bounds (moving up/down list of entries, clamp cursor).
   - Swapping active pane focus via Tab key events.
   - Folder navigation (entering a folder on Enter, handling parent `..` traverse).
   - Command mode (`g` key triggering prompt, typing path, Enter executing target navigation).
2. Use `MockVfs` to isolate state logic from standard fs I/O during tests.

### Implementation
- Create `src/state/pane.rs` and `src/state/manager.rs`.
- Integrate and export in `src/state/mod.rs`.
- Ensure unit and state flow tests pass.

---

## Milestone 3: TUI Rendering & Layout

### Implementation
- Create `src/ui/render.rs` with ratatui styling.
- Create helper function for drawing borders, colors, and the ASCII banner header.
- Add rendering code for directory items (prefixing folders, highlight row selection styling).
- Draw active pane border with green highlight color; inactive pane border with dark gray.
- Render text prompt in footer when `GoToPath` input mode is active.

---

## Milestone 4: Main Loop & Integration

### Implementation
- Setup terminal raw mode configuration and alternate screen in `src/main.rs`.
- Setup event loop polling `crossterm::event::read` asynchronously using Tokio.
- Feed event keys directly to `AppStateManager`.
- Refresh terminal layout drawing on state transitions.
- Gracefully restore terminal on program termination or crash.
