use crate::domain::models::config::{Endpoint, Model, Framework};

pub fn generate_handler(model: &Model, endpoint: &Endpoint, framework: Framework) -> String {
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
    let mut extractors = Vec::new();

    // Define framework-specific variable names
    let state_var = match framework {
        Framework::Axum => "state",
        Framework::ActixWeb => "state",
    };
    let params_var = match framework {
        Framework::Axum => "params",
        Framework::ActixWeb => "path",
    };
    let json_type = match framework {
        Framework::Axum => "Json",
        Framework::ActixWeb => "web::Json",
    };

    // State extractor
    let state_extractor = match framework {
        Framework::Axum => format!("State({}): State<AppState>", state_var),
        Framework::ActixWeb => format!("{}: web::Data<AppState>", state_var),
    };
    extractors.push(state_extractor);

    // Path parameters
    if let Some(path_params) = &endpoint.path_params {
        handler_code.push_str("#[derive(Deserialize)]\n");
        handler_code.push_str(&format!("pub struct {}PathParams {{\n", model_name));
        for param in path_params {
            handler_code.push_str(&format!("    pub {}: {},\n", param.name, param.field_type));
        }
        handler_code.push_str("}\n\n");
        let path_extractor = match framework {
            Framework::Axum => format!("Path({}): Path<{}PathParams>", params_var, model_name),
            Framework::ActixWeb => format!("{}: web::Path<{}PathParams>", params_var, model_name),
        };
        extractors.push(path_extractor);
    }

    // Query parameters
    if endpoint.query_params.is_some() {
        extractors.push(format!("Query(query): Query<{}QueryParams>", model_name));
    }

    // Body parameters
    if endpoint.body_params.is_some() {
        let body_extractor = match framework {
            Framework::Axum => format!("Json(body): Json<{}>", model_name),
            Framework::ActixWeb => format!("body: web::Json<{}>", model_name),
        };
        extractors.push(body_extractor);
    }

    let extractor_str = extractors.join(", ");

    // Handler body with parameterized variable names and JSON type
    let handler_body = match (method.as_str(), path.as_str()) {
        ("get", p) if p == format!("/{}", model_lower) => {
            format!(
                "let items = {}.{}.get_all().await?;\nOk({}(items))",
                state_var, service_field, json_type
            )
        }
        ("post", p) if p == format!("/{}", model_lower) => {
            format!(
                "let item = {}.{}.create(body).await?;\nOk({}(item))",
                state_var, service_field, json_type
            )
        }
        ("get", p) if p.contains("{") => {
            format!(
                "let item = {}.{}.get_by_id({}.id).await?.ok_or(AppError::NotFound(format!(\"Not found\")))?;\nOk({}(item))",
                state_var, service_field, params_var, json_type
            )
        }
        _ => "// TODO: Implement handler logic\ntodo!()".to_string(),
    };

    // Define the return type
    let return_type = match framework {
        Framework::Axum => format!("Result<Json<{}>, AppError>", model_name),
        Framework::ActixWeb => format!("Result<web::Json<{}>, AppError>", model_name),
    };

    // Assemble the handler code
    handler_code.push_str(&format!(
        "pub async fn {}({}) -> {} {{\n\
             {}\n\
         }}\n",
        handler_name, extractor_str, return_type, handler_body
    ));

    handler_code
}