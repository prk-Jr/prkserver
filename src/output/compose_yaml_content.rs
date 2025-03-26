pub fn compose_yaml_content(database_type: &str) -> String {
    match database_type.to_lowercase().as_str() {
        "mysql" => r#"
version: '3.8'
services:
  db:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: root
      MYSQL_DATABASE: test_db
    ports:
      - "3306:3306"
"#.to_string(),
        _ => r#"
version: '3.8'
services:
  db:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: root
      POSTGRES_DB: test_db
    ports:
      - "5432:5432"
"#.to_string(),
    }
}