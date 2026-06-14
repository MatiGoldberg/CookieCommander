use crate::state::pane::PaneState;
use crate::vfs::Vfs;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    GoToPath,
    FileViewer,
}

#[derive(Debug, Clone)]
pub struct FileViewerState {
    pub file_path: String,
    pub file_name: String,
    pub lines: Vec<String>,
    pub scroll_offset: usize,
}

fn serialize_sorted_set<S>(set: &HashSet<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut vec: Vec<&String> = set.iter().collect();
    vec.sort();
    serializer.collect_seq(vec)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(serialize_with = "serialize_sorted_set")]
    pub text_extensions: HashSet<String>,
    #[serde(default)]
    pub editors: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let extensions = vec![
            "txt", "md", "rs", "cs", "py", "java", "c", "cpp", "h", "csproj", "json", "toml", "js",
            "ts", "html", "css", "yaml", "yml", "sh", "bat",
        ];
        Self {
            text_extensions: extensions.into_iter().map(String::from).collect(),
            editors: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum SupportedEditor {
    VsCode,
}

impl SupportedEditor {
    pub fn id(&self) -> &'static str {
        match self {
            Self::VsCode => "vscode",
        }
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::VsCode => "VS Code",
        }
    }

    pub fn find_path(&self) -> Option<String> {
        match self {
            Self::VsCode => {
                // 1. Search PATH
                if let Some(path_var) = std::env::var_os("PATH") {
                    for path in std::env::split_paths(&path_var) {
                        let p = path.join("code");
                        if p.exists() && p.is_file() {
                            return Some(p.to_string_lossy().to_string());
                        }
                    }
                }

                // 2. Check standard macOS Application locations
                let macos_paths =
                    ["/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code"];
                for p in &macos_paths {
                    let path = Path::new(p);
                    if path.exists() {
                        return Some(p.to_string());
                    }
                }

                // 3. Check home directory Application folder
                if let Ok(home) = std::env::var("HOME") {
                    let p = format!(
                        "{}/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
                        home
                    );
                    if Path::new(&p).exists() {
                        return Some(p);
                    }
                }

                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppStateManager {
    pub left_pane: PaneState,
    pub right_pane: PaneState,
    pub active_left: bool,
    pub mode: InputMode,
    pub input_buffer: String,
    pub status_message: Option<String>,
    pub file_viewer: Option<FileViewerState>,
    pub config: AppConfig,
}

impl AppStateManager {
    pub fn new(left_path: String, right_path: String) -> Self {
        Self {
            left_pane: PaneState::new(left_path),
            right_pane: PaneState::new(right_path),
            active_left: true,
            mode: InputMode::Normal,
            input_buffer: String::new(),
            status_message: None,
            file_viewer: None,
            config: AppConfig::default(),
        }
    }

    pub async fn init(&mut self, vfs: &dyn Vfs) -> Result<()> {
        // Load configuration
        match vfs.read_file("config.json").await {
            Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                Ok(cfg) => {
                    self.config = cfg;
                }
                Err(e) => {
                    self.status_message =
                        Some(format!("Config parse error: {}. Using defaults.", e));
                    self.config = AppConfig::default();
                }
            },
            Err(_) => {
                // Generate default config.json
                let default_cfg = AppConfig::default();
                if let Ok(json_str) = serde_json::to_string_pretty(&default_cfg) {
                    let _ = tokio::fs::write("config.json", json_str).await;
                }
                self.config = default_cfg;
            }
        }

        // Scan for editors and cache them if they are found and not already cached (or if the cached path is invalid)
        let mut config_updated = false;
        let vscode = SupportedEditor::VsCode;
        let vscode_cached = self
            .config
            .editors
            .get(vscode.id())
            .map(|p| Path::new(p).exists())
            .unwrap_or(false);

        if !vscode_cached {
            if let Some(path) = vscode.find_path() {
                self.config.editors.insert(vscode.id().to_string(), path);
                config_updated = true;
            }
        }

        if config_updated {
            if let Ok(json_str) = serde_json::to_string_pretty(&self.config) {
                let _ = tokio::fs::write("config.json", json_str).await;
            }
        }

        let left_res = self.left_pane.refresh(vfs).await;
        let right_res = self.right_pane.refresh(vfs).await;

        if let Err(e) = &left_res {
            self.status_message = Some(format!("Left pane error: {}", e));
        }
        if let Err(e) = &right_res {
            self.status_message = Some(format!("Right pane error: {}", e));
        }

        left_res?;
        right_res?;
        Ok(())
    }

    pub fn active_pane(&self) -> &PaneState {
        if self.active_left {
            &self.left_pane
        } else {
            &self.right_pane
        }
    }

    pub fn active_pane_mut(&mut self) -> &mut PaneState {
        if self.active_left {
            &mut self.left_pane
        } else {
            &mut self.right_pane
        }
    }

    pub fn switch_pane(&mut self) {
        self.active_left = !self.active_left;
    }

    pub async fn handle_enter(&mut self, vfs: &dyn Vfs) -> Result<()> {
        let (name, is_dir, file_type) = {
            let active = self.active_pane();
            if let Some(entry) = active.selected_entry() {
                (
                    entry.name.clone(),
                    entry.file_type == crate::vfs::FileType::Directory,
                    entry.file_type,
                )
            } else {
                return Ok(());
            }
        };

        if !is_dir {
            if file_type == crate::vfs::FileType::File {
                let path = Path::new(&name);
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if self.config.text_extensions.contains(&ext) {
                    let active = self.active_pane();
                    let full_path = Path::new(&active.current_path).join(&name);
                    let full_path_str = full_path.to_string_lossy().to_string();

                    match vfs.read_file(&full_path_str).await {
                        Ok(content) => {
                            let lines = content.lines().map(String::from).collect::<Vec<_>>();
                            self.file_viewer = Some(FileViewerState {
                                file_path: full_path_str,
                                file_name: name,
                                lines,
                                scroll_offset: 0,
                            });
                            self.mode = InputMode::FileViewer;
                            self.status_message = None;
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Failed to read file: {}", e));
                        }
                    }
                } else {
                    self.status_message = Some(format!("Unsupported file type: .{}", ext));
                }
            }
            return Ok(());
        }

        let active = self.active_pane_mut();
        if name == ".." {
            if let Some(parent) = Path::new(&active.current_path).parent() {
                if let Some(p_str) = parent.to_str() {
                    if !p_str.is_empty() {
                        active.current_path = p_str.to_string();
                        active.selected_index = 0;
                    }
                }
            }
        } else {
            let next_path = Path::new(&active.current_path).join(name);
            if let Some(p_str) = next_path.to_str() {
                active.current_path = p_str.to_string();
                active.selected_index = 0;
            }
        }

        if let Err(e) = active.refresh(vfs).await {
            self.status_message = Some(format!("Error: {}", e));
        } else {
            self.status_message = None;
        }

        Ok(())
    }

    pub fn scroll_viewer_up(&mut self, lines: usize) {
        if let Some(viewer) = &mut self.file_viewer {
            viewer.scroll_offset = viewer.scroll_offset.saturating_sub(lines);
        }
    }

    pub fn scroll_viewer_down(&mut self, lines: usize, visible_height: usize) {
        if let Some(viewer) = &mut self.file_viewer {
            let max_scroll = viewer.lines.len().saturating_sub(visible_height);
            viewer.scroll_offset = (viewer.scroll_offset + lines).min(max_scroll);
        }
    }

    pub fn close_file_viewer(&mut self) {
        self.mode = InputMode::Normal;
        self.file_viewer = None;
    }

    pub fn start_go_to_path(&mut self) {
        self.mode = InputMode::GoToPath;
        self.input_buffer = self.active_pane().current_path.clone();
        self.status_message = None;
    }

    pub async fn commit_go_to_path(&mut self, vfs: &dyn Vfs) -> Result<()> {
        let path_to_set = self.input_buffer.trim().to_string();
        self.mode = InputMode::Normal;

        let (res, final_path) = {
            let active = self.active_pane_mut();
            let old_path = active.current_path.clone();

            active.current_path = path_to_set;
            active.selected_index = 0;

            let res = active.refresh(vfs).await;
            if res.is_err() {
                active.current_path = old_path;
                let _ = active.refresh(vfs).await;
            }
            (res, active.current_path.clone())
        };

        match res {
            Ok(_) => {
                self.status_message = Some(format!("Navigated to {}", final_path));
            }
            Err(e) => {
                self.status_message = Some(format!("Navigation failed: {}", e));
            }
        }

        Ok(())
    }

    pub fn cancel_input(&mut self) {
        self.mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub async fn navigate_up_directory(&mut self, vfs: &dyn Vfs) -> Result<()> {
        if let Some(first_entry) = self.active_pane().entries.first() {
            if first_entry.name == ".." {
                self.active_pane_mut().selected_index = 0;
                self.handle_enter(vfs).await?;
            }
        }
        Ok(())
    }

    pub fn open_in_editor(&mut self) {
        let file_path = if let Some(viewer) = &self.file_viewer {
            viewer.file_path.clone()
        } else {
            return;
        };

        let vscode_path = self.config.editors.get("vscode").cloned();
        if let Some(path) = vscode_path {
            if Path::new(&path).exists() {
                match std::process::Command::new(path).arg(&file_path).spawn() {
                    Ok(_) => {
                        self.status_message = Some(format!("Opened in VS Code: {}", file_path));
                    }
                    Err(e) => {
                        self.status_message =
                            Some(format!("Error: Failed to launch VS Code: {}", e));
                    }
                }
            } else {
                self.status_message = Some("Error: No editor is defined".to_string());
            }
        } else {
            self.status_message = Some("Error: No editor is defined".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::{FileMetadata, FileType, MockVfs};

    struct TempDirGuard {
        old_cwd: std::path::PathBuf,
        temp_dir: std::path::PathBuf,
    }

    impl TempDirGuard {
        fn new() -> Self {
            let unique_id = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let temp_dir = std::env::temp_dir().join(format!("test_run_{}", unique_id));
            std::fs::create_dir_all(&temp_dir).unwrap();
            let old_cwd = std::env::current_dir().unwrap();
            std::env::set_current_dir(&temp_dir).unwrap();
            Self { old_cwd, temp_dir }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.old_cwd);
            let _ = std::fs::remove_dir_all(&self.temp_dir);
        }
    }

    #[tokio::test]
    async fn test_manager_pane_switching() {
        let mut manager = AppStateManager::new("/left".to_string(), "/right".to_string());
        assert!(manager.active_left);
        assert_eq!(manager.active_pane().current_path, "/left");

        manager.switch_pane();
        assert!(!manager.active_left);
        assert_eq!(manager.active_pane().current_path, "/right");
    }

    #[tokio::test]
    async fn test_manager_handle_enter_directory() {
        let _guard = TempDirGuard::new();
        let mut mock_vfs = MockVfs::new();
        // Expectations:
        // 0. Config file reading (returns error to use default)
        mock_vfs
            .expect_read_file()
            .with(mockall::predicate::eq("config.json"))
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("No config")));

        // 1. Initial refresh for left and right
        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/left"))
            .times(1)
            .returning(|_| {
                Ok(vec![FileMetadata {
                    name: "dir_a".to_string(),
                    size: 0,
                    file_type: FileType::Directory,
                    modified: None,
                }])
            });
        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/right"))
            .times(1)
            .returning(|_| Ok(vec![]));

        // 2. Refresh after entering dir_a
        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/left/dir_a"))
            .times(1)
            .returning(|_| {
                Ok(vec![FileMetadata {
                    name: "..".to_string(),
                    size: 0,
                    file_type: FileType::Directory,
                    modified: None,
                }])
            });

        let mut manager = AppStateManager::new("/left".to_string(), "/right".to_string());
        manager.init(&mock_vfs).await.unwrap();

        assert_eq!(manager.active_pane().entries[0].name, "dir_a");

        // Enter the directory
        manager.handle_enter(&mock_vfs).await.unwrap();
        assert_eq!(manager.active_pane().current_path, "/left/dir_a");
        assert_eq!(manager.active_pane().entries[0].name, "..");
    }

    #[tokio::test]
    async fn test_manager_go_to_path() {
        let _guard = TempDirGuard::new();
        let mut mock_vfs = MockVfs::new();
        mock_vfs
            .expect_read_file()
            .with(mockall::predicate::eq("config.json"))
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("No config")));

        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/left"))
            .times(1)
            .returning(|_| Ok(vec![]));
        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/right"))
            .times(1)
            .returning(|_| Ok(vec![]));

        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/new_path"))
            .times(1)
            .returning(|_| {
                Ok(vec![FileMetadata {
                    name: "file.txt".to_string(),
                    size: 5,
                    file_type: FileType::File,
                    modified: None,
                }])
            });

        let mut manager = AppStateManager::new("/left".to_string(), "/right".to_string());
        manager.init(&mock_vfs).await.unwrap();

        manager.start_go_to_path();
        assert_eq!(manager.mode, InputMode::GoToPath);
        assert_eq!(manager.input_buffer, "/left");

        manager.input_buffer = "/new_path".to_string();
        manager.commit_go_to_path(&mock_vfs).await.unwrap();

        assert_eq!(manager.mode, InputMode::Normal);
        assert_eq!(manager.active_pane().current_path, "/new_path");
        assert_eq!(manager.active_pane().entries[0].name, "file.txt");
    }

    #[tokio::test]
    async fn test_manager_open_text_file() {
        let _guard = TempDirGuard::new();
        let mut mock_vfs = MockVfs::new();
        mock_vfs
            .expect_read_file()
            .with(mockall::predicate::eq("config.json"))
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("No config")));

        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/left"))
            .times(1)
            .returning(|_| {
                Ok(vec![FileMetadata {
                    name: "readme.md".to_string(),
                    size: 12,
                    file_type: FileType::File,
                    modified: None,
                }])
            });
        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/right"))
            .times(1)
            .returning(|_| Ok(vec![]));

        mock_vfs
            .expect_read_file()
            .with(mockall::predicate::eq("/left/readme.md"))
            .times(1)
            .returning(|_| Ok("line 1\nline 2\nline 3".to_string()));

        let mut manager = AppStateManager::new("/left".to_string(), "/right".to_string());
        manager.init(&mock_vfs).await.unwrap();

        assert_eq!(manager.active_pane().entries[0].name, "readme.md");
        assert_eq!(manager.mode, InputMode::Normal);

        // Enter on text file
        manager.handle_enter(&mock_vfs).await.unwrap();

        assert_eq!(manager.mode, InputMode::FileViewer);
        let viewer = manager.file_viewer.as_ref().unwrap();
        assert_eq!(viewer.file_name, "readme.md");
        assert_eq!(viewer.lines, vec!["line 1", "line 2", "line 3"]);
        assert_eq!(viewer.scroll_offset, 0);
    }

    #[tokio::test]
    async fn test_manager_open_unsupported_file() {
        let _guard = TempDirGuard::new();
        let mut mock_vfs = MockVfs::new();
        mock_vfs
            .expect_read_file()
            .with(mockall::predicate::eq("config.json"))
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("No config")));

        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/left"))
            .times(1)
            .returning(|_| {
                Ok(vec![FileMetadata {
                    name: "image.png".to_string(),
                    size: 1024,
                    file_type: FileType::File,
                    modified: None,
                }])
            });
        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/right"))
            .times(1)
            .returning(|_| Ok(vec![]));

        let mut manager = AppStateManager::new("/left".to_string(), "/right".to_string());
        manager.init(&mock_vfs).await.unwrap();

        assert_eq!(manager.active_pane().entries[0].name, "image.png");
        assert_eq!(manager.mode, InputMode::Normal);

        // Enter on unsupported file
        manager.handle_enter(&mock_vfs).await.unwrap();

        assert_eq!(manager.mode, InputMode::Normal);
        assert!(manager.file_viewer.is_none());
        assert!(manager
            .status_message
            .as_ref()
            .unwrap()
            .contains("Unsupported file type"));
    }

    #[tokio::test]
    async fn test_manager_file_viewer_scrolling() {
        let mut manager = AppStateManager::new("/left".to_string(), "/right".to_string());
        manager.file_viewer = Some(FileViewerState {
            file_path: "/left/file.txt".to_string(),
            file_name: "file.txt".to_string(),
            lines: (1..=20).map(|i| format!("line {}", i)).collect(),
            scroll_offset: 0,
        });
        manager.mode = InputMode::FileViewer;

        // Scroll down 5 lines with visible height 10
        manager.scroll_viewer_down(5, 10);
        assert_eq!(manager.file_viewer.as_ref().unwrap().scroll_offset, 5);

        // Scroll down another 10 lines (total 15, but lines.len() = 20, visible height = 10, so max_scroll = 10)
        manager.scroll_viewer_down(10, 10);
        assert_eq!(manager.file_viewer.as_ref().unwrap().scroll_offset, 10);

        // Scroll up 3 lines
        manager.scroll_viewer_up(3);
        assert_eq!(manager.file_viewer.as_ref().unwrap().scroll_offset, 7);

        // Scroll up too much (clamp to 0)
        manager.scroll_viewer_up(20);
        assert_eq!(manager.file_viewer.as_ref().unwrap().scroll_offset, 0);

        // Close viewer
        manager.close_file_viewer();
        assert_eq!(manager.mode, InputMode::Normal);
        assert!(manager.file_viewer.is_none());
    }

    #[tokio::test]
    async fn test_manager_open_in_editor_missing() {
        let mut manager = AppStateManager::new("/left".to_string(), "/right".to_string());
        manager.file_viewer = Some(FileViewerState {
            file_path: "/left/file.txt".to_string(),
            file_name: "file.txt".to_string(),
            lines: vec!["hello".to_string()],
            scroll_offset: 0,
        });
        manager.mode = InputMode::FileViewer;

        manager.config.editors.clear();

        manager.open_in_editor();

        assert_eq!(
            manager.status_message.as_deref(),
            Some("Error: No editor is defined")
        );
    }

    #[tokio::test]
    async fn test_manager_open_in_editor_invalid_path() {
        let mut manager = AppStateManager::new("/left".to_string(), "/right".to_string());
        manager.file_viewer = Some(FileViewerState {
            file_path: "/left/file.txt".to_string(),
            file_name: "file.txt".to_string(),
            lines: vec!["hello".to_string()],
            scroll_offset: 0,
        });
        manager.mode = InputMode::FileViewer;

        manager.config.editors.insert(
            "vscode".to_string(),
            "/invalid/path/to/nonexistent/code".to_string(),
        );

        manager.open_in_editor();

        assert_eq!(
            manager.status_message.as_deref(),
            Some("Error: No editor is defined")
        );
    }

    #[tokio::test]
    async fn test_find_vscode_on_path() {
        use std::env;
        use std::fs::{create_dir_all, File};

        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("mock_path_{}", unique_id));
        create_dir_all(&temp_dir).unwrap();

        let code_path = temp_dir.join("code");
        File::create(&code_path).unwrap();

        // Save old PATH
        let old_path = env::var_os("PATH");

        // Set new PATH to include our temp_dir
        env::set_var("PATH", temp_dir.to_str().unwrap());

        let editor = SupportedEditor::VsCode;
        let found = editor.find_path();

        // Restore old PATH
        if let Some(p) = old_path {
            env::set_var("PATH", p);
        } else {
            env::remove_var("PATH");
        }

        // Clean up
        let _ = std::fs::remove_file(code_path);
        let _ = std::fs::remove_dir(temp_dir);

        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_config_caching_behavior() {
        use std::env;
        use std::fs::File;

        let _guard = TempDirGuard::new();
        let current_temp_dir = env::current_dir().unwrap();

        let code_path = current_temp_dir.join("code");
        File::create(&code_path).unwrap();

        // Mock Vfs
        let mut mock_vfs = MockVfs::new();
        mock_vfs
            .expect_read_file()
            .with(mockall::predicate::eq("config.json"))
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("No config")));

        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/left"))
            .times(1)
            .returning(|_| Ok(vec![]));
        mock_vfs
            .expect_read_dir()
            .with(mockall::predicate::eq("/right"))
            .times(1)
            .returning(|_| Ok(vec![]));

        let old_path = env::var_os("PATH");
        env::set_var("PATH", current_temp_dir.to_str().unwrap());

        let mut manager = AppStateManager::new("/left".to_string(), "/right".to_string());
        let init_res = manager.init(&mock_vfs).await;

        // Restore PATH
        if let Some(p) = old_path {
            env::set_var("PATH", p);
        } else {
            env::remove_var("PATH");
        }

        init_res.unwrap();

        // Verify that vscode is cached in manager config
        assert!(manager.config.editors.contains_key("vscode"));
        assert!(manager
            .config
            .editors
            .get("vscode")
            .unwrap()
            .contains("code"));
    }

    #[tokio::test]
    async fn test_config_serialization_sorted() {
        let mut extensions = HashSet::new();
        extensions.insert("rs".to_string());
        extensions.insert("json".to_string());
        extensions.insert("c".to_string());
        extensions.insert("bat".to_string());

        let cfg = AppConfig {
            text_extensions: extensions,
            editors: HashMap::new(),
        };

        let json_str = serde_json::to_string(&cfg).unwrap();

        assert_eq!(
            json_str,
            r#"{"text_extensions":["bat","c","json","rs"],"editors":{}}"#
        );
    }
}
