use convert_case::{Case, Casing};

pub fn compose_yaml_content(database_type: &str) -> String {
    let pool_options = database_type.to_case(Case::Upper);

    format!(
        r#"
services:
  server:
    build:
      context: .
      target: final
    ports:
      - 80:80
    depends_on:
      - db_image
    networks:
      - common-net

  db_image:
    image: {}:latest
    environment:
      {}_PORT: 3306
      {}_DATABASE: database_name
      {}_USER: user
      {}_PASSWORD: database_password
      {}_ROOT_PASSWORD: strong_database_password
    expose:
      - 3306
    ports:
      - "3307:3306"
    networks:
      - common-net

networks:
  common-net: {{}}

        "#,
        pool_options.to_lowercase(),
        pool_options,
        pool_options,
        pool_options,
        pool_options,
        pool_options,
    )
}
