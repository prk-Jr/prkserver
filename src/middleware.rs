use std::fs;

use convert_case::{Case, Casing};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Middleware {
    pub model: String,
    select_from_model: String,
    validate_header: Vec<ValidaHeader>,
}

#[derive(Deserialize)]
struct ValidaHeader {
    model_field: String,
    header_key: String,
}

pub fn generate_middleware(
    project_name: &str,
    database_type: &String,
    middleware: &Middleware,
) -> std::io::Result<()> {
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
    let model_dir = format!("./{}/src/middlewares", project_name);
    let model_path = format!(
        "{}/{}_middleware.rs",
        model_dir,
        middleware.model.to_case(Case::Snake).to_lowercase()
    );
    let mut model_content = String::new();
    model_content.push_str(&format!(
        "use axum::{{
    extract::{{FromRef, FromRequestParts}},
    http::{{request::Parts, StatusCode}},
}};\n
use serde::{{Deserialize, Serialize}};\n
use sqlx::*;\n
use crate::models::*;\n\n

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct {}Middleware(pub {});

",
        &middleware.model, &middleware.model,
    ));

    model_content.push_str(&format!(
        "
    
impl<S> FromRequestParts<S> for {}Middleware
where
    {}Pool: FromRef<S>,
    S: Send + Sync,
{{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {{
     ",
        &middleware.model,
        &database_type,
    ));
    model_content.push_str(&format!(
        " let pool = {}Pool::from_ref(state);\n",
        database_type
    ));

    for field in &middleware.validate_header {
        model_content.push_str(&format!(
            " let {} = _parts.headers.get(\"{}\");\n
            let {} = match {} {{
                Some(value) => value.to_str().unwrap(),
                None => return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from(\"Missing Header field: {}\"))),
            }};\n
            ",
            field.header_key,
            field.header_key,
            field.header_key,
            field.header_key,
            field.header_key,
        ));
    }

    model_content.push_str(&format!(
        "let query = {}::select()\n",
        &middleware.select_from_model
    ));

    for field in &middleware.validate_header {
        model_content.push_str(&format!(
            ".where_{}({})",
            &field.model_field, &field.header_key
        ));
    }

    model_content.push_str(".build();\n");

    model_content.push_str(&format!(
        "let {} = sqlx::query_as(&query).fetch_one(&pool).await.map_err(internal_error)?;\n",
        &middleware.model.to_lowercase(),
    ));

    model_content.push_str(&format!(
        "Ok({}Middleware({}))}}\n\n\n",
        &middleware.model,
        &middleware.model.to_lowercase()
    ));
    model_content.push_str("}\n\n\n");
    model_content.push_str(
        "fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}",
    );

    fs::create_dir_all(model_dir).expect("Failed to create model dir");
    fs::write(model_path, model_content)?;
    Ok(())
}
