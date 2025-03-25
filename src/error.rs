use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChessError {
    #[error("Invalid move: {0}")]
    InvalidMove(String),
    
    #[error("Invalid position: {0}")]
    InvalidPosition(String),
    
    #[error("Game over: {0}")]
    GameOver(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
