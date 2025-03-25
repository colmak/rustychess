use actix_web::{App, HttpServer, web};
use actix_files as fs;
use actix_cors::Cors;
use rustychess::api;
use log::info;
use std::io;
use std::sync::Mutex;
use std::collections::HashMap;

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    info!("Starting RustyChess engine API on http://0.0.0.0:8080");
    
    // Create app state for storing games
    let app_state = web::Data::new(api::AppState {
        games: Mutex::new(HashMap::new()),
    });
    
    HttpServer::new(move || {
        // Configure CORS middleware
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .configure(api::config)
            // Serve static files from the 'static' directory
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
