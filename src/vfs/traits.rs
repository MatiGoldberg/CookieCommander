#![allow(dead_code)]

use anyhow::Result;
use async_trait::async_trait;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileMetadata {
    pub name: String,
    pub size: u64,
    pub file_type: FileType,
    pub modified: Option<SystemTime>,
    // For macOS specifics, we can later add fields for xattr presence, etc.
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Vfs: Send + Sync {
    /// Returns the metadata for a given path.
    async fn metadata(&self, path: &str) -> Result<FileMetadata>;

    /// Reads a directory and returns a list of its entries.
    async fn read_dir(&self, path: &str) -> Result<Vec<FileMetadata>>;

    /// Reads a file and returns its content.
    async fn read_file(&self, path: &str) -> Result<String>;

    /// Writes content to a file, creating it if it doesn't exist or overwriting it if it does.
    async fn write_file(&self, path: &str, content: &str) -> Result<()>;

    /// Creates a directory.
    async fn create_dir(&self, path: &str) -> Result<()>;

    /// Deletes a file.
    async fn remove_file(&self, path: &str) -> Result<()>;

    /// Deletes a directory and all its contents recursively.
    async fn remove_dir_all(&self, path: &str) -> Result<()>;

    /// Copies a single file from one path to another.
    async fn copy_file(&self, from: &str, to: &str) -> Result<()>;
}
