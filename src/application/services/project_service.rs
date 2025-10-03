use crate::domain::models::config::Config;
use crate::domain::models::template::Template;
use crate::domain::ports::file_system::FileSystem;
use crate::domain::ports::project_generator::ProjectGenerator;
use crate::output::{
    cargo_toml_content, compose_yaml_content, database_connection_content, docker_ignore_content,
    dockerfile_content, env_content, git_ignore_content, main_content,
};
use std::error::Error;

pub struct ProjectService<F: FileSystem> {
    pub file_system: F,
}

impl<F: FileSystem> ProjectService<F> {
    pub fn new(file_system: F) -> Self {
        Self { file_system }
    }

    /// Creates a file at the specified path within the project directory.
    async fn create_file(
        &self,
        project_name: &str,
        file_path: &str,
        content: &str,
    ) -> Result<(), Box<dyn Error>> {
        // Construct the full path (e.g., "my_project/src/domain/mod.rs")
        let path = format!("{}/{}", project_name, file_path);

        // Get the parent directory (e.g., "my_project/src/domain")
        if let Some(parent) = std::path::Path::new(&path).parent() {
            // Create all necessary parent directories
            self.file_system
                .create_dir_all(parent.to_str().unwrap())
                .await?;
        }

        // Write the file to the path
        self.file_system.write_file(&path, content).await?;
        Ok(())
    }

    /// Generates the content for a `mod.rs` file by declaring public modules.
    fn generate_mod_rs_content(&self, file_names: &[String]) -> String {
        file_names
            .iter()
            .map(|name| format!("pub mod {};\npub use {}::*;\n\n", name, name))
            .collect()
    }

    /// Writes a `mod.rs` file to the specified directory with dynamic module declarations.
    async fn generate_mod_rs(
        &self,
        project_name: &str,
        dir: &str,
        file_names: &[String],
    ) -> Result<(), Box<dyn Error>> {
        let relative_path = format!("src/{}/mod.rs", dir);
        let content = self.generate_mod_rs_content(file_names);
        self.create_file(project_name, &relative_path, &content)
            .await?;
        Ok(())
    }
}

impl<F: FileSystem> ProjectGenerator for ProjectService<F> {
    /// Generates a complete Rust project based on the provided configuration.
    async fn generate_project(&self, config: Config) -> Result<(), Box<dyn Error>> {
        let template = Template::new(config.clone());

        // ### Create Directory Structure
        self.file_system
            .create_dir_all(&config.project_name)
            .await?;
        self.file_system
            .create_dir_all(&format!("{}/src", config.project_name))
            .await?;
        self.file_system
            .create_dir_all(&format!("{}/src/domain", config.project_name))
            .await?;
        self.file_system
            .create_dir_all(&format!("{}/src/domain/models", config.project_name))
            .await?;
        self.file_system
            .create_dir_all(&format!("{}/src/domain/ports", config.project_name))
            .await?;
        self.file_system
            .create_dir_all(&format!("{}/src/application", config.project_name))
            .await?;
        self.file_system
            .create_dir_all(&format!("{}/src/application/services", config.project_name))
            .await?;
        self.file_system
            .create_dir_all(&format!("{}/src/infrastructure", config.project_name))
            .await?;
        self.file_system
            .create_dir_all(&format!(
                "{}/src/infrastructure/repositories",
                config.project_name
            ))
            .await?;
        self.file_system
            .create_dir_all(&format!("{}/src/adapters", config.project_name))
            .await?;
        self.file_system
            .create_dir_all(&format!("{}/src/adapters/http", config.project_name))
            .await?;

        // ### Generate Static Files
        self.create_file(
            &config.project_name,
            "Cargo.toml",
            &cargo_toml_content(
                &config.project_name,
                &config.database_type,
                false,
                &config.framework,
            ),
        )
        .await?;
        self.create_file(&config.project_name, "src/main.rs", &main_content(&config))
            .await?;
        self.create_file(
            &config.project_name,
            "src/database_connection.rs",
            &database_connection_content(&config.database_type),
        )
        .await?;
        self.create_file(
            &config.project_name,
            ".env",
            &env_content(&config.database_url),
        )
        .await?;
        self.create_file(
            &config.project_name,
            "Dockerfile",
            &dockerfile_content(&config.project_name),
        )
        .await?;
        self.create_file(
            &config.project_name,
            ".dockerignore",
            &docker_ignore_content(),
        )
        .await?;
        self.create_file(&config.project_name, ".gitignore", &git_ignore_content())
            .await?;
        self.create_file(
            &config.project_name,
            "compose.yaml",
            &compose_yaml_content(&config.database_type),
        )
        .await?;

        // ### Generate `mod.rs` Files
        // **Adapters**
        self.generate_mod_rs(
            &config.project_name,
            "adapters/http",
            &vec!["http".to_string()],
        )
        .await?;

        // **Domain**
        self.generate_mod_rs(
            &config.project_name,
            "domain",
            &vec![
                "models".to_string(),
                "ports".to_string(),
                "error".to_string(),
            ],
        )
        .await?;

        // **Domain/Models**
        let model_files: Vec<String> = config
            .models
            .iter()
            .map(|m| m.name.to_lowercase())
            .collect();
        self.generate_mod_rs(&config.project_name, "domain/models", &model_files)
            .await?;

        // **Domain/Ports**
        let port_files: Vec<String> = config
            .models
            .iter()
            .map(|m| format!("{}_repository", m.name.to_lowercase()))
            .collect();
        self.generate_mod_rs(&config.project_name, "domain/ports", &port_files)
            .await?;

        // **Application**
        self.generate_mod_rs(
            &config.project_name,
            "application",
            &vec!["services".to_string()],
        )
        .await?;

        // **Application/Services**
        let service_files: Vec<String> = config
            .models
            .iter()
            .map(|m| format!("{}_service", m.name.to_lowercase()))
            .collect();
        self.generate_mod_rs(&config.project_name, "application/services", &service_files)
            .await?;

        // **Infrastructure**
        self.generate_mod_rs(
            &config.project_name,
            "infrastructure",
            &vec!["repositories".to_string()],
        )
        .await?;

        // **Infrastructure/Repositories**
        let repo_files: Vec<String> = config
            .models
            .iter()
            .map(|m| format!("sqlx_{}_repository", m.name.to_lowercase()))
            .collect();
        self.generate_mod_rs(
            &config.project_name,
            "infrastructure/repositories",
            &repo_files,
        )
        .await?;

        // **Adapters/HTTP** (currently empty, assuming `http.rs` is the only file)
        let http_files: Vec<String> = vec!["http".to_string()];
        let middles_files: Vec<String> = config
            .middlewares
            .iter()
            .map(|m| {
                m.iter()
                    .map(|m| format!("{}_middleware", m.model.to_lowercase()))
            })
            .flatten()
            .collect();
        let http_files = [&http_files[..], &middles_files[..]].concat();
        self.generate_mod_rs(&config.project_name, "adapters/", &vec!["http".to_string()])
            .await?;
        self.generate_mod_rs(&config.project_name, "adapters/http", &http_files)
            .await?;

        // ### Generate Dynamic Files
        for model in &config.models {
            // **Model**
            let model_path = format!("src/domain/models/{}.rs", model.name.to_lowercase());
            self.create_file(
                &config.project_name,
                &model_path,
                &template.generate_model_content(model),
            )
            .await?;

            // **Repository Trait**
            let repo_trait_path = format!(
                "src/domain/ports/{}_repository.rs",
                model.name.to_lowercase()
            );
            self.create_file(
                &config.project_name,
                &repo_trait_path,
                &template.generate_repository_trait(model),
            )
            .await?;

            // **Repository Implementation**
            let repo_impl_path = format!(
                "src/infrastructure/repositories/sqlx_{}_repository.rs",
                model.name.to_lowercase()
            );
            self.create_file(
                &config.project_name,
                &repo_impl_path,
                &template.generate_repository_impl(model, &config.database_type),
            )
            .await?;

            // **Service**
            let service_path = format!(
                "src/application/services/{}_service.rs",
                model.name.to_lowercase()
            );
            self.create_file(
                &config.project_name,
                &service_path,
                &template.generate_service(model),
            )
            .await?;
        }

        // **HTTP Server File**
        self.create_file(
            &config.project_name,
            "src/adapters/http/http.rs",
            &template.generate_http_content(),
        )
        .await?;

        // ### Generate Middleware (if present)
        if let Some(middlewares) = &config.middlewares {
            for middleware in middlewares {
                let middleware_path = format!(
                    "src/adapters/http/{}_middleware.rs",
                    middleware.model.to_lowercase()
                );
                self.create_file(
                    &config.project_name,
                    &middleware_path,
                    &template.generate_middleware_content(middleware, &config.database_type),
                )
                .await?;
            }
        }

        // ### Generate Error Handling
        self.create_file(
            &config.project_name,
            "src/domain/error.rs",
            &template.generate_error_content(),
        )
        .await?;

        Ok(())
    }
}
