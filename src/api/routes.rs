use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;

use crate::chess::{Game, Engine, ChessMove};
use crate::error::ChessError;

// Simple in-memory game storage
// In a real app, you'd use a database
pub struct AppState {
    pub games: Mutex<HashMap<String, Game>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize)]
struct GameResponse {
    id: String,
    game: Game,
}

#[derive(Deserialize)]
struct MoveRequest {
    from: String,
    to: String,
}

#[derive(Serialize)]
struct BestMoveResponse {
    from: String,
    to: String,
    evaluation: i32,
    nodes_searched: u32,
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[post("/games")]
async fn new_game(data: web::Data<AppState>) -> impl Responder {
    let game = Game::new();
    let game_id = Uuid::new_v4().to_string();
    
    // Store the game
    let mut games = data.games.lock().unwrap();
    games.insert(game_id.clone(), game.clone());
    
    HttpResponse::Created().json(GameResponse {
        id: game_id,
        game,
    })
}

#[post("/games/{id}/moves")]
async fn make_move(
    game_id: web::Path<String>,
    move_req: web::Json<MoveRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let game_id_str = game_id.into_inner(); // Extract String from Path
    let mut games = data.games.lock().unwrap();
    
    // Find game
    let game = match games.get_mut(&game_id_str) {
        Some(game) => game,
        None => return HttpResponse::NotFound().body("Game not found"),
    };
    
    // Make move
    match game.make_move(&move_req.from, &move_req.to) {
        Ok(_) => HttpResponse::Ok().json(game),
        Err(e) => match e {
            ChessError::InvalidMove(msg) => HttpResponse::BadRequest().body(msg),
            _ => HttpResponse::InternalServerError().body("Internal server error"),
        },
    }
}

#[get("/games/{id}")]
async fn get_game(game_id: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let game_id_str = game_id.into_inner(); // Extract String from Path
    let games = data.games.lock().unwrap();
    
    // Find game
    match games.get(&game_id_str) {
        Some(game) => HttpResponse::Ok().json(game),
        None => HttpResponse::NotFound().body("Game not found"),
    }
}

#[get("/games/{id}/best-move")]
async fn get_best_move(game_id: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let game_id_str = game_id.into_inner();
    let games = data.games.lock().unwrap();
    
    // Find game with improved error message
    let game = match games.get(&game_id_str) {
        Some(game) => game,
        None => return HttpResponse::NotFound().json(json!({
            "error": "Game not found",
            "details": format!("No active game with ID: {}", game_id_str)
        })),
    };
    
    // Initialize engine with a search depth of 3
    let mut engine = Engine::new(3);
    
    // Find the best move with improved error handling
    match engine.find_best_move(game) {
        Ok(best_move) => {
            let (nodes_searched, _) = engine.get_stats();
            let response = BestMoveResponse {
                from: best_move.from.to_string(),
                to: best_move.to.to_string(),
                evaluation: best_move.score,
                nodes_searched,
            };
            
            // Print debug information
            println!("Best move found: {} -> {} (score: {})", 
                    best_move.from, best_move.to, best_move.score);
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            // Print the error for debugging
            println!("Error finding best move: {:?}", e);
            match e {
                ChessError::InvalidMove(msg) => HttpResponse::BadRequest().json(json!({
                    "error": "Invalid move",
                    "details": msg
                })),
                _ => HttpResponse::InternalServerError().json(json!({
                    "error": "Engine error",
                    "details": format!("Failed to compute best move: {:?}", e)
                })),
            }
        },
    }
}

// Configure routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(health_check)
            .service(new_game)
            .service(make_move)
            .service(get_game)
            .service(get_best_move)
    );
}
