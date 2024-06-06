use convert_case::Case;
use convert_case::Casing;
pub fn cargo_toml_content(project_name: &str, database_type: &str) -> String {
    format!(
        r#"
    [package]
    name = "{}"
    version = "0.1.0"
    edition = "2021"
    
    [dependencies]
    axum = "0.7.5"
    tokio = {{ version = "1", features = ["full"] }}
    sqlx = {{ version = "0.7.4", features = ["runtime-tokio-rustls", "{}"] }}
    serde = {{ version = "1.0", features = ["derive"] }}
    toml = "0.8.14"
    env_logger = "0.11.3"
    log = "0.4"

        "#,
        project_name, database_type
    )
}

pub fn main_rs_content(database_type: &str) -> String {
    let pool_pptions = match database_type
        .to_case(Case::UpperCamel)
        .to_lowercase()
        .as_str()
    {
        "sqlite" => "Sqlite",
        "mysql" => "MySql",
        _ => "Pg",
    };

    format!(
        r#"
    use axum::{{routing::get, Router}};
    use sqlx::{};
    use sqlx::Pool;
    use std::net::SocketAddr;
    use log::info;
    use env_logger;
    use serde::Deserialize;
    use std::fs;
    use sqlx::pool::PoolOptions;

    
    mod routes;
    use routes::*;

    mod models;
    use models::*;
    
    #[derive(Deserialize)]
    struct Config {{
        database_url: String,
    }}
    
    async fn read_config() -> Config {{
        let config_contents = fs::read_to_string("config/config.toml")
            .expect("Failed to read config file");
        toml::from_str(&config_contents).expect("Failed to parse config file")
    }}
    
    #[tokio::main]
    async fn main() {{
        env_logger::init();
    
        let config = read_config().await;
    
        // connect to the database
        let pool: Pool<{}> = PoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await
            .expect("Could not connect to the database");
    
        info!("Connected to the database");
    
        // build our application with a single route
        let app = Router::new().route("/", get(|| async {{ "Hello, World!" }}));
    
        // run it with hyper on localhost:3000
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }}
        "#,
        pool_pptions, pool_pptions,
    )
}

pub fn config_toml_content(database_url: &str) -> String {
    format!(
        r#"
database_url = "{}"
        "#,
        database_url
    )
}
