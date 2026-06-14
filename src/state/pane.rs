use crate::vfs::{FileMetadata, FileType, Vfs};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PaneState {
    pub current_path: String,
    pub entries: Vec<FileMetadata>,
    pub selected_index: usize,
}

impl PaneState {
    pub fn new(initial_path: String) -> Self {
        Self {
            current_path: initial_path,
            entries: Vec::new(),
            selected_index: 0,
        }
    }

    pub async fn refresh(&mut self, vfs: &dyn Vfs) -> Result<()> {
        let raw_entries = vfs.read_dir(&self.current_path).await?;
        
        let mut dotdot = None;
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        
        for entry in raw_entries {
            if entry.name == ".." {
                dotdot = Some(entry);
            } else if entry.file_type == FileType::Directory {
                dirs.push(entry);
            } else {
                files.push(entry);
            }
        }
        
        dirs.sort_by_key(|a| a.name.to_lowercase());
        files.sort_by_key(|a| a.name.to_lowercase());
        
        let mut sorted = Vec::new();
        if let Some(dd) = dotdot {
            sorted.push(dd);
        }
        sorted.extend(dirs);
        sorted.extend(files);
        
        self.entries = sorted;
        if self.selected_index >= self.entries.len() {
            self.selected_index = self.entries.len().saturating_sub(1);
        }
        Ok(())
    }

    pub fn selected_entry(&self) -> Option<&FileMetadata> {
        self.entries.get(self.selected_index)
    }

    pub fn select_next(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.entries.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        if !self.entries.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::MockVfs;
    use crate::vfs::FileMetadata;
    use crate::vfs::FileType;

    #[tokio::test]
    async fn test_pane_state_refresh_and_sorting() {
        let mut mock_vfs = MockVfs::new();
        mock_vfs.expect_read_dir()
            .with(mockall::predicate::eq("/test"))
            .times(1)
            .returning(|_| {
                Ok(vec![
                    FileMetadata {
                        name: "file_b.txt".to_string(),
                        size: 100,
                        file_type: FileType::File,
                        modified: None,
                    },
                    FileMetadata {
                        name: "dir_b".to_string(),
                        size: 0,
                        file_type: FileType::Directory,
                        modified: None,
                    },
                    FileMetadata {
                        name: "..".to_string(),
                        size: 0,
                        file_type: FileType::Directory,
                        modified: None,
                    },
                    FileMetadata {
                        name: "dir_a".to_string(),
                        size: 0,
                        file_type: FileType::Directory,
                        modified: None,
                    },
                    FileMetadata {
                        name: "file_a.txt".to_string(),
                        size: 50,
                        file_type: FileType::File,
                        modified: None,
                    },
                ])
            });

        let mut state = PaneState::new("/test".to_string());
        state.refresh(&mock_vfs).await.unwrap();

        // Check entries order: ".." then directories sorted, then files sorted
        assert_eq!(state.entries.len(), 5);
        assert_eq!(state.entries[0].name, "..");
        assert_eq!(state.entries[1].name, "dir_a");
        assert_eq!(state.entries[2].name, "dir_b");
        assert_eq!(state.entries[3].name, "file_a.txt");
        assert_eq!(state.entries[4].name, "file_b.txt");
    }

    #[test]
    fn test_pane_navigation() {
        let mut state = PaneState::new("/test".to_string());
        state.entries = vec![
            FileMetadata { name: "..".to_string(), size: 0, file_type: FileType::Directory, modified: None },
            FileMetadata { name: "dir_a".to_string(), size: 0, file_type: FileType::Directory, modified: None },
            FileMetadata { name: "file_a.txt".to_string(), size: 10, file_type: FileType::File, modified: None },
        ];
        state.selected_index = 0;

        state.select_next();
        assert_eq!(state.selected_index, 1);
        
        state.select_next();
        assert_eq!(state.selected_index, 2);

        // Clamp at end
        state.select_next();
        assert_eq!(state.selected_index, 2);

        state.select_prev();
        assert_eq!(state.selected_index, 1);

        state.select_prev();
        assert_eq!(state.selected_index, 0);

        // Clamp at start
        state.select_prev();
        assert_eq!(state.selected_index, 0);
    }
}
