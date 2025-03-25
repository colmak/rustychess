use crate::chess::{Board, Position, Color, Piece, ChessMove};
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
    pub engine: ChessEngine,
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            current_turn: Color::White,
            status: GameStatus::InProgress,
            move_history: Vec::new(),
            engine: ChessEngine::new(),
        }
    }
    
    pub fn make_move(&mut self, chess_move: ChessMove) -> Result<(), ChessError> {
        // Validate that it's the correct player's turn
        let current_piece = self.board.get_piece(&chess_move.from)
            .ok_or(ChessError::InvalidMove("No piece at source position".into()))?;
            
        if current_piece.color != self.current_turn {
            return Err(ChessError::InvalidMove("Not your turn".into()));
        }

        // Get all legal moves and verify this is one of them
        let legal_moves = self.engine.get_legal_moves(&self.board, self.current_turn)?;
        if !legal_moves.contains(&chess_move) {
            return Err(ChessError::InvalidMove("Illegal move".into()));
        }

        // Make the move
        self.board.move_piece(chess_move)?;
        
        // Switch turns
        self.current_turn = match self.current_turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        Ok(())
    }

    pub fn get_best_move(&self) -> Result<ChessMove, ChessError> {
        self.engine.get_best_move(&self.board, self.current_turn)
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
                    if piece.color == color && piece.is_king() {
                        king_pos = Some(pos);
                        break;
                    }
                }
            }
        }
        
        // If we somehow can't find the king, consider it in check
        let king_pos = match king_pos {
            Some(pos) => pos,
            None => return true,
        };
        
        // Check if the king's position is under attack
        self.board.is_square_attacked(&king_pos, color.opposite())
    }
    
    // Update the game status (check, checkmate, stalemate, etc.)
    fn update_game_status(&mut self) {
        let next_player = self.current_turn.opposite();
        
        if self.is_king_in_check(next_player) {
            // Check if it's checkmate by verifying if any move can get out of check
            if self.has_legal_moves(next_player) {
                self.status = GameStatus::Check;
            } else {
                self.status = GameStatus::Checkmate;
            }
        } else if !self.has_legal_moves(next_player) {
            self.status = GameStatus::Stalemate;
        } else {
            self.status = GameStatus::InProgress;
        }
    }
    
    // Check if the given color has any legal moves
    fn has_legal_moves(&self, color: Color) -> bool {
        // Try all possible moves for all pieces of the given color
        for from_rank in 0..8 {
            for from_file in 0..8 {
                let from = Position::new(from_file, from_rank);
                if let Some(piece) = self.board.get_piece(&from) {
                    if piece.color == color {
                        // Try moving to every square
                        for to_rank in 0..8 {
                            for to_file in 0..8 {
                                let to = Position::new(to_file, to_rank);
                                // Clone the game and try the move
                                let mut test_game = self.clone();
                                if test_game.make_move(ChessMove::new(from, to)).is_ok() {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }
}
