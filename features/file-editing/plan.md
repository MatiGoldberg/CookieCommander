# Implementation Plan - File Editing Integration

This plan outlines the milestones for implementing the file editing integration.

## Milestones

### Milestone 1: Extensible Editor Detection
- Define the `SupportedEditor` enum with detection strategies for VS Code.
- Implement path searching on the local machine (PATH environment variable, Application paths).
- Checkpoint: Add unit tests verifying VS Code path detection when simulated or mocked.

### Milestone 2: AppConfig Extension & Caching
- Update `AppConfig` struct and serialization/deserialization.
- Add caching logic to `AppStateManager::init` to detect, write, and reload config.
- Checkpoint: Write a test showing that detecting an editor updates the configuration file.

### Milestone 3: Editor Launch Hook & Key Binding
- Implement `open_in_editor` on `AppStateManager`.
- Handle cases where VS Code is not installed (sets status message `"Error: No editor is defined"`).
- Connect `KeyCode::Char('e')` key event in `src/main.rs`.
- Checkpoint: Unit test for launching when path is missing (displays correct status message).

### Milestone 4: UI Updates & Polish
- Update `src/ui/render.rs` to show `e: Edit` in the file viewer footer block.
- Verify overall compilation, warnings, and formatting.
- Checkpoint: Run entire test suite and verify clean compilation using `cargo build --all-targets --all-features`.
