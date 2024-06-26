use std::fs;

use serde::Deserialize;

use crate::{Middleware, Model};

#[derive(Deserialize)]
pub struct Config {
    // pub authorization: bool,
    pub project_name: String,
    pub database_url: String,
    pub database_type: String,
    pub models: Vec<Model>,
    pub middlewares: Option<Vec<Middleware>>,
}
pub fn read_config(path: &str) -> std::io::Result<Config> {
    let config_contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_contents).expect("missing config file");
    Ok(config)
}
