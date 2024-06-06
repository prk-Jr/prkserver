mod models;
use models::*;

mod endpoints;
use endpoints::*;

mod config;
use config::*;

mod template;
use template::*;

mod output;

fn main() {
    // let matches = Command::new("prkserver")
    //     .version("1.0")
    //     .about("Generates a backend project with Axum and SQLx based on a config file")
    //     .arg(Arg::new("config").required(true).index(1))
    //     .get_matches();

    // let config_path = matches.get_one::<&str>("config").unwrap();

    let config: Config = read_config("./config.toml").expect("Failed to read config.toml");

    match generate_project(&config) {
        Ok(_) => println!("Project '{}' created successfully.", config.project_name),
        Err(e) => eprintln!("Error creating project: {}", e),
    }
}
