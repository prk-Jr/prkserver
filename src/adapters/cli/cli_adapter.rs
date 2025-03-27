use crate::application::services::project_service::ProjectService;
use crate::domain::models::config::Config;
use crate::domain::ports::file_system::FileSystem;
use crate::domain::ports::project_generator::ProjectGenerator;

pub struct CliAdapter<F: FileSystem> {
    project_service: ProjectService<F>,
}

impl<F: FileSystem> CliAdapter<F> {
    pub fn new(project_service: ProjectService<F>) -> Self {
        Self { project_service }
    }

    pub async fn run(&self, config_path: &str) {
        let config_content = self.project_service.file_system.read_to_string(config_path)
            .await
            .expect("Failed to read config.toml");
        let config: Config = toml::from_str(&config_content).expect("Failed to parse config.toml");

        match self.project_service.generate_project(config).await {
            Ok(()) => println!(
                "Project '{}' created successfully.\n\ncd {}\ngit init\ncargo fmt",
                self.project_service.file_system.read_to_string(config_path).await.unwrap(), // Simplified for demo
                self.project_service.file_system.read_to_string(config_path).await.unwrap()
            ),
            Err(e) => eprintln!("Error creating project: {}", e),
        }
    }
}