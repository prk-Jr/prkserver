pub fn main_content(router: String) -> String {
    format!(
        r#"
    use axum::Router;

    use axum::routing::*;
    
    mod database_connection;
    use database_connection::*;
    
    mod routes;
    use routes::*;

    mod models;
    
    
    #[tokio::main]
    async fn main() {{
    
        // connect to the database
        let database_connection = connect_to_database()
        .await
        .expect("Could not connect to database");
        println!("Connected to the database without any error");;
    
        let app = {}.with_state(database_connection);
    
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }}
        "#,
        router
    )
}
