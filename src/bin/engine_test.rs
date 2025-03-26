use rustychess::chess::{Board, Position, Piece, PieceType, Color, Game, ChessMove, Engine};
use rustychess::error::ChessError;
use std::str::FromStr;

fn main() -> Result<(), ChessError> {
    println!("RustyChess Engine Test");
    println!("======================");
    
    // Create a new board with the initial position
    let mut board = Board::new();
    
    // Create an engine with depth 3
    let mut engine = Engine::new(3);
    
    // Print the initial board
    println!("Initial board:");
    println!("{}", board.debug_print());
    
    // Generate legal moves for white
    println!("Legal moves for White:");
    let moves = engine.generate_moves(&board, Color::White)?;
    for m in &moves {
        println!("  {} -> {}", m.from, m.to);
    }
    
    // Try a specific move: e2-e4
    let from = Position::from_str("e2")?;
    let to = Position::from_str("e4")?;
    println!("\nMaking move: {} -> {}", from, to);
    board.make_move(&from, &to)?;
    
    // Print the updated board
    println!("\nBoard after e2-e4:");
    println!("{}", board.debug_print());
    
    // Generate legal moves for black
    println!("\nLegal moves for Black:");
    let moves = engine.generate_moves(&board, Color::Black)?;
    for m in &moves {
        println!("  {} -> {}", m.from, m.to);
    }
    
    // Try finding the best move for black
    println!("\nFinding best move for Black...");
    
    // Create a new game with the updated board
    let mut game = Game::new();
    game.board = board.clone(); // Clone the board to keep our original copy
    game.current_turn = Color::Black;
    
    match engine.find_best_move(&game) {
        Ok(best_move) => {
            println!("Best move found: {} -> {} (score: {})", 
                     best_move.from, best_move.to, best_move.score);
            
            // Make the move on the board
            board.make_move(&best_move.from, &best_move.to)?;
            
            // Print the final board
            println!("\nFinal board after Black's best move:");
            println!("{}", board.debug_print());
        },
        Err(e) => {
            println!("Error finding best move: {:?}", e);
        }
    }
    
    Ok(())
}