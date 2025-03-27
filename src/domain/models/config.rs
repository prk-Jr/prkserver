use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub project_name: String,
    pub database_url: String,
    pub database_type: String,
    pub models: Vec<Model>,
    pub middlewares: Option<Vec<Middleware>>,
    pub framework: Framework,
}

#[derive(Clone, Deserialize, Debug)]
pub enum Framework {
    Axum,
    ActixWeb,
}

#[derive(Deserialize, Clone)]
pub struct Model {
    pub name: String,
    pub table_name: String,
    pub fields: Vec<Field>,
    pub endpoints: Option<Vec<Endpoint>>,
}

#[derive(Deserialize, Clone)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
}

#[derive(Deserialize, Clone)]
pub struct Endpoint {
    pub method: String,
    pub path: String,
    pub middlewares: Option<Vec<String>>,
    pub path_params: Option<Vec<Field>>,
    pub body_params: Option<Vec<Field>>,
    pub query_params: Option<Vec<Field>>,
}

#[derive(Deserialize, Clone)]
pub struct Middleware {
    pub model: String,
    // Add additional fields as needed
}