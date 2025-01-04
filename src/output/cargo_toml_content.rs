pub fn cargo_toml_content(project_name: &str, database_type: &str, authorization: bool) -> String {
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
sqlx = {{ version = "0.7.4", features = ["runtime-tokio-rustls", "{}"] }}
prkorm = "0.5.4"
serde = {{ version = "1.0", features = ["derive"] }}
{}
        "#,
        project_name,
        database_type,
        if authorization {
            format!("axum-extra = {{ version = \"0.9.3\", features = [\"typed-header\"] }}\njsonwebtoken = \"9.3.0\"\nonce_cell = \"1.19.0\"")
        } else {
            "".to_string()
        }
    )
}
