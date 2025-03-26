pub fn cargo_toml_content(project_name: &str, database_type: &str, _authorization: bool) -> String {
    format!(
        r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8.1"
dotenvy = "0.15.7"
tokio = {{ version = "1.38.0", features = ["rt-multi-thread"] }}
sqlx = {{ version = "0.8.3", features = ["runtime-tokio-rustls", "{}"] }}
serde = {{ version = "1.0", features = ["derive"] }}
prkorm = "0.5.4"
tower-http = {{ version = "0.5.2", features = ["trace", "cors"] }}
tower-layer = "0.3.2"
tracing = "0.1.40"
tracing-subscriber = "0.3"
anyhow = "1.0.97"
thiserror = "2.0.12"

        "#,
        project_name, database_type
    )
}