use std::fs;

use convert_case::{Case, Casing};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Endpoint {
    pub method: String,
    pub path: String,
}

pub fn generate_endpoint(
    project_name: &str,
    endpoint: &Endpoint,
    database_type: &str,
    model_name: &str,
) -> std::io::Result<()> {
    let endpoint_dir = format!("./{}/src/routes", project_name);
    let database_type = match database_type
        .to_string()
        .to_case(Case::UpperCamel)
        .to_lowercase()
        .as_str()
    {
        "sqlite" => "Sqlite",
        "mysql" => "MySql",
        _ => "Pg",
    };
    let endpoint_path = format!(
        "{}/{}{}.rs",
        endpoint_dir,
        endpoint.method.to_lowercase(),
        endpoint
            .path
            .to_lowercase()
            .replace("/", "_")
            .replace(":", "")
    );
    let mut endpoint_content = String::new();
    endpoint_content.push_str(&format!(
        "use axum::{{ extract::State, http::StatusCode}};\n",
    ));
    endpoint_content.push_str(&format!(
        "use axum::Json;
        use sqlx::*;\n",
    ));
    endpoint_content.push_str(&format!("use axum::response::IntoResponse;\n",));
    endpoint_content.push_str(&format!(
        "use crate::models::{}::*;\n\n",
        model_name.to_lowercase()
    ));
    if endpoint.method.to_lowercase() == "get" {
        endpoint_content.push_str(&format!(
            "pub async fn get_all{}(State(db): State<{}Pool>,) -> impl IntoResponse {{\n
            let query = {}::select().build();

            let db_reponse : Result<Vec<{}>, Error> = sqlx::query_as(&query)
                            .fetch_all(&db)
                            .await;

            match db_reponse {{
                Ok(data) => (StatusCode::OK, Json(data)).into_response(),
                Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
            }}
        }}\n",
            endpoint
                .path
                .to_lowercase()
                .replace("/", "_")
                .replace(":", ""),
            database_type,
            model_name,
            model_name
        ));
    }

    endpoint_content.push_str(&format!(
        "pub async fn {}{}(State(db): State<{}Pool>,) -> impl IntoResponse {{\n",
        endpoint.method.to_lowercase(),
        endpoint
            .path
            .to_lowercase()
            .replace("/", "_")
            .replace(":", ""),
        database_type,
    ));
    endpoint_content.push_str("    // TODO: Implement endpoint logic\n");
    endpoint_content.push_str("}\n");
    fs::create_dir_all(endpoint_dir).expect("Failed to create model dir");

    fs::write(endpoint_path, endpoint_content)?;
    Ok(())
}
