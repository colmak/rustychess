use crate::chess::{Board, Position, Color, Engine, ChessMove, PieceType};
use crate::error::ChessError;
use serde::{Serialize, Deserialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    InProgress,
    Check,
    Checkmate,
    Stalemate,
    Draw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub board: Board,
    pub current_turn: Color,
    pub status: GameStatus,
    pub move_history: Vec<String>,
    pub engine: Engine,
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            current_turn: Color::White,
            status: GameStatus::InProgress,
            move_history: Vec::new(),
            engine: Engine::new(3),  // Default depth of 3
        }
    }
    
    pub fn make_move(&mut self, from_str: &str, to_str: &str) -> Result<(), ChessError> {
        // Parse positions from strings
        let from = Position::from_str(from_str)?;
        let to = Position::from_str(to_str)?;
        let chess_move = ChessMove::new(from, to);
        
        // Validate that it's the correct player's turn
        let current_piece = self.board.get_piece(&chess_move.from)
            .ok_or(ChessError::InvalidMove("No piece at source position".into()))?;
            
        if current_piece.color != self.current_turn {
            return Err(ChessError::InvalidMove("Not your turn".into()));
        }

        // Make the move on the board
        self.board.make_move(&chess_move.from, &chess_move.to)?;
        
        // Record the move
        self.move_history.push(format!("{}-{}", from_str, to_str));
        
        // Switch turns
        self.current_turn = self.current_turn.opposite();
        
        // Update game status
        self.update_game_status();

        Ok(())
    }

    pub fn get_best_move(&self) -> Result<ChessMove, ChessError> {
        let mut engine = Engine::new(3);
        engine.find_best_move(self)
    }
    
    pub fn get_status(&self) -> GameStatus {
        self.status.clone()
    }
    
    // Check if the king of the given color is in check
    fn is_king_in_check(&self, color: Color) -> bool {
        // Find the king's position
        let mut king_pos = None;
        for rank in 0..8 {
            for file in 0..8 {
                let pos = Position::new(file, rank);
                if let Some(piece) = self.board.get_piece(&pos) {
                    if piece.color == color && piece.piece_type == PieceType::King {
                        king_pos = Some(pos);
                        break;
                    }
                }
            }
            if king_pos.is_some() {
                break;
            }
        }
        
        // If we somehow can't find the king, consider it in check
        let king_pos = match king_pos {
            Some(pos) => pos,
            None => return true,
        };
        
        // Check if any opponent piece can capture the king
        // This is a simplified approach - just see if any legal move for the opponent
        // can land on the king's position
        let mut engine = Engine::new(1);  // Shallow depth for finding attacks
        let opponent_color = color.opposite();
        
        if let Ok(moves) = engine.generate_moves(&self.board, opponent_color) {
            for chess_move in moves {
                if chess_move.to == king_pos {
                    return true;
                }
            }
        }
        
        false
    }
    
    // Update the game status (check, checkmate, stalemate, etc.)
    fn update_game_status(&mut self) {
        let current_player = self.current_turn;
        
        // Check if the current player is in check
        let in_check = self.is_king_in_check(current_player);
        
        // Create a temporary engine to check for legal moves
        let mut engine = Engine::new(1);
        let has_legal_moves = match engine.generate_moves(&self.board, current_player) {
            Ok(moves) => !moves.is_empty(),
            Err(_) => false,
        };
        
        // Update status based on check status and available moves
        if in_check {
            if has_legal_moves {
                self.status = GameStatus::Check;
            } else {
                self.status = GameStatus::Checkmate;
            }
        } else if !has_legal_moves {
            self.status = GameStatus::Stalemate;
        } else {
            self.status = GameStatus::InProgress;
        }
    }
}
