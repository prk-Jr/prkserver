use crate::output::cargo_toml_content::cargo_toml_content;
use crate::output::compose_yaml_content::compose_yaml_content;
use crate::output::database_connection_content::*;
use crate::output::docker_ignore_content::docker_ignore_conent;
use crate::output::dockerfile_content::dockerfile_content;
use crate::output::env_content::env_content;
use crate::output::gitignore_content::git_ignore_conent;
use crate::output::main_content::main_content;
use crate::{generate_endpoint, generate_middleware, generate_model, Config};
use convert_case::Casing;
use std::fs;

fn create_file(project_name: &str, file_path: &str, content: &str) -> std::io::Result<()> {
    let path = format!("{}/{}", project_name, file_path);
    fs::write(path, content)
}

pub fn generate_project(config: &Config) -> std::io::Result<()> {
    fs::create_dir_all(&config.project_name).expect("Failed to create dir");
    fs::create_dir_all(format!("{}/src", &config.project_name)).expect("Failed to create dir");
    create_file(
        &config.project_name,
        "Cargo.toml",
        &cargo_toml_content(
            &config.project_name,
            &config.database_type,
            false, /* config.authorization */
        ),
    )?;
    let middleware = match &config.middlewares {
        Some(_) => format!("pub mod middlewares;\npub use middlewares::*;\n"),
        None => String::new(),
    };
    // if config.authorization {
    //     middleware.push_str("pub mod authorization;\npub use authorization::*;\n")
    // }
    create_file(
        &config.project_name,
        "src/main.rs",
        &main_content(create_router(&config), middleware),
    )?;
    create_file(
        &config.project_name,
        "src/database_connection.rs",
        &database_connection_content(&config.database_type),
    )?;
    create_file(
        &config.project_name,
        ".env",
        &env_content(&config.database_url),
    )?;
    create_file(
        &config.project_name,
        "Dockerfile",
        &dockerfile_content(&config.project_name),
    )?;
    create_file(
        &config.project_name,
        ".dockerignore",
        &docker_ignore_conent(),
    )?;
    create_file(&config.project_name, ".gitignore", &git_ignore_conent())?;
    create_file(
        &config.project_name,
        "compose.yaml",
        &compose_yaml_content(&config.database_type),
    )?;

    // extract_template_files(&config.project_name).expect("Failed to extract template files");
    modify_files(&config.project_name, config).expect("Failed to modify files");
    Ok(())
}

pub fn create_router(config: &Config) -> String {
    let mut router = String::from("Router::new()\n");

    for models in &config.models {
        if let Some(endpoints) = &models.endpoints {
            for endpoint in endpoints {
                let functions1 = format!(
                    "get({}{})",
                    "get_all",
                    &endpoint
                        .path
                        .to_lowercase()
                        .replace("/", "_")
                        .replace(":", ""),
                );
                let functions2 = format!(
                    "{}({}{})",
                    &endpoint.method.to_lowercase(),
                    endpoint.method.to_lowercase(),
                    &endpoint
                        .path
                        .to_lowercase()
                        .replace("/", "_")
                        .replace(":", ""),
                );
                if endpoint.method.to_lowercase() == "get" {
                    router.push_str(&format!(
                        "    .route(\"{}/all\",{})\n",
                        endpoint.path, functions1
                    ));
                }
                // if config.authorization {
                //     router.push_str(&format!(
                //         "    .route(\"{}\",{})\n\t",
                //         "/auth", "post(authorize)"
                //     ));
                // }
                router.push_str(&format!(
                    "    .route(\"{}\",{})\n\t",
                    endpoint.path, functions2
                ));
            }
        }
    }
    router
}

pub fn modify_files(project_name: &str, config: &Config) -> std::io::Result<()> {
    // Generate models and endpoints
    for model in &config.models {
        generate_model(project_name, model).expect("Failed to generate model");
    }

    let models_names: Vec<&str> = config.models.iter().map(|m| m.name.as_str()).collect();
    // if config.authorization {
    //     generate_module(
    //         project_name,
    //         "/src/authorization",
    //         vec!["auth", "claims", "init"],
    //     )
    //     .expect("Failed to generate module");
    // }
    if config.models.len() > 0 {
        generate_module(project_name, "/src/models", models_names.clone())
            .expect("Failed to generate module");
    }

    let mut endpoint_files: Vec<String> = vec![];
    let mut example_endpoint_files: Vec<String> = vec![];
    for model in &config.models {
        if let Some(endpoints) = &model.endpoints {
            for endpoint in endpoints {
                let endpoint_type = endpoint.method.clone();
                let path = endpoint.path.clone();
                let file = format!(
                    "{}{}",
                    endpoint_type,
                    path.replace("/", "_").replace(":", ""),
                );
                endpoint_files.push(file.clone());
                if endpoint.method.to_lowercase() == "get" {
                    example_endpoint_files.push(file);
                }
            }
        }
    }

    let endpoint_files: Vec<&str> = endpoint_files.iter().map(|f| f.as_str()).collect();
    let example_endpoint_files: Vec<&str> =
        example_endpoint_files.iter().map(|f| f.as_str()).collect();

    for model in &config.models {
        if let Some(endpoints) = &model.endpoints {
            for endpoint in endpoints {
                generate_endpoint(
                    project_name,
                    &endpoint,
                    &config.database_type,
                    model.name.as_str(),
                    false,
                )
                .expect("Failed to generate endpoint");
                if endpoint.method.to_lowercase() == "get" {
                    generate_endpoint(
                        project_name,
                        &endpoint,
                        &config.database_type,
                        model.name.as_str(),
                        true,
                    )
                    .expect("Failed to generate endpoint");
                }
            }
        }
    }

    if config
        .models
        .iter()
        .any(|e| e.endpoints.is_some() && e.endpoints.as_ref().unwrap().len() > 0)
    {
        let mut ep = endpoint_files.clone();
        if example_endpoint_files.len() > 0 {
            ep.push("example");
        }
        generate_module(project_name, "/src/routes", ep).expect("Failed to generate module");
    }

    if example_endpoint_files.len() > 0 {
        generate_module(
            project_name,
            "/src/routes/example",
            example_endpoint_files.clone(),
        )
        .expect("Failed to generate module");
    }

    if let Some(middlewares) = &config.middlewares {
        for middleware in middlewares {
            generate_middleware(project_name, &config.database_type, middleware)
                .expect("Failed to generate middlewre");
        }
        let mut middleware_names: Vec<String> = vec![];
        for middleware in middlewares {
            let file = format!("{}_middleware", middleware.model,);
            middleware_names.push(file);
        }
        let middleware_names: Vec<&str> = middleware_names.iter().map(|f| f.as_str()).collect();

        if config.models.len() > 0 {
            generate_module(project_name, "/src/middlewares", middleware_names.clone())
                .expect("Failed to generate module");
        }
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
