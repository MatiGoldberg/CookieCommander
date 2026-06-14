use crate::vfs::Vfs;
use crate::state::pane::PaneState;
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    GoToPath,
}

#[derive(Debug, Clone)]
pub struct AppStateManager {
    pub left_pane: PaneState,
    pub right_pane: PaneState,
    pub active_left: bool,
    pub mode: InputMode,
    pub input_buffer: String,
    pub status_message: Option<String>,
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
        }
    }

    pub async fn init(&mut self, vfs: &dyn Vfs) -> Result<()> {
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
        let (name, is_dir) = {
            let active = self.active_pane();
            if let Some(entry) = active.selected_entry() {
                (entry.name.clone(), entry.file_type == crate::vfs::FileType::Directory)
            } else {
                return Ok(());
            }
        };

        if !is_dir {
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
}
