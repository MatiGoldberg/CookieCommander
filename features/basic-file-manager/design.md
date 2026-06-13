# Design Document - Basic Terminal File Manager

## Code Architecture

We decompose the implementation into clean, testable layers:
```
+-------------------------------------------------------+
|                       main.rs                         |
|   (crossterm initialization, terminal event loop)      |
+--------------------------+----------------------------+
                           |
                           v
+--------------------------+----------------------------+
|                    ui/render.rs                       |
|           (Ratatui widgets, layout, ASCII banner)      |
+--------------------------+----------------------------+
                           |
                           v
+--------------------------+----------------------------+
|                  state/manager.rs                     |
|           (AppStateManager, focus, input mode)        |
+--------------------------+----------------------------+
                           |
                           v
+--------------------------+----------------------------+
|                    state/pane.rs                      |
|        (PaneState, scroll offset, path history)        |
+--------------------------+----------------------------+
                           |
                           v
+--------------------------+----------------------------+
|                      vfs/local.rs                     |
|             (Vfs trait impl using tokio::fs)          |
+-------------------------------------------------------+
```

## Module Descriptions

### 1. VFS Layer (`src/vfs/local.rs`)
Implements the asynchronous `Vfs` trait for the local file system.
- `metadata(path)`: Fetches file information. Resolves path absolute references.
- `read_dir(path)`: Fetches list of entries in a folder, constructs metadata structs. Always places parent reference `..` as first entry if a parent folder exists.

### 2. State Layer (`src/state/`)
Maintains logical state independently of TUI code, enabling full test coverage without initializing terminal graphics.
- **`PaneState`**:
  - `current_path`: absolute path.
  - `entries`: sorted list of directories first, then files, alphabetically.
  - `selected_index`: cursor selection index.
- **`AppStateManager`**:
  - `left_pane`, `right_pane`: dual pane state.
  - `active_left`: boolean mapping pane focus.
  - `input_mode`: Normal vs GoToPath input.
  - `input_buffer`: current input typed in input prompt.
  - `status_message`: short status/error messages to display in footer.

### 3. UI Layer (`src/ui/`)
- Uses `ratatui` to build visual widgets.
- Generates ASCII layout and styles highlighting active/inactive window border blocks.
- Employs layout constraints: ASCII Art Banner (6 height) -> Two pane content split (remaining vertical space) -> Status bar / Input block (1-3 height).

### 4. Application Loop (`src/main.rs`)
- Standard terminal raw mode toggles.
- Subscribes to crossterm key event handler in async tokio thread.
- Tick rate loop to repaint when state is mutated.
