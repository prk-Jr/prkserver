use crate::{domain::models::config::{Config,  Middleware, Model}, output::generate_handler};

use super::config::Framework;

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
        match self.config.framework {
            Framework::Axum => self.generate_axum_router(),
            Framework::ActixWeb => self.generate_actix_router(),
        }
        
    }

    fn generate_axum_router(&self) -> String {
        let mut routes = String::new();
        for model in &self.config.models {
            if let Some(endpoints) = &model.endpoints {
                for endpoint in endpoints {
                    let method = endpoint.method.to_lowercase();
                    let path = &endpoint.path;
                    let middlewares =  endpoint.middlewares.clone().unwrap_or_default().iter().map(|f| f.to_lowercase().replace("middleware", "_middleware").to_string()).map(|f| format!(".layer(axum::middleware::from_fn_with_state(state.clone(), super::{}::{}))", &f, &f)).collect::<String>();
                    let handler = format!(
                        "{}{}",
                        method,
                        path.replace("/", "_")
                            .replace(":", "")
                            .replace("{", "by_")
                            .replace("}", "")
                    );
                    routes.push_str(&format!(
                        "    .route(\"{}\", {}({}){})\n",
                        path, method, handler, middlewares
                    ));
                }
            }
        }
        routes
    }

    fn generate_actix_router(&self) -> String {
        let mut routes = String::new();
        for model in &self.config.models {
            if let Some(endpoints) = &model.endpoints {
                for endpoint in endpoints {
                    let method = match endpoint.method.to_lowercase().as_str() {
                        "get" => "web::get",
                        "post" => "web::post",
                        "put" => "web::put",
                        "delete" => "web::delete",
                        _ => "web::route",
                    };
                    let path = &endpoint.path;
                    let handler = format!(
                        "{}{}",
                        endpoint.method.to_lowercase(),
                        path.replace("/", "_")
                            .replace(":", "")
                            .replace("{", "by_")
                            .replace("}", "")
                    );
                    routes.push_str(&format!(
                        "    .route(\"{}\", {}().to({}))\n",
                        path, method, handler
                    ));
                }
            }
        }
        routes
    }


    pub fn generate_middleware_content(&self, middleware: &Middleware, _db_type: &str) -> String {
        let model_lower = middleware.model.to_lowercase();
        match self.config.framework {
            Framework::Axum => format!(
                "use crate::adapters::http::http::AppState;\n\
                 use crate::domain::error::AppError;\n\
                 use axum::{{extract::{{State, Request}}, middleware::Next, response::Response}};\n\n\
                 pub async fn {}_middleware(\n\
                     State(state): State<AppState>,\n\
                     request: Request,\n\
                     next: Next,\n\
                 ) -> Result<Response, AppError> {{\n\
                     let service = state.{}_service;\n\
                     tracing::info!(\"Processing request for {} model\");\n\
                     let response = next.run(request).await;\n\
                     Ok(response)\n\
                 }}",
                model_lower, model_lower, middleware.model
            ),
            Framework::ActixWeb => format!(
                "use crate::adapters::http::http::AppState;
                use actix_web::{{
                        body::BoxBody,
                        dev::{{ ServiceRequest, ServiceResponse}},
                        middleware::Next,
                        web, Error,
                    }};
                 pub async fn {}_middleware(\n\
                     req: ServiceRequest,\n\
                     next: Next<BoxBody>,\n\
                 ) -> Result<ServiceResponse, Error> {{\n\
                     let state = req.app_data::<web::Data<AppState>>().unwrap();\n\
                     let service = state.{}_service.clone();\n\
                     tracing::info!(\"Processing request for {} model\");\n\
                     let res = next.call(req).await?;\n\
                     Ok(res)\n\
                 }}",
                model_lower, model_lower, middleware.model
            ),
        }
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
        match self.config.framework {
            Framework::Axum => {
                "use axum::{response::IntoResponse, http::StatusCode};\n\
                 use thiserror::Error;\n\n\
                 #[derive(Error, Debug)]\n\
                 pub enum AppError {\n\
                     #[error(\"Database error: {0}\")]\n\
                     Database(#[from] sqlx::Error),\n\
                     #[error(\"Not found: {0}\")]\n\
                     NotFound(String),\n\
                     #[error(\"Unauthorized: {0}\")]\n\
                     Unauthorized(String),\n\
                 }\n\n\
                 impl IntoResponse for AppError {\n\
                     fn into_response(self) -> axum::response::Response {\n\
                         match self {\n\
                             AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),\n\
                             AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),\n\
                             AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()).into_response(),\n\
                         }\n\
                     }\n\
                 }".to_string()
            }
            Framework::ActixWeb => {
                "use actix_web::{error::Error as ActixError, HttpResponse, http::StatusCode};\n\
                 use thiserror::Error;\n\n\
                 #[derive(Error, Debug)]\n\
                 pub enum AppError {\n\
                     #[error(\"Database error: {0}\")]\n\
                     Database(#[from] sqlx::Error),\n\
                     #[error(\"Not found: {0}\")]\n\
                     NotFound(String),\n\
                     #[error(\"Unauthorized: {0}\")]\n\
                     Unauthorized(String),\n\
                 }\n\n\
                 impl actix_web::error::ResponseError for AppError {\n\
                     fn error_response(&self) -> HttpResponse {\n\
                         match self {\n\
                             AppError::Database(_) => HttpResponse::InternalServerError().body(self.to_string()),\n\
                             AppError::NotFound(msg) => HttpResponse::NotFound().body(msg.clone()),\n\
                             AppError::Unauthorized(_) => HttpResponse::Unauthorized().body(self.to_string()),\n\
                         }\n\
                     }\n\
                 }".to_string()
            }
        }
    }

    pub fn generate_http_content(&self) -> String {
        match self.config.framework {
            Framework::Axum => self.generate_axum_http_content(),
            Framework::ActixWeb => self.generate_actix_http_content(),
        }
    }
    
    fn generate_axum_http_content(&self) -> String {
        // Your existing Axum implementation (slightly adjusted for clarity)
        let template = self;
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
        
        let new_params = template.config.models.iter()
            .map(|m| {
                let service_name = format!("{}_service", m.name.to_lowercase());
                format!("{}: services::{}Service<Sqlx{}Repository>", service_name, m.name, m.name)
            })
            .collect::<Vec<_>>()
            .join(", ");
        
        let state_fields = template.config.models.iter()
            .map(|m| {
                let service_name = format!("{}_service", m.name.to_lowercase());
                format!("        {}: Arc::new({}),", service_name, service_name)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        let handlers: Vec<String> = template.config.models.iter().flat_map(|model| {
            model.endpoints.clone().map(|endpoint| {
                endpoint.iter().map(|endpoint|
                    generate_handler(model, endpoint, Framework::Axum)
                ).collect::<Vec<_>>()
            })
        }).flatten().collect();
        let handlers = handlers.join("\n\n");
        
        let router = template.generate_axum_router();
        
        format!(
            r#"
    use std::{{net::SocketAddr, sync::Arc}};
    use anyhow::Context;
    use serde::*;
    use axum::{{routing::{{get, post}}, Router, http::StatusCode, extract::*}};
    use tokio::net;
    use tower_http::{{cors::CorsLayer, trace::TraceLayer}};
    use crate::infrastructure::*;
    use crate::domain::*;
    use crate::application::services;
    use crate::domain::error::AppError;
    
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
                .nest("/api", api_routes(state.clone()))
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
    
    fn api_routes(state: AppState) -> Router<AppState> {{
        Router::new()
    {router}
    }}
    
    async fn health_route() -> (StatusCode, &'static str) {{
        (StatusCode::OK, "OK")
    }}
            "#
        )
    }
    
    fn generate_actix_http_content(&self) -> String {
        let template = self;
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
        
        let new_params = template.config.models.iter()
            .map(|m| {
                let service_name = format!("{}_service", m.name.to_lowercase());
                format!("{}: services::{}Service<Sqlx{}Repository>", service_name, m.name, m.name)
            })
            .collect::<Vec<_>>()
            .join(", ");
        
        let state_fields = template.config.models.iter()
            .map(|m| {
                let service_name = format!("{}_service", m.name.to_lowercase());
                format!("        {}: {},", service_name, service_name)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let self_state_fields = template.config.models.iter()
            .map(|m| {
                let service_name = format!("{}_service", m.name.to_lowercase());
                format!("        {}: Arc::new(self.{}),", service_name, service_name)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        let handlers: Vec<String> = template.config.models.iter().flat_map(|model| {
            model.endpoints.clone().map(|endpoint| {
                endpoint.iter().map(|endpoint|
                    generate_handler(model, endpoint, Framework::ActixWeb)
                ).collect::<Vec<_>>()
            })
        }).flatten().collect();
        let handlers = handlers.join("\n\n");
        
        let router = template.generate_actix_router();
        
        format!(
            r#"
    use actix_web::{{web, App, HttpResponse}};
    use std::sync::Arc;
    use anyhow::Context;
    use crate::infrastructure::*;
    use crate::domain::*;
    use crate::application::services;
    use crate::domain::error::AppError;
    use serde::*;
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct HttpServerConfig {{
        pub port: String,
    }}
    
    {app_state}
    
    {handlers}
    
    pub struct HttpServer {{
    {new_params},
    config: HttpServerConfig
    }}
    
    impl HttpServer {{

        pub async fn new(
        {new_params},
        config: HttpServerConfig,
        ) -> anyhow::Result<Self> {{
            Ok(
            Self {{config, {state_fields}}} 
            )
        }}

        pub async fn run(self) -> anyhow::Result<()> {{
            let state = web::Data::new(AppState {{
    {self_state_fields}
            }});
    
            actix_web::HttpServer::new(move || {{
                App::new()
                    .app_data(state.clone())
                    .route("/health", web::get().to(health_route))
                    .service(
                        web::scope("/api")
    {router}
                    )
            }})
            .bind(format!("0.0.0.0:{{}}", self.config.port.parse::<u16>().unwrap_or(3000)))?
            .run()
            .await
            .context("received error from running server")?;
            Ok(())
        }}
    }}
    
    async fn health_route() -> impl actix_web::Responder {{
        HttpResponse::Ok().body("OK")
    }}
            "#
        )
    }
}