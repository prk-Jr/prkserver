use std::fs;

use serde::Deserialize;

use crate::{Endpoint, Model};

#[derive(Deserialize)]
pub struct Config {
    pub project_name: String,
    pub database_url: String,
    pub database_type: String,
    pub models: Vec<Model>,
    pub endpoints: Vec<Endpoint>,
}
pub fn read_config(path: &str) -> std::io::Result<Config> {
    let config_contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_contents).expect("missing config file");
    Ok(config)
}
