pub fn env_content(database_url: &str) -> String {
    format!(
        r#"
DATABASE_URL = "{}"
        "#,
        database_url
    )
}
