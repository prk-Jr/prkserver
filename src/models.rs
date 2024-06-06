use std::fs;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
}

#[derive(Deserialize)]
pub struct Model {
    pub name: String,
    pub fields: Vec<Field>,
}

pub fn generate_model(project_name: &str, model: &Model) -> std::io::Result<()> {
    let model_dir = format!("./{}/src/models", project_name);
    let model_path = format!("{}/{}.rs", model_dir, model.name.to_lowercase());
    let mut model_content = String::new();
    model_content.push_str(&format!("pub struct {} {{\n", model.name));
    for field in &model.fields {
        model_content.push_str(&format!("    pub {}: {},\n", field.name, field.field_type));
    }
    model_content.push_str("}\n");
    fs::create_dir_all(model_dir).expect("Failed to create model dir");
    fs::write(model_path, model_content)?;
    Ok(())
}
