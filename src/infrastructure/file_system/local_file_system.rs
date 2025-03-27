use crate::domain::ports::file_system::FileSystem;
use std::fs;

pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    async fn create_dir_all(&self, path: &str) -> std::io::Result<()> {
        fs::create_dir_all(path)
    }

    async fn write_file(&self, path: &str, content: &str) -> std::io::Result<()> {
        fs::write(path, content)
    }

    async fn read_to_string(&self, path: &str) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
}