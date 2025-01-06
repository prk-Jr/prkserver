use std::fs;

use convert_case::{Case, Casing};
use serde::Deserialize;

use crate::Field;

#[derive(Deserialize)]
pub struct Endpoint {
    pub method: String,
    pub path: String,
    pub middlewares: Option<Vec<String>>,
    pub path_params: Option<Vec<Field>>,
    pub body_params: Option<Vec<Field>>,
    pub query_params: Option<Vec<Field>>,
}

pub fn generate_endpoint(
    project_name: &str,
    endpoint: &Endpoint,
    database_type: &str,
    model_name: &str,
    example: bool,
) -> std::io::Result<()> {
    if endpoint.method.to_lowercase() != "get" && example {
        return Ok(());
    }
    let endpoint_dir = if example {
        format!("./{}/src/routes/example", project_name)
    } else {
        format!("./{}/src/routes", project_name)
    };

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
            .replace("/", "_").replace(":", "").replace("{", "_").replace("}", "_")
    );

    let mut endpoint_content = String::new();
    endpoint_content.push_str(&format!(
        "use axum::{{ extract::{{State, Path, Query }}, http::StatusCode}};\n",
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

    let mut extraction_builder = String::new();

    if let Some(_) = &endpoint.middlewares {
        endpoint_content.push_str(&format!("use crate::middlewares::*;\n",));
    }

    if let Some(middlewares) = &endpoint.middlewares {
        for middleware in middlewares {
            let middleware = middleware
                .to_case(Case::UpperCamel)
                .replace("Middleware", "");

            extraction_builder.push_str(
                format!(
                    "{}Middleware({}): {}Middleware, ",
                    middleware,
                    middleware.to_lowercase(),
                    middleware
                )
                .as_str(),
            );
        }
    }

    if let Some(body) = &endpoint.body_params {
        endpoint_content.push_str(&format!(
            "use serde::{{Deserialize, Serialize}};\n
            ",
        ));
        endpoint_content.push_str(&format!("#[derive(Deserialize)]\n"));
        endpoint_content.push_str(&format!("pub struct {}Body {{\n", model_name));
        for field in body {
            endpoint_content.push_str(&format!("    pub {}: {},\n", field.name, field.field_type));
        }
        endpoint_content.push_str("}\n");
        extraction_builder.push_str(format!("Json(body): Json<{}Body>, ", model_name).as_str());
    }
    if let Some(path) = &endpoint.path_params {
        endpoint_content.push_str(&format!(
            "use serde::{{Deserialize, Serialize}};\n
            ",
        ));
        endpoint_content.push_str(&format!("#[derive(Deserialize)]\n"));
        endpoint_content.push_str(&format!("pub struct {}PathParams {{\n", model_name));
        for field in path {
            endpoint_content.push_str(&format!("    pub {}: {},\n", field.name, field.field_type));
        }
        endpoint_content.push_str("}\n");
        extraction_builder
            .push_str(format!("Path(path_params): Path<{}PathParams>, ", model_name).as_str());
    }
    if let Some(query) = &endpoint.query_params {
        endpoint_content.push_str(&format!(
            "
        use serde::{{Deserialize, Serialize}};\n
            ",
        ));
        endpoint_content.push_str(&format!("#[derive(Deserialize)]\n"));
        endpoint_content.push_str(&format!("pub struct {}PathParams {{\n", model_name));
        for field in query {
            endpoint_content.push_str(&format!("    pub {}: {},\n", field.name, field.field_type));
        }
        endpoint_content.push_str("}\n");
        extraction_builder
            .push_str(format!("Query(query_params): Query<{}QueryParams>, ", model_name).as_str());
    }

    if endpoint.method.to_lowercase() == "get" && example {
        endpoint_content.push_str(&format!(
            "pub async fn get_all{}(State(db): State<{}Pool>, {}) -> impl IntoResponse {{\n
            let query = {}::select().build();

            let db_response : Result<Vec<{}>, Error> = sqlx::query_as(&query)
                            .fetch_all(&db)
                            .await;

            match db_response {{
                Ok(data) => (StatusCode::OK, Json(data)).into_response(),
                Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
            }}
        }}\n",
            endpoint
                .path
                .to_lowercase()
                .replace("/", "_").replace(":", "").replace("{", "_").replace("}", "_"),
            database_type,
            extraction_builder,
            model_name,
            model_name
        ));
    }
    if !example {
        endpoint_content.push_str(&format!(
            "pub async fn {}{}(State(db): State<{}Pool>, {}) -> impl IntoResponse {{\n",
            endpoint.method.to_lowercase(),
            endpoint
                .path
                .to_lowercase()
                .replace("/", "_").replace(":", "").replace("{", "_").replace("}", "_"),
            database_type,
            extraction_builder,
        ));
        endpoint_content.push_str("    // TODO: Implement endpoint logic\n");
        endpoint_content.push_str("}\n");
    }
    fs::create_dir_all(endpoint_dir).expect("Failed to create model dir");

    fs::write(endpoint_path, endpoint_content)?;
    Ok(())
}
