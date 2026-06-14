use crate::vfs::{FileMetadata, FileType, Vfs};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct LocalVfs;

#[async_trait]
impl Vfs for LocalVfs {
    async fn metadata(&self, path: &str) -> Result<FileMetadata> {
        let path_obj = Path::new(path);
        let name = path_obj.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();
        
        let meta = tokio::fs::metadata(path_obj).await?;
        let file_type = if meta.is_dir() {
            FileType::Directory
        } else if meta.is_symlink() {
            FileType::Symlink
        } else if meta.is_file() {
            FileType::File
        } else {
            FileType::Unknown
        };
        
        let modified = meta.modified().ok();
        
        Ok(FileMetadata {
            name,
            size: meta.len(),
            file_type,
            modified,
        })
    }

    async fn read_dir(&self, path: &str) -> Result<Vec<FileMetadata>> {
        let path_obj = Path::new(path);
        let mut entries = Vec::new();
        
        // Add ".." entry if a parent exists and path is not "/" or empty
        if let Some(_parent) = path_obj.parent() {
            if path_obj != Path::new("/") && !path_obj.as_os_str().is_empty() {
                entries.push(FileMetadata {
                    name: "..".to_string(),
                    size: 0,
                    file_type: FileType::Directory,
                    modified: None,
                });
            }
        }
        
        let mut dir = tokio::fs::read_dir(path_obj).await?;
        while let Some(entry) = dir.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = entry.metadata().await?;
            let file_type = if meta.is_dir() {
                FileType::Directory
            } else if meta.is_symlink() {
                FileType::Symlink
            } else if meta.is_file() {
                FileType::File
            } else {
                FileType::Unknown
            };
            
            let modified = meta.modified().ok();
            
            entries.push(FileMetadata {
                name,
                size: meta.len(),
                file_type,
                modified,
            });
        }
        
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir, write, remove_file, remove_dir};

    #[tokio::test]
    async fn test_local_vfs_metadata() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let file_path = std::env::temp_dir().join(format!("test_file_{}.txt", unique_id));
        write(&file_path, b"hello world").unwrap();

        let vfs = LocalVfs;
        let meta = vfs.metadata(file_path.to_str().unwrap()).await.unwrap();

        assert_eq!(meta.name, format!("test_file_{}.txt", unique_id));
        assert_eq!(meta.size, 11);
        assert_eq!(meta.file_type, FileType::File);
        assert!(meta.modified.is_some());

        let _ = remove_file(file_path);
    }

    #[tokio::test]
    async fn test_local_vfs_read_dir() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sub_dir = std::env::temp_dir().join(format!("subdir_{}", unique_id));
        create_dir(&sub_dir).unwrap();
        
        let file_path = sub_dir.join("file.txt");
        write(&file_path, b"abc").unwrap();

        let vfs = LocalVfs;
        let entries = vfs.read_dir(sub_dir.to_str().unwrap()).await.unwrap();

        // Should contain ".." (parent) and "file.txt"
        assert!(entries.iter().any(|e| e.name == ".."));
        assert!(entries.iter().any(|e| e.name == "file.txt"));
        
        let file_meta = entries.iter().find(|e| e.name == "file.txt").unwrap();
        assert_eq!(file_meta.size, 3);
        assert_eq!(file_meta.file_type, FileType::File);

        let _ = remove_file(file_path);
        let _ = remove_dir(sub_dir);
    }
}
