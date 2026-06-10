#![allow(unused_imports)]

pub mod traits;

pub use traits::{FileMetadata, FileType, Vfs};

#[cfg(test)]
pub use traits::MockVfs;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_file_metadata_creation() {
        let now = SystemTime::now();
        let meta = FileMetadata {
            name: "test.txt".to_string(),
            size: 1024,
            file_type: FileType::File,
            modified: Some(now),
        };

        assert_eq!(meta.name, "test.txt");
        assert_eq!(meta.size, 1024);
        assert_eq!(meta.file_type, FileType::File);
        assert_eq!(meta.modified, Some(now));
    }

    #[test]
    fn test_file_type_equality() {
        assert_eq!(FileType::Directory, FileType::Directory);
        assert_ne!(FileType::File, FileType::Directory);
    }
}
