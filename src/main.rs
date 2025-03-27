mod domain;
mod application;
mod infrastructure;
mod adapters;
mod output;

use adapters::cli::cli_adapter::CliAdapter;
use application::services::project_service::ProjectService;
use infrastructure::file_system::local_file_system::LocalFileSystem;

#[tokio::main]
async fn main() {
    let file_system = LocalFileSystem;
    let project_service = ProjectService::new(file_system);
    let cli_adapter = CliAdapter::new(project_service);

    cli_adapter.run("./config.toml").await;
}