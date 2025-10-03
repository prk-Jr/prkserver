pub fn env_content(database_url: &str) -> String {
    format!("DATABASE_URL={}", database_url)
}
