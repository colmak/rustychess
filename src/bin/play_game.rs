use rustychess::chess::{Board, Position, Color, Game, Engine};
use rustychess::error::ChessError;
use std::str::FromStr;
use std::io::{self, Write};

fn main() -> Result<(), ChessError> {
    println!("RustyChess Engine Interactive Test");
    println!("==================================");
    
    // Create a new game
    let mut game = Game::new();
    
    // Print the initial board
    println!("Initial board:");
    println!("{}", game.board.debug_print());
    
    // Game loop
    loop {
        println!("\n{}'s turn", game.current_turn);
        
        if game.current_turn == Color::White {
            // Human player's turn (White)
            let mut input = String::new();
            print!("Enter your move (e.g., 'e2-e4') or 'quit' to exit: ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut input).unwrap();
            
            let input = input.trim();
            if input == "quit" {
                break;
            }
            
            // Split by hyphen or just parse the 4 characters
            let parts: Vec<&str> = if input.contains('-') {
                input.split('-').collect()
            } else if input.len() == 4 {
                vec![&input[0..2], &input[2..4]]
            } else {
                println!("Invalid input format. Please use format 'e2-e4' or 'e2e4'.");
                continue;
            };
            
            if parts.len() != 2 {
                println!("Invalid input format. Please use format 'e2-e4' or 'e2e4'.");
                continue;
            }
            
            // Make the move
            match game.make_move(parts[0], parts[1]) {
                Ok(_) => {
                    println!("Move made: {} -> {}", parts[0], parts[1]);
                    println!("{}", game.board.debug_print());
                },
                Err(e) => {
                    println!("Invalid move: {:?}", e);
                    continue;
                }
            }
        } else {
            // Engine's turn (Black)
            println!("Engine is thinking...");
            
            // Create an engine with depth 3
            let mut engine = Engine::new(3);
            
            // Find the best move
            match engine.find_best_move(&game) {
                Ok(best_move) => {
                    let (nodes, depth) = engine.get_stats();
                    println!("Engine's move: {} -> {} (score: {}, nodes: {}, depth: {})", 
                             best_move.from, best_move.to, best_move.score, nodes, depth);
                    
                    // Apply the move to the game
                    game.make_move(&best_move.from.to_string(), &best_move.to.to_string())?;
                    println!("{}", game.board.debug_print());
                },
                Err(e) => {
                    println!("Engine error: {:?}", e);
                    break;
                }
            }
        }
        
        // Check if the game is over
        match game.status {
            rustychess::chess::GameStatus::Checkmate => {
                println!("Checkmate! {} wins.", 
                         if game.current_turn == Color::White { "Black" } else { "White" });
                break;
            },
            rustychess::chess::GameStatus::Stalemate => {
                println!("Stalemate! The game is a draw.");
                break;
            },
            rustychess::chess::GameStatus::Draw => {
                println!("Draw!");
                break;
            },
            rustychess::chess::GameStatus::Check => {
                println!("{} is in check!", game.current_turn);
            },
            _ => { /* Game continues */ }
        }
    }
    
    Ok(())
}