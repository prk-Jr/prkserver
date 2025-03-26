use crate::{domain::models::config::{Config,  Middleware, Model}, output::generate_handler};

pub struct Template {
    pub config: Config,
}

impl Template {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn generate_model_content(&self, model: &Model) -> String {
        let fields = model.fields.iter()
            .map(|f| format!("    pub {}: {},", f.name, f.field_type))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "use serde::{{Deserialize, Serialize}};\n\
             use prkorm::Table;\n\
             use sqlx::FromRow;\n\n\
             #[derive(Debug, Serialize, Deserialize, Table, Default, FromRow)]\n\
             #[table_name(\"{}\")]\n\
             #[primary_key(\"id\")]\n\
             pub struct {} {{\n\
             {}\n\
             }}",
            model.table_name, model.name, fields
        )
    }

    pub fn generate_router(&self) -> String {
        let mut routes = String::new();
        for model in &self.config.models {
            if let Some(endpoints) = &model.endpoints {
                for endpoint in endpoints {
                    let method = endpoint.method.to_lowercase();
                    let path = &endpoint.path;
                    let handler = format!(
                        "{}{}",
                        method,
                        path.replace("/", "_")
                            .replace(":", "")
                            .replace("{", "by_")
                            .replace("}", "")
                    );
                    routes.push_str(&format!(
                        "    .route(\"{}\", {}({}))\n",
                        path, method, handler
                    ));
                }
            }
        }
        routes
    }

    // pub fn generate_endpoint_content(
    //     &self,
    //     endpoint: &Endpoint,
    //     model_name: &str,
    //     _db_type: &str,
    // ) -> String {
    //     let handler_name = format!(
    //         "{}{}",
    //         endpoint.method.to_lowercase(),
    //         endpoint.path
    //             .replace("/", "_")
    //             .replace(":", "")
    //             .replace("{", "by_")
    //             .replace("}", "")
    //     );
    //     let service_field = format!("{}_service", model_name.to_lowercase());
    //     let model_lower = model_name.to_lowercase();

    //     match (endpoint.method.as_str(), endpoint.path.as_str()) {
    //         ("GET", path) if path == format!("/{}", model_lower) => {
    //             format!(
    //                 "use axum::extract::State;\n\
    //                  use axum::Json;\n\
    //                  use crate::application::services::{}Service;\n\
    //                  use crate::domain::error::AppError;\n\
    //                  use crate::domain::models::{}::{};\n\n\
    //                  pub async fn {}(State(state): State<AppState>) -> Result<Json<Vec<{}>>, AppError> {{\n\
    //                      let items = state.{}.get_all().await?;\n\
    //                      Ok(Json(items))\n\
    //                  }}",
    //                 model_name, model_lower, model_name, handler_name, model_name, service_field
    //             )
    //         }
    //         ("GET", path) if path.starts_with(&format!("/{}/", model_lower)) => {
    //             format!(
    //                 "use axum::extract::{{State, Path}};\n\
    //                  use axum::Json;\n\
    //                  use crate::application::services::{}Service;\n\
    //                  use crate::domain::error::AppError;\n\
    //                  use crate::domain::models::{}::{};\n\n\
    //                  pub async fn {}(State(state): State<AppState>, Path(id): Path<i32>) -> Result<Json<Option<{}>>, AppError> {{\n\
    //                      let item = state.{}.get_by_id(id).await?;\n\
    //                      Ok(Json(item))\n\
    //                  }}",
    //                 model_name, model_lower, model_name, handler_name, model_name, service_field
    //             )
    //         }
    //         ("POST", path) if path == format!("/{}", model_lower) => {
    //             format!(
    //                 "use axum::extract::State;\n\
    //                  use axum::Json;\n\
    //                  use crate::application::services::{}Service;\n\
    //                  use crate::domain::error::AppError;\n\
    //                  use crate::domain::models::{}::{};\n\n\
    //                  pub async fn {}(State(state): State<AppState>, Json(payload): Json<{}>) -> Result<Json<{}>, AppError> {{\n\
    //                      let item = state.{}.create(payload).await?;\n\
    //                      Ok(Json(item))\n\
    //                  }}",
    //                 model_name, model_lower, model_name, handler_name, model_name, model_name, service_field
    //             )
    //         }
    //         _ => format!(
    //             "// TODO: Implement handler for {} {}",
    //             endpoint.method, endpoint.path
    //         ),
    //     }
    // }

    pub fn generate_middleware_content(&self, middleware: &Middleware, _db_type: &str) -> String {
        let model_lower = middleware.model.to_lowercase();
        format!(
            "use axum::{{extract::State, middleware::Next, response::Response, http::Request}};\n\
             use crate::adapters::http::http::AppState;\n\
             use crate::domain::error::AppError;\n\n\
             use axum::body::Body;\n\n

             pub async fn {}_middleware<B>(\n\
                 State(state): State<AppState>,\n\
                 request: Request<Body>,\n
                 next: Next,\n\
             ) -> Result<Response, AppError> {{\n\
                 // Example middleware: Log the request\n\
                 tracing::info!(\"Processing request for {} model\");\n\
                 let response = next.run(request).await;\n\
                 Ok(response)\n\
             }}",
            model_lower, middleware.model
        )
    }

    pub fn generate_repository_trait(&self, model: &Model) -> String {
        let model_name = &model.name;
        format!(
            "use std::future::Future;\n\
             use crate::domain::models::{}::{};\n\
             use crate::domain::error::AppError;\n\n\
             pub trait {}Repository: Send + Sync + 'static {{\n\
                 fn find_all(&self) -> impl Future<Output = Result<Vec<{}>, AppError>> + Send;\n\
                 fn find_by_id(&self, id: i32) -> impl Future<Output = Result<Option<{}>, AppError>> + Send;\n\
                 fn create(&self, body: {}) -> impl Future<Output = Result<{}, AppError>> + Send;\n\
             }}",
            model_name.to_lowercase(), model_name, model_name, model_name, model_name, model_name, model_name
        )
    }

    pub fn generate_repository_impl(&self, model: &Model, database_type: &str) -> String {
        let model_name = &model.name;
        let pool_type = match database_type.to_lowercase().as_str() {
            "mysql" => "sqlx::MySqlPool",
            "postgres" => "sqlx::PgPool",
            _ => panic!("Unsupported database type: {}", database_type),
        };
        format!(
            "use crate::domain::models::{}::{};\n\
             use crate::domain::ports::{}_repository::{}Repository;\n\
             use crate::domain::error::AppError;\n\
             use sqlx::{};\n\n\
             #[derive(Clone)]\n\
             pub struct Sqlx{}Repository {{\n\
                 pool: {},\n\
             }}\n\n\
             impl Sqlx{}Repository {{\n\
                 pub fn new(pool: {}) -> Self {{\n\
                     Self {{ pool }}\n\
                 }}\n\
             }}\n\n\
             impl {}Repository for Sqlx{}Repository {{\n\
                 async fn find_all(&self) -> Result<Vec<{}>, AppError> {{\n\
                    let query = {}::select().build();\n\
                       sqlx::query_as(&query).fetch_all(&self.pool).await.map_err(AppError::from) \n\
                 }}\n\
                 async fn find_by_id(&self, id: i32) -> Result<Option<{}>, AppError> {{\n\
                    let query = {}::select().where_id(id).build();\n\
                       sqlx::query_as(&query).fetch_optional(&self.pool).await.map_err(AppError::from) \n\
                 }}\n\
                 async fn create(&self, body: {}) -> Result<{}, AppError> {{\n\
                    todo!()\n\
                 }}\n\
                 // Implement create, update, delete similarly\n\
             }}",
            model_name.to_lowercase(), model_name, model_name.to_lowercase(), model_name, database_type, model_name, pool_type,
            model_name, pool_type, model_name, model_name, model_name,model_name, model_name, model_name, model_name, model_name
        )
    }

    pub fn generate_service(&self, model: &Model) -> String {
        let model_name = &model.name;
        format!(
            "use crate::domain::models::{}::{};\n\
             use crate::domain::ports::{}_repository::{}Repository;\n\
             use crate::domain::error::AppError;\n\n\
             #[derive(Clone)]\n\
             pub struct {}Service<R: {}Repository> {{\n\
                 repo: R,\n\
             }}\n\n\
             impl<R: {}Repository> {}Service<R> {{\n\
                 pub fn new(repo: R) -> Self {{\n\
                     Self {{ repo }}\n\
                 }}\n\n\
                 pub async fn get_all(&self) -> Result<Vec<{}>, AppError> {{\n\
                     self.repo.find_all().await\n\
                 }}\n\
                 pub async fn get_by_id(&self, id: i32) -> Result<Option<{}>, AppError> {{\n\
                     self.repo.find_by_id(id).await\n\
                 }}\n\
                    pub async fn create(&self, body: {}) -> Result<{}, AppError> {{\n\
                        self.repo.create(body).await\n\
                    }}\n\
                 // Add other methods as needed\n\
             }}",
            model_name.to_lowercase(), model_name, model_name.to_lowercase(), model_name, model_name, model_name, model_name, model_name, model_name, model_name, model_name, model_name
        )
    }

    pub fn generate_error_content(&self) -> String {
        "use axum::{response::IntoResponse, http::StatusCode};\n\
         use thiserror::Error;\n\n\
         #[derive(Error, Debug)]\n\
         pub enum AppError {\n\
             #[error(\"Database error: {0}\")]\n\
             Database(#[from] sqlx::Error),\n\
             #[error(\"Not found: {0}\")]\n\
             NotFound(String),\n\
         }\n\n\
         impl IntoResponse for AppError {\n\
             fn into_response(self) -> axum::response::Response {\n\
                 match self {\n\
                     AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),\n\
                     AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),\n\
                 }\n\
             }\n\
         }".to_string()
    }

    pub fn generate_http_content(&self) -> String {
        let template = self;
        // Generate AppState struct with fields for each model's service
        let app_state_fields = template.config.models.iter().map(|model| {
            let service_name = format!("{}_service", model.name.to_lowercase());
            let repo_type = format!("Sqlx{}Repository", model.name);
            format!("    pub {}: Arc<services::{}Service<{}>>,\n", service_name, model.name, repo_type)
        }).collect::<String>();
        
        let app_state = format!(
            "#[derive(Clone)]\n\
             pub struct AppState {{\n\
             {}\n\
             }}",
            app_state_fields
        );
    
        // Generate parameters for HttpServer::new
        let new_params = template.config.models.iter()
            .map(|m| {
                let service_name = format!("{}_service", m.name.to_lowercase());
                format!("{}: services::{}Service<Sqlx{}Repository>", service_name, m.name, m.name)
            })
            .collect::<Vec<_>>()
            .join(", ");
    
        // Generate state initialization in HttpServer::new
        let state_fields = template.config.models.iter()
            .map(|m| {
                let service_name = format!("{}_service", m.name.to_lowercase());
                format!("        {}: Arc::new({}),", service_name, service_name)
            })
            .collect::<Vec<_>>()
            .join("\n");
    
        // Generate handler functions
        let handlers: Vec<String> = template.config.models.iter().flat_map(|model| {
            model.endpoints.clone().map(|endpoint| {
                endpoint.iter().map(|endpoint|

                generate_handler(model, endpoint)
                ).collect::<Vec<_>>()
            })
        }).flatten().collect();

        let handlers = format!(
            "{}\n\n\
             ",
             handlers.join("\n\n")
        );
    
        // Get the router content
        let router = template.generate_router();
    
        format!(
            r#"
    /*!
        Module `http` exposes an HTTP server that handles HTTP requests to the application. Its
        implementation is opaque to module consumers.
    */
    
    use std::{{net::SocketAddr, sync::Arc}};
    
    use anyhow::Context;
    use axum::{{
        routing::{{get, post}},Json,
        Router,
        http::StatusCode,
        extract::*,
    }};
    use tokio::net;
    use tower_http::{{cors::CorsLayer, trace::TraceLayer}};
    use crate::infrastructure::*;
    use crate::domain::*;
    use crate::application::services;
    use crate::domain::error::AppError;
    use serde::Deserialize;
    
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct HttpServerConfig<'a> {{
        pub port: &'a str,
    }}
    
    {app_state}
    
    {handlers}
    
    pub struct HttpServer {{
        router: Router,
        listener: net::TcpListener,
    }}
    
    impl HttpServer {{
        pub async fn new(
            {new_params},
            config: HttpServerConfig<'_>,
        ) -> anyhow::Result<Self> {{
            let trace_layer = TraceLayer::new_for_http().make_span_with(
                |request: &axum::extract::Request<_>| {{
                    let uri = request.uri().to_string();
                    tracing::info_span!("http_request", method = ?request.method(), uri)
                }}
            );
    
            let state = AppState {{
    {state_fields}
            }};
    
            let router = Router::new()
                .route("/health", get(health_route))
                .nest("/api", api_routes())
                .layer(CorsLayer::permissive())
                .layer(trace_layer)
                .with_state(state);
    
            let addr = SocketAddr::from((
                [0, 0, 0, 0, 0, 0, 0, 0],
                config.port.parse::<u16>().unwrap_or(3000),
            ));
    
            let listener = net::TcpListener::bind(&addr)
                .await
                .with_context(|| format!("failed to listen on port {{}}", config.port))?;
    
            Ok(Self {{ router, listener }})
        }}
    
        pub async fn run(self) -> anyhow::Result<()> {{
            tracing::debug!("listening on {{}}", self.listener.local_addr().unwrap());
            axum::serve(self.listener, self.router)
                .await
                .context("received error from running server")?;
            Ok(())
        }}
    }}
    
    fn api_routes() -> Router<AppState> {{
        Router::new()
    {router}
    }}
    
    async fn health_route() -> (StatusCode, &'static str) {{
        (StatusCode::OK, "OK")
    }}
            "#
        )
    }
}