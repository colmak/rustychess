use crate::error::ChessError;
use serde::{Serialize, Deserialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub file: u8, // a-h (0-7)
    pub rank: u8, // 1-8 (0-7)
}

impl Position {
    // Create a new position without result wrapping - for internal use only
    // This assumes the coordinates are valid (0-7)
    pub fn new(file: u8, rank: u8) -> Self {
        Self { file, rank }
    }
    
    // Safe constructor that validates input
    pub fn create(file: u8, rank: u8) -> Result<Self, ChessError> {
        if file > 7 || rank > 7 {
            return Err(ChessError::InvalidPosition(format!(
                "File and rank must be between 0-7, got {},{}", file, rank
            )));
        }
        
        Ok(Self { file, rank })
    }
    
    // Create position from algebraic notation coordinates
    pub fn from_algebraic(file_char: char, rank_char: char) -> Result<Self, ChessError> {
        if !('a'..='h').contains(&file_char) || !('1'..='8').contains(&rank_char) {
            return Err(ChessError::InvalidPosition(format!(
                "Invalid algebraic coordinates: {}{}", file_char, rank_char
            )));
        }
        
        let file = (file_char as u8) - b'a';
        let rank = (rank_char as u8) - b'1';
        
        Ok(Self { file, rank })
    }
    
    // Convert to algebraic notation coordinates
    pub fn to_algebraic(&self) -> (char, char) {
        let file_char = (b'a' + self.file) as char;
        let rank_char = (b'1' + self.rank) as char;
        (file_char, rank_char)
    }
    
    // Check if position is valid on a chess board
    pub fn is_valid(&self) -> bool {
        self.file <= 7 && self.rank <= 7
    }
    
    // Get the relative rank from a given color's perspective
    // This helps with move generation where we need to know if a piece is on its starting rank
    pub fn relative_rank(&self, color: crate::chess::Color) -> u8 {
        match color {
            crate::chess::Color::White => self.rank,
            crate::chess::Color::Black => 7 - self.rank,
        }
    }
    
    // Check if the position is on a given relative rank from a color's perspective
    pub fn is_on_relative_rank(&self, rank: u8, color: crate::chess::Color) -> bool {
        self.relative_rank(color) == rank
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (file_char, rank_char) = self.to_algebraic();
        write!(f, "{}{}", file_char, rank_char)
    }
}

impl FromStr for Position {
    type Err = ChessError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(ChessError::InvalidPosition(
                format!("Position must be 2 characters, got {}", s)
            ));
        }
        
        let mut chars = s.chars();
        let file_char = chars.next().unwrap();
        let rank_char = chars.next().unwrap();
        
        Self::from_algebraic(file_char, rank_char)
    }
}
