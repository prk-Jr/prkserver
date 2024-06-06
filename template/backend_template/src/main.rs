use axum::{routing::get, Router};
use env_logger;
use log::info;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;

use std::fs;
use std::net::SocketAddr;

mod routes;
use routes::*;

mod models;
use models::*;

#[derive(Deserialize)]
struct Config {
    database_url: String,
}

async fn read_config() -> Config {
    let config_contents = fs::read_to_string("../config.toml").expect("Failed to read config file");
    toml::from_str(&config_contents).expect("Failed to parse config file")
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = read_config().await;

    // connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Could not connect to the database");

    info!("Connected to the database");

    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
