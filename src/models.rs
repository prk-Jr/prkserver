use std::fs;

use convert_case::{Case, Casing};
use serde::Deserialize;

use crate::Endpoint;

#[derive(Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
}

#[derive(Deserialize)]
pub struct Model {
    pub name: String,
    pub table_name: String,
    pub fields: Vec<Field>,
    pub endpoints: Option<Vec<Endpoint>>,
}

pub fn generate_model(project_name: &str, model: &Model) -> std::io::Result<()> {
    let model_dir = format!("./{}/src/models", project_name);
    let model_path = format!(
        "{}/{}.rs",
        model_dir,
        model.name.to_case(Case::Snake).to_lowercase()
    );
    let mut model_content = String::new();
    model_content.push_str(&format!(
        "use prkorm::Table;\n
        use serde::{{Deserialize, Serialize}};\n
        use sqlx::FromRow;\n\n"
    ));
    model_content.push_str(&format!(
        "#[derive(Deserialize, Debug, Table, Serialize, FromRow, Default)]\n"
    ));
    model_content.push_str(&format!(
        "#[table_name(\"{}\")]\n",
        model.table_name.to_case(Case::Snake)
    ));
    model_content.push_str(&format!(
        "#[table_alias(\"{}\")]\n",
        get_alias_from_model(&model.name)
    ));
    model_content.push_str(&format!("pub struct {} {{\n", model.name));
    for field in &model.fields {
        model_content.push_str(&format!("    pub {}: {},\n", field.name, field.field_type));
    }
    model_content.push_str("}\n");
    fs::create_dir_all(model_dir).expect("Failed to create model dir");
    fs::write(model_path, model_content)?;
    Ok(())
}

fn get_alias_from_model(input_string: &String) -> String {
    let mut capital_characters = String::new();

    for c in input_string.chars() {
        if c.is_ascii_uppercase() {
            capital_characters.push(c);
        }
    }
    capital_characters
}
