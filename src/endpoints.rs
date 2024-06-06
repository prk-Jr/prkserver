use std::fs;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Endpoint {
    pub model: String,
    #[serde(rename = "type")]
    pub endpoint_type: String,
    pub path: String,
}

pub fn generate_endpoint(project_name: &str, endpoint: &Endpoint) -> std::io::Result<()> {
    let endpoint_dir = format!("./{}/src/routes", project_name);

    let endpoint_path = format!("{}/{}.rs", endpoint_dir, endpoint.model.to_lowercase());
    let mut endpoint_content = String::new();
    endpoint_content.push_str(&format!(
        "use axum::{{routing::{}, Router}};\n",
        endpoint.endpoint_type.to_lowercase()
    ));
    endpoint_content.push_str(&format!("use axum::response::IntoResponse;\n",));
    endpoint_content.push_str(&format!(
        "use crate::models::{}::*;\n\n",
        endpoint.model.to_lowercase()
    ));
    endpoint_content.push_str(&format!(
        "pub async fn {}() -> impl IntoResponse {{\n",
        endpoint.path.to_lowercase().replace("/", "_")
    ));
    endpoint_content.push_str("    // TODO: Implement endpoint logic\n");
    endpoint_content.push_str("}\n");
    fs::create_dir_all(endpoint_dir).expect("Failed to create model dir");

    fs::write(endpoint_path, endpoint_content)?;
    Ok(())
}
