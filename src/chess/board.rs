use crate::chess::{Piece, PieceType, Color, Position};
use crate::error::ChessError;
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    // Using Option to represent empty squares
    squares: [[Option<Piece>; 8]; 8],
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            squares: [[None; 8]; 8],
        };
        
        board.setup_initial_position();
        board
    }
    
    fn setup_initial_position(&mut self) {
        // Set up pawns
        for file in 0..8 {
            self.squares[1][file] = Some(Piece::new(PieceType::Pawn, Color::White));
            self.squares[6][file] = Some(Piece::new(PieceType::Pawn, Color::Black));
        }
        
        // Set up the rest of the pieces
        self.setup_back_rank(0, Color::White);
        self.setup_back_rank(7, Color::Black);
    }
    
    fn setup_back_rank(&mut self, rank: usize, color: Color) {
        self.squares[rank][0] = Some(Piece::new(PieceType::Rook, color));
        self.squares[rank][1] = Some(Piece::new(PieceType::Knight, color));
        self.squares[rank][2] = Some(Piece::new(PieceType::Bishop, color));
        self.squares[rank][3] = Some(Piece::new(PieceType::Queen, color));
        self.squares[rank][4] = Some(Piece::new(PieceType::King, color));
        self.squares[rank][5] = Some(Piece::new(PieceType::Bishop, color));
        self.squares[rank][6] = Some(Piece::new(PieceType::Knight, color));
        self.squares[rank][7] = Some(Piece::new(PieceType::Rook, color));
    }
    
    // Get a piece at a specific position
    pub fn get_piece(&self, pos: &Position) -> Option<Piece> {
        if !pos.is_valid() {
            return None;
        }
        
        self.squares[pos.rank as usize][pos.file as usize]
    }
    
    // Set a piece at a specific position
    pub fn set_piece(&mut self, pos: &Position, piece: Option<Piece>) -> Result<(), ChessError> {
        if !pos.is_valid() {
            return Err(ChessError::InvalidPosition(format!("Invalid position: {}", pos)));
        }
        
        self.squares[pos.rank as usize][pos.file as usize] = piece;
        Ok(())
    }
    
    // Make a move on the board
    pub fn make_move(&mut self, from: &Position, to: &Position) -> Result<(), ChessError> {
        // Validate positions
        if !from.is_valid() {
            return Err(ChessError::InvalidPosition(format!("Invalid from position: {}", from)));
        }
        if !to.is_valid() {
            return Err(ChessError::InvalidPosition(format!("Invalid to position: {}", to)));
        }
        
        // Check if there's a piece at the from position
        let piece = match self.get_piece(from) {
            Some(p) => p,
            None => return Err(ChessError::InvalidMove(format!("No piece at position {}", from))),
        };
        
        // Simple move logic (without validation)
        self.set_piece(from, None)?;
        self.set_piece(to, Some(piece))?;
        
        Ok(())
    }
    
    // Print the pieces on the board - useful for debugging
    pub fn debug_print(&self) -> String {
        let mut output = String::new();
        output.push_str("Board state:\n");
        
        for rank in (0..8).rev() {
            output.push_str(&format!("{}  ", rank + 1));
            for file in 0..8 {
                let piece = self.squares[rank][file];
                let symbol = match piece {
                    Some(p) => p.to_char(),
                    None => '.',
                };
                output.push_str(&format!("{} ", symbol));
            }
            output.push('\n');
        }
        output.push_str("   a b c d e f g h\n");
        
        output
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{}  ", rank + 1)?;
            for file in 0..8 {
                let piece = self.squares[rank][file];
                let symbol = match piece {
                    Some(p) => p.to_char(),
                    None => '.',
                };
                write!(f, "{} ", symbol)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "   a b c d e f g h")?;
        Ok(())
    }
}
