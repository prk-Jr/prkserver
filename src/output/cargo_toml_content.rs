pub fn cargo_toml_content(project_name: &str, database_type: &str) -> String {
    format!(
        r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"
    
[dependencies]
axum = "0.7.5"
dotenvy = "0.15.7"
tokio = {{ version = "1.38.0", features = ["rt-multi-thread"] }}
sqlx = {{ version = "0.7.4", features = ["runtime-tokio-rustls", "{}"] }}
prkorm = "0.5.3"
serde = {{ version = "1.0", features = ["derive"] }}
toml = "0.8.14"
env_logger = "0.11.3"
log = "0.4"

        "#,
        project_name, database_type
    )
}
