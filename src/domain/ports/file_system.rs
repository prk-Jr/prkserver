
pub trait FileSystem: Send + Sync {
    async fn create_dir_all(&self, path: &str) -> std::io::Result<()>;
    async fn write_file(&self, path: &str, content: &str) -> std::io::Result<()>;
    async fn read_to_string(&self, path: &str) -> std::io::Result<String>;
}