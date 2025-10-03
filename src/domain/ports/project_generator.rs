use std::error::Error;

use crate::domain::models::config::Config;

pub trait ProjectGenerator: Send + Sync {
    async fn generate_project(&self, config: Config) -> Result<(), Box<dyn Error>>;
}
