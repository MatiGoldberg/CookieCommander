use crate::vfs::Vfs;
use crate::state::pane::PaneState;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub text_extensions: HashSet<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let extensions = vec![
            "txt", "md", "rs", "cs", "py", "java", "c", "cpp", "h", "csproj", "json", "toml", "js", "ts", "html", "css", "yaml", "yml", "sh", "bat"
        ];
        Self {
            text_extensions: extensions.into_iter().map(String::from).collect(),
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
            Ok(content) => {
                match serde_json::from_str::<AppConfig>(&content) {
                    Ok(cfg) => {
                        self.config = cfg;
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Config parse error: {}. Using defaults.", e));
                        self.config = AppConfig::default();
                    }
                }
            }
            Err(_) => {
                // Generate default config.json
                let default_cfg = AppConfig::default();
                if let Ok(json_str) = serde_json::to_string_pretty(&default_cfg) {
                    let _ = tokio::fs::write("config.json", json_str).await;
                }
                self.config = default_cfg;
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
                (entry.name.clone(), entry.file_type == crate::vfs::FileType::Directory, entry.file_type)
            } else {
                return Ok(());
            }
        };

        if !is_dir {
            if file_type == crate::vfs::FileType::File {
                let path = Path::new(&name);
                let ext = path.extension()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::{MockVfs, FileMetadata, FileType};

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
        let mut mock_vfs = MockVfs::new();
        // Expectations:
        // 0. Config file reading (returns error to use default)
        mock_vfs.expect_read_file()
            .with(mockall::predicate::eq("config.json"))
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("No config")));

        // 1. Initial refresh for left and right
        mock_vfs.expect_read_dir()
            .with(mockall::predicate::eq("/left"))
            .times(1)
            .returning(|_| Ok(vec![
                FileMetadata { name: "dir_a".to_string(), size: 0, file_type: FileType::Directory, modified: None }
            ]));
        mock_vfs.expect_read_dir()
            .with(mockall::predicate::eq("/right"))
            .times(1)
            .returning(|_| Ok(vec![]));
        
        // 2. Refresh after entering dir_a
        mock_vfs.expect_read_dir()
            .with(mockall::predicate::eq("/left/dir_a"))
            .times(1)
            .returning(|_| Ok(vec![
                FileMetadata { name: "..".to_string(), size: 0, file_type: FileType::Directory, modified: None }
            ]));

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
        let mut mock_vfs = MockVfs::new();
        mock_vfs.expect_read_file()
            .with(mockall::predicate::eq("config.json"))
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("No config")));

        mock_vfs.expect_read_dir()
            .with(mockall::predicate::eq("/left"))
            .times(1)
            .returning(|_| Ok(vec![]));
        mock_vfs.expect_read_dir()
            .with(mockall::predicate::eq("/right"))
            .times(1)
            .returning(|_| Ok(vec![]));

        mock_vfs.expect_read_dir()
            .with(mockall::predicate::eq("/new_path"))
            .times(1)
            .returning(|_| Ok(vec![
                FileMetadata { name: "file.txt".to_string(), size: 5, file_type: FileType::File, modified: None }
            ]));

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
        let mut mock_vfs = MockVfs::new();
        mock_vfs.expect_read_file()
            .with(mockall::predicate::eq("config.json"))
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("No config")));

        mock_vfs.expect_read_dir()
            .with(mockall::predicate::eq("/left"))
            .times(1)
            .returning(|_| Ok(vec![
                FileMetadata { name: "readme.md".to_string(), size: 12, file_type: FileType::File, modified: None }
            ]));
        mock_vfs.expect_read_dir()
            .with(mockall::predicate::eq("/right"))
            .times(1)
            .returning(|_| Ok(vec![]));

        mock_vfs.expect_read_file()
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
        let mut mock_vfs = MockVfs::new();
        mock_vfs.expect_read_file()
            .with(mockall::predicate::eq("config.json"))
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("No config")));

        mock_vfs.expect_read_dir()
            .with(mockall::predicate::eq("/left"))
            .times(1)
            .returning(|_| Ok(vec![
                FileMetadata { name: "image.png".to_string(), size: 1024, file_type: FileType::File, modified: None }
            ]));
        mock_vfs.expect_read_dir()
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
        assert!(manager.status_message.as_ref().unwrap().contains("Unsupported file type"));
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
}
