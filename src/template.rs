use crate::template_create::*;
use crate::{generate_endpoint, generate_model, Config};
use convert_case::Casing;
use include_dir::{include_dir, Dir};
use std::fs;

fn create_file(project_name: &str, file_path: &str, content: &str) -> std::io::Result<()> {
    let path = format!("{}/{}", project_name, file_path);
    fs::write(path, content)
}

pub fn generate_project(config: &Config) -> std::io::Result<()> {
    fs::create_dir_all(&config.project_name).expect("Failed to create dir");
    fs::create_dir_all(format!("{}/src", &config.project_name)).expect("Failed to create dir");
    fs::create_dir_all(format!("{}/config", &config.project_name)).expect("Failed to create dir");
    create_file(
        &config.project_name,
        "Cargo.toml",
        &cargo_toml_content(&config.project_name, &config.database_type),
    )?;
    create_file(
        &config.project_name,
        "src/main.rs",
        &main_rs_content(&config.database_type),
    )?;
    create_file(
        &config.project_name,
        "config/config.toml",
        &config_toml_content(&config.database_url),
    )?;

    // extract_template_files(&config.project_name).expect("Failed to extract template files");
    modify_files(&config.project_name, config).expect("Failed to modify files");
    Ok(())
}

// static TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/template/backend_template");

// pub fn extract_template_files(project_name: &str) -> std::io::Result<()> {
//     TEMPLATE_DIR.extract(project_name).map_err(|e| {
//         std::io::Error::new(
//             std::io::ErrorKind::Other,
//             format!("Failed to extract template files: {}", e),
//         )
//     })
// }

pub fn modify_files(project_name: &str, config: &Config) -> std::io::Result<()> {
    // Update database URL in config.toml
    // let config_path = format!("config.toml");
    // let config_content = fs::read_to_string(&config_path).expect("Failed to read config file");
    // let new_config_content = config_content.replace(
    //     "postgres://user:password@localhost/database_name",
    //     &config.database_url,
    // );
    // fs::write(config_path, new_config_content).expect("Failed to write config file");

    // Generate models and endpoints
    for model in &config.models {
        generate_model(project_name, model).expect("Failed to generate model");
    }

    let models_names: Vec<&str> = config.models.iter().map(|m| m.name.as_str()).collect();
    if config.models.len() > 0 {
        generate_module(project_name, "/src/models", models_names.clone())
            .expect("Failed to generate module");
    }

    for endpoint in &config.endpoints {
        generate_endpoint(project_name, endpoint).expect("Failed to generate endpoint");
    }
    if config.endpoints.len() > 0 {
        generate_module(project_name, "/src/routes", models_names)
            .expect("Failed to generate module");
    }
    // generate_module(project_name, "/src", vec!["models", "routes"])
    //     .expect("Failed to generate root module");

    Ok(())
}

pub fn generate_module(project_name: &str, dir_name: &str, name: Vec<&str>) -> std::io::Result<()> {
    let module_dir = format!("./{}{}", project_name, dir_name);
    let module_path = format!("{}/mod.rs", module_dir);
    let mut model_content = String::new();
    for field in &name {
        model_content.push_str(&format!(
            "pub mod {};\n",
            field.to_case(convert_case::Case::Snake)
        ));
    }
    for field in name {
        model_content.push_str(&format!(
            "pub use {}::*;\n",
            field.to_case(convert_case::Case::Snake)
        ));
    }
    fs::create_dir_all(module_dir).expect("Failed to create model dir");
    fs::write(module_path, model_content)?;
    Ok(())
}
