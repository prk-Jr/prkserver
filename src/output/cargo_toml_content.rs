use crate::domain::models::config::{Config, Framework};

pub fn cargo_toml_content(project_name: &str, database_type: &str, _authorization: bool, framework: &Framework) -> String {
    format!(
        r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
dotenvy = "0.15.7"
tokio = {{ version = "1.38.0", features = ["{}"] }}
sqlx = {{ version = "0.8.3", features = ["runtime-tokio-rustls", "{}"] }}
serde = {{ version = "1.0", features = ["derive"] }}
prkorm = "0.5.4"
tower-http = {{ version = "0.5.2", features = ["trace", "cors"] }}
tower-layer = "0.3.2"
tracing = "0.1.40"
tracing-subscriber = "0.3"
anyhow = "1.0.97"
thiserror = "2.0.12"
{}

        "#,
        project_name, generate_tokio_features(framework), database_type, generate_framework(framework)
    )
}

fn generate_framework( framework: &Framework) -> String {
    match framework {
        Framework::Axum => "axum = \"0.8.1\"\n",
        Framework::ActixWeb => "actix-web = \"4\"\n",
    }.into()
}
fn generate_tokio_features( framework: &Framework) -> String {
    match framework {
        Framework::Axum => "rt-multi-thread",
        Framework::ActixWeb => "full",
    }.into()
}