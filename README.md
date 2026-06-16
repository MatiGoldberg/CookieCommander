# 🍪 CookieCommander

CookieCommander is a high-performance, terminal-based **orthodox (dual-pane) file manager** built specifically for macOS. It is written in Rust and utilizes `ratatui` for its terminal user interface (TUI) rendering, `crossterm` for terminal manipulation, and `tokio` for async runtime support.

---

## ✨ Key Capabilities & Features

- **Orthodox (Dual-Pane) Interface**: Parallel visual panes for efficient directory comparison and navigation. Focus can be toggled using `Tab`.
- **Flexible Path Navigation**: Fast directory changes via standard arrow keys, `Backspace` (navigate up), and interactive path input via `g` ("Go To Path").
- **Built-in Scrollable File Viewer**: Instantly preview file contents by hitting `Enter` on any text/code files. Supports scrolling with Arrow keys and `PageUp`/`PageDown`.
- **JSON Prettification**: Press `p` inside the file viewer to format single-line or minified JSON files, complete with a save confirmation prompt on exit.
- **External Editor Integration**: Open files directly in your preferred editor (e.g., VS Code) by pressing `e` in the file viewer.
- **Dynamic Configuration**: A local `config.json` allows configuring text extensions, editor command paths, and file size limits for formatting.
- **Automated Versioning & Lifecycle Scripts**: Robust bash scripts for version increments, testing, cleaning, and publishing.

---

## 📂 Project Architecture

```
CookieCommander/
├── Cargo.toml               # Cargo package configuration and dependencies
├── config.json              # Runtime configuration (text extensions, editor pathways)
├── scripts/                 # Lifecycle automation & build scripts
│   ├── dev.sh               # Local development and run coordinator
│   ├── release.sh           # Optimised release-mode testing & runner
│   ├── publish.sh           # Build compiler & binary exporter
│   └── increment_version.sh # Semantic version bump utility
├── src/
│   ├── main.rs              # Application entrypoint & terminal setup
│   ├── prettify.rs          # Logic for file prettifying (e.g. JSON formatters)
│   ├── state/               # Application state managers
│   │   ├── mod.rs           # Module declarations
│   │   ├── manager.rs       # AppStateManager (handles input modes, files, path buffers)
│   │   └── pane.rs          # Pane state tracking active directories & selected files
│   ├── task_manager/        # Coordinator for background async IO tasks using Tokio
│   ├── ui/                  # UI components and layout rendering
│   │   ├── mod.rs
│   │   └── render.rs        # Layout templates for panels, headers, input bar, and viewer
│   └── vfs/                 # Virtual File System layer
│       └── local.rs / mod.rs# Local filesystem implementation and traits
└── features/                # Documentation and design specifications
```

---

## ⚙️ Configuration (`config.json`)

The application parses `config.json` at startup. Customize the following fields to fit your environment:

```json
{
  "text_extensions": [
    "rs", "toml", "json", "md", "txt", "js", "ts", "html", "css", "yaml", "yml"
  ],
  "editors": {
    "vscode": "/usr/local/bin/code"
  },
  "max_prettify_size_kb": 512
}
```

- **`text_extensions`**: Extensions that CookieCommander opens using the built-in file viewer.
- **`editors.vscode`**: Path to the VS Code executable (used when pressing `e` to edit a file).
- **`max_prettify_size_kb`**: Maximum file size limit in kilobytes allowed for JSON prettification.

---

## 🛠️ Development & Build Scripts

The `./scripts` directory contains utilities that manage the build-test-run loop:

### 1. `scripts/dev.sh`
Used for day-to-day development. Bumps the patch version, checks code, runs tests, builds, and launches CookieCommander.
```bash
./scripts/dev.sh [options]
```
- `--clean`: Runs `cargo clean` before building.
- `--no-test`: Skips running tests.
- `--save-logs`: Saves build, test, and runtime output to logs.
- `--force-quit`: Terminates any running `cookie_commander` processes before starting.
- `--log-level <level>`: Sets the log level (`error`, `warn`, `info`, `debug`, `trace`).

### 2. `scripts/release.sh`
Performs optimizations and builds the application in release mode for production performance.
```bash
./scripts/release.sh [options]
```
- `--clean`: Runs `cargo clean` before building.
- `--no-test`: Skips running tests.
- `--run`: Automatically runs the release binary after compilation.
- `--save-logs`: Saves release compilation logs.
- `--force-quit`: Kills other instances before launching.

### 3. `scripts/publish.sh`
Tags, increments the semantic version, builds, and compiles a standalone, version-stamped release binary into the `dist/` folder.
```bash
./scripts/publish.sh [options]
```
- By default, it increments the **minor** version.
- `--major` or `-m`: Increments the **major** version (setting minor and patch to `0`).

### 4. `scripts/increment_version.sh`
A internal utility script used by the build tools to safely parse, increment (`major` | `minor` | `patch`), and write back version numbers to `Cargo.toml`.

---

## 🚀 Running Manually

If you prefer to run raw Cargo commands directly:

### Installation & Cargo Check
```bash
cargo check
```

### Running in Debug Mode
```bash
cargo run
```

### Building Release Executables
```bash
cargo build --release
```
The compiled binary will be placed at `target/release/cookie_commander`.

---

## 🧪 Testing

CookieCommander uses Rust's native unit testing combined with `mockall` for testing VFS logic.

To run the full test suite:
```bash
cargo test
```

To test a specific module (e.g. `prettify` or `vfs`):
```bash
cargo test prettify
```
