mod routes;

pub use routes::AppState;  // Re-export AppState to make it publicly accessible
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(routes::health_check)
            .service(routes::new_game)
            .service(routes::make_move)
            .service(routes::get_game)
    );
}
