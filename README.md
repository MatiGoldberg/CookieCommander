# 🍪 CookieCommander

CookieCommander is a high-performance, terminal-based **orthodox (dual-pane) file manager** built specifically for macOS. It is written in Rust and utilizes `ratatui` for its terminal user interface (TUI) rendering, `crossterm` for terminal manipulation, and `tokio` for async runtime support.

---

## 🚀 Getting Started

### 📋 Prerequisites

To build, run, and test CookieCommander, you need the standard Rust toolchain installed. If you do not have it, you can install it via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 🛠️ Installation & Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/CookieCommander.git
   cd CookieCommander
   ```

2. **Verify the installation:**
   Ensure your Rust compiler is up to date and can compile the codebase:
   ```bash
   cargo check
   ```

---

## 🏃 Running the Application

To run the application in development mode, run:

```bash
cargo run
```

This will compile and launch the terminal interface. Because it is in active development, the initial view starts with a startup message and will boot into the dual-pane UI once complete.

To build the release version for optimal performance:

```bash
cargo build --release
```
The compiled binary will be located at `target/release/cookie_commander`.

---

## 🧪 Testing

CookieCommander uses Rust's built-in testing framework along with `mockall` for mock-based unit tests.

### Running all tests

To run the test suite, run:

```bash
cargo test
```

### Running specific tests

To run tests within a specific module (e.g., the virtual file system `vfs`):

```bash
cargo test vfs
```

---

## 📂 Project Structure

Here is a breakdown of the primary directories and modules:

- **`src/main.rs`**: Application entrypoint.
- **[`src/vfs/`](file:///Users/matigoldberg/Code/CookieCommander/src/vfs)**: Virtual File System abstraction. Contains traits, file types, and metadata structures supporting mockable file operations.
- **[`src/state/`](file:///Users/matigoldberg/Code/CookieCommander/src/state)**: Application state management, tracking selected items, panel focus, and active operations.
- **[`src/task_manager/`](file:///Users/matigoldberg/Code/CookieCommander/src/task_manager)**: Background task coordinator for async IO operations (copying, moving, deleting) using Tokio tasks.
- **[`src/ui/`](file:///Users/matigoldberg/Code/CookieCommander/src/ui)**: The layout, styling, and widget rendering logic using `ratatui`.
