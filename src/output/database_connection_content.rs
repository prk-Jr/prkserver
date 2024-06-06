use convert_case::{Case, Casing};

pub fn database_connection_content(database_type: &str) -> String {
    let pool_options = match database_type
        .to_case(Case::UpperCamel)
        .to_lowercase()
        .as_str()
    {
        "sqlite" => "Sqlite",
        "mysql" => "MySql",
        _ => "Pg",
    };

    format!(
        r#"
        use dotenvy::dotenv;
        use sqlx::*;
        use std::env;
        
        /// .
        ///
        /// # Panics
        ///
        /// Panics if fails to acquire a database connection
        ///
        /// # Errors
        ///
        /// This function will return an error if fails to acquire a pool
        pub async fn connect_to_database() -> Result<{}Pool, sqlx::Error> {{
            dotenv().ok();
            let database_url = &env::var("DATABASE_URL").expect("Env var unavailable");
        
            println!("databse {{database_url}}");
            {}Pool::connect(&database_url).await
        }}
        "#,
        pool_options, pool_options,
    )
}
