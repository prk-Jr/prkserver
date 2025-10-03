use crate::domain::models::config::Config;

pub fn main_content(config: &Config) -> String {
    // Generate repository and service initializations dynamically
    let mut repo_initializations = String::new();
    let mut service_initializations = String::new();
    let mut service_params = String::new();

    for model in &config.models {
        let model_lower = model.name.to_lowercase();
        let repo_name = format!("Sqlx{}Repository", model.name);
        let service_name = format!("{}Service", model.name);
        let repo_var = format!("{}_repo", model_lower);
        let service_var = format!("{}_service", model_lower);

        // Repository initialization
        repo_initializations.push_str(&format!(
            "    let {} = infrastructure::repositories::{}::new(database_connection.clone());\n",
            repo_var, repo_name
        ));

        // Service initialization
        service_initializations.push_str(&format!(
            "    let {} = application::services::{}::new({});\n",
            service_var, service_name, repo_var
        ));

        // Collect service parameters for HttpServer::new
        service_params.push_str(&format!("{}, ", service_var));
    }

    // Remove trailing comma and space from service_params
    let service_params = service_params.trim_end_matches(", ");

    // Return the formatted main.rs content
    format!(
        r#"
mod database_connection;
mod adapters;
mod domain;
mod application;
mod infrastructure;

#[tokio::main]
async fn main() {{ 
    tracing_subscriber::fmt::init();
    let database_connection = database_connection::connect_to_database().await.expect("Could not connect to database");
    let config = adapters::http::http::HttpServerConfig {{ port: "3000".into() }};
{repo_initializations}{service_initializations}
    let http_server = adapters::http::http::HttpServer::new({service_params}, config).await.expect("Failed to create HTTP server");
    http_server.run().await.expect("Failed to run HTTP server");
}}
        "#
    )
}
