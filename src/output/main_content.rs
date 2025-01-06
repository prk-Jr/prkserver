pub fn main_content(router: String, extra_imports: String) -> String {
    format!(
        r#"
    use axum::Router;

    use axum::routing::*;
    
    mod database_connection;
    use database_connection::*;
    
    mod routes;
    use routes::*;

    mod models;

    {}
    
    
    #[tokio::main]
    async fn main() {{
        // A minimal tracing middleware for request logging.
        tracing_subscriber::fmt::init();
        // connect to the database
        let database_connection = connect_to_database()
        .await
        .expect("Could not connect to database");
        println!("Connected to the database without any error");
        
        
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        println!("Server running on port 3000");
        // Trace layer
         let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request<_>| {{
                let uri = request.uri().to_string();
                tracing::info_span!("http_request", method = ?request.method(), uri)
            }},
        );
        let app = {}.layer(trace_layer).with_state(database_connection);
        axum::serve(listener, app).await.unwrap();
    }}
        "#,
        extra_imports, router
    )
}
