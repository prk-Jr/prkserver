use crate::domain::models::config::{Endpoint, Model};

pub fn generate_handler(model: &Model, endpoint: &Endpoint) -> String {
    let method = endpoint.method.to_lowercase();
    let path = &endpoint.path;
    let handler_name = format!(
        "{}{}",
        method,
        path.replace("/", "_").replace("{", "by_").replace("}", "")
    );
    let service_field = format!("{}_service", model.name.to_lowercase());
    let model_name = &model.name;
    let model_lower = model_name.to_lowercase();

    let mut handler_code = String::new();
    let mut extractors = vec!["State(state): State<AppState>".to_string()];

    // Path parameters (e.g., /todos/{id})
    if let Some(path_params) = &endpoint.path_params {
        handler_code.push_str("#[derive(Deserialize)]\n");
        handler_code.push_str(&format!("pub struct {}PathParams {{\n", model_name));
        for param in path_params {
            handler_code.push_str(&format!("    pub {}: {},\n", param.name, param.field_type));
        }
        handler_code.push_str("}\n\n");
        extractors.push(format!("Path(params): Path<{}PathParams>", model_name));
    }

    // Query parameters
    if endpoint.query_params.is_some() {
        // Similar struct generation could be added here
        extractors.push(format!("Query(query): Query<{}QueryParams>", model_name));
    }

    // Body parameters (for POST, PUT, etc.)
    if endpoint.body_params.is_some() {
        extractors.push(format!("Json(body): Json<{}>", model_name));
    }

    let extractor_str = extractors.join(", ");

    // Handler body based on method and path
    let handler_body = match (method.to_lowercase().as_str(), path.as_str()) {
        ("get", p) if p == format!("/{}", model_lower) => {
            format!("let items = state.{}.get_all().await?;\nOk(Json(items))", service_field)
        }
        ("post", p) if p == format!("/{}", model_lower) => {
            format!("let item = state.{}.create(body).await?;\nOk(Json(item))", service_field)
        }
        ("get", p) if p.contains("{") => {
            format!(
                "let item = state.{}.get_by_id(params.id).await?.ok_or(AppError::NotFound(format!(\"Not found\")))?;\nOk(Json(item))",
                service_field
            )
        }
        _ => "// TODO: Implement handler logic\ntodo!()".to_string(),
    };

    handler_code.push_str(&format!(
        "pub async fn {}({}) -> Result<Json<{}>, AppError> {{\n\
             {}\n\
         }}\n",
        handler_name, extractor_str, model_name, handler_body
    ));

    handler_code
}