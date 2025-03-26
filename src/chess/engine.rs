use crate::chess::{Board, Position, Piece, PieceType, Color, Game};
use crate::error::ChessError;
use std::str::FromStr;
use std::cmp;
use serde::{Serialize, Deserialize};

// Point values for each piece type (traditional chess values)
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 20000; // Very high to ensure king safety

// Position evaluation bonus for controlling center, good pawn structure, etc.
const CENTER_CONTROL_BONUS: i32 = 10;
const DEVELOPED_PIECE_BONUS: i32 = 15;

// Directions for move generation
const DIRECTIONS: [(i32, i32); 8] = [
    (1, 0), (-1, 0), (0, 1), (0, -1), // Rook (and Queen)
    (1, 1), (1, -1), (-1, 1), (-1, -1), // Bishop (and Queen)
];

// Knight move patterns
const KNIGHT_MOVES: [(i32, i32); 8] = [
    (2, 1), (2, -1), (-2, 1), (-2, -1),
    (1, 2), (1, -2), (-1, 2), (-1, -2),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChessMove {
    pub from: Position,
    pub to: Position,
    pub score: i32, // Used for move ordering
}

impl ChessMove {
    pub fn new(from: Position, to: Position) -> Self {
        Self {
            from,
            to,
            score: 0,
        }
    }
    
    pub fn to_string(&self) -> String {
        format!("{}-{}", self.from, self.to)
    }
}

impl FromStr for ChessMove {
    type Err = ChessError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Format should be like "e2-e4" or "e2e4"
        let s = s.trim();
        
        // Handle both formats with or without separator
        let (from_str, to_str) = if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 2 {
                return Err(ChessError::InvalidMove(format!("Invalid move format: {}", s)));
            }
            (parts[0], parts[1])
        } else if s.len() == 4 {
            (&s[0..2], &s[2..4])
        } else {
            return Err(ChessError::InvalidMove(format!("Invalid move format: {}", s)));
        };
        
        // Parse the positions
        let from = Position::from_str(from_str)?;
        let to = Position::from_str(to_str)?;
        
        Ok(ChessMove { from, to, score: 0 })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engine {
    // Search depth for the minimax algorithm
    depth: u8,
    // Number of positions evaluated
    nodes_searched: u32,
    // Debug mode
    #[serde(default)]
    debug: bool,
}

impl Engine {
    pub fn new(depth: u8) -> Self {
        Self {
            depth,
            nodes_searched: 0,
            debug: false, // Turn off debug mode by default
        }
    }
    
    // Enable/disable debug mode
    pub fn set_debug_mode(&mut self, debug: bool) {
        self.debug = debug;
    }
    
    // Helper to print debug info
    fn debug_print(&self, msg: &str) {
        if self.debug {
            println!("[Engine Debug] {}", msg);
        }
    }
    
    // Find the best move in the current position
    pub fn find_best_move(&mut self, game: &Game) -> Result<ChessMove, ChessError> {
        let current_color = game.current_turn;
        self.debug_print(&format!("Finding best move for {:?}", current_color));
        self.debug_print(&format!("Current board state:\n{}", game.board.debug_print()));
        
        let mut best_move = None;
        let mut best_score = i32::MIN;
        self.nodes_searched = 0;
        
        // Generate all legal moves
        self.debug_print("About to generate legal moves");
        let moves = self.generate_moves(&game.board, current_color)?;
        
        if moves.is_empty() {
            self.debug_print(&format!("No legal moves found for {:?}", current_color));
            return Err(ChessError::InvalidMove("No legal moves available".to_string()));
        }
        
        self.debug_print(&format!("Generated {} legal moves:", moves.len()));
        for m in &moves {
            self.debug_print(&format!("  Move: {} -> {}", m.from, m.to));
        }
        
        // For each move, evaluate the resulting position
        for mut chess_move in moves {
            self.debug_print(&format!("Evaluating move: {} -> {}", chess_move.from, chess_move.to));
            
            // Create a copy of the board to simulate the move
            let mut board_copy = game.board.clone();
            
            match board_copy.make_move(&chess_move.from, &chess_move.to) {
                Ok(_) => {
                    // Evaluate with minimax
                    let score = -self.minimax(&board_copy, self.depth - 1, i32::MIN + 1, i32::MAX - 1, current_color.opposite());
                    
                    // Store the score in the move
                    chess_move.score = score;
                    
                    self.debug_print(&format!("Evaluated move {} -> {} with score: {}", 
                                             chess_move.from, chess_move.to, score));
                    
                    // Update best move if this is better
                    if score > best_score {
                        best_score = score;
                        best_move = Some(chess_move);
                    }
                },
                Err(e) => {
                    self.debug_print(&format!("Error applying move during evaluation: {:?}", e));
                    continue;
                }
            }
        }
        
        // Return the best move found
        match best_move {
            Some(mv) => {
                self.debug_print(&format!("Found best move: {} -> {} with score {}", 
                                         mv.from, mv.to, mv.score));
                Ok(mv)
            },
            None => {
                self.debug_print("No valid moves found after evaluation");
                Err(ChessError::InvalidMove("No valid moves found".to_string()))
            }
        }
    }
    
    // Minimax algorithm with alpha-beta pruning
    fn minimax(&mut self, board: &Board, depth: u8, mut alpha: i32, mut beta: i32, color: Color) -> i32 {
        self.nodes_searched += 1;
        
        // Base case: if we've reached the maximum depth, evaluate the board
        if depth == 0 {
            return self.evaluate_board(board, color);
        }
        
        // Generate legal moves for the current player
        let moves = match self.generate_moves(board, color) {
            Ok(m) => m,
            Err(e) => {
                self.debug_print(&format!("Error generating moves in minimax: {:?}", e));
                return self.evaluate_board(board, color); // If no moves, evaluate current position
            }
        };
        
        // If there are no legal moves, it's either checkmate or stalemate
        if moves.is_empty() {
            // This is a simplified evaluation - ideally check for checkmate vs stalemate
            return self.evaluate_board(board, color);
        }
        
        // Maximize or minimize based on the current player
        if color == Color::White {
            let mut max_score = i32::MIN;
            for chess_move in moves {
                // Create a copy of the board to simulate the move
                let mut board_copy = board.clone();
                if board_copy.make_move(&chess_move.from, &chess_move.to).is_err() {
                    continue;
                }
                
                // Recursively evaluate the position
                let score = self.minimax(&board_copy, depth - 1, alpha, beta, Color::Black);
                max_score = cmp::max(max_score, score);
                alpha = cmp::max(alpha, score);
                
                // Alpha-beta pruning
                if beta <= alpha {
                    break;
                }
            }
            max_score
        } else {
            let mut min_score = i32::MAX;
            for chess_move in moves {
                // Create a copy of the board to simulate the move
                let mut board_copy = board.clone();
                if board_copy.make_move(&chess_move.from, &chess_move.to).is_err() {
                    continue;
                }
                
                // Recursively evaluate the position
                let score = self.minimax(&board_copy, depth - 1, alpha, beta, Color::White);
                min_score = cmp::min(min_score, score);
                beta = cmp::min(beta, score);
                
                // Alpha-beta pruning
                if beta <= alpha {
                    break;
                }
            }
            min_score
        }
    }
    
    // Generate all legal moves for a given position and player
    // Making this public so it can be called from Game
    pub fn generate_moves(&self, board: &Board, color: Color) -> Result<Vec<ChessMove>, ChessError> {
        let mut moves = Vec::new();
        
        // Track what pieces we find for debugging
        let mut found_pieces = 0;
        
        // Loop through all squares on the board
        for rank in 0..8 {
            for file in 0..8 {
                // Create position with direct constructor since we're sure coordinates are valid
                let from = Position::new(file, rank);
                
                // Check if there's a piece at this position
                if let Some(piece) = board.get_piece(&from) {
                    // Only generate moves for pieces of the current player's color
                    if piece.color == color {
                        found_pieces += 1;
                        self.debug_print(&format!("Found {:?} {:?} at {}", piece.color, piece.piece_type, from));
                        
                        // Generate moves based on piece type
                        match piece.piece_type {
                            PieceType::Pawn => {
                                if let Err(e) = self.generate_pawn_moves(board, &from, piece, &mut moves) {
                                    self.debug_print(&format!("Error generating pawn moves: {:?}", e));
                                }
                            },
                            PieceType::Knight => {
                                if let Err(e) = self.generate_knight_moves(board, &from, piece, &mut moves) {
                                    self.debug_print(&format!("Error generating knight moves: {:?}", e));
                                }
                            },
                            PieceType::Bishop => {
                                if let Err(e) = self.generate_bishop_moves(board, &from, piece, &mut moves) {
                                    self.debug_print(&format!("Error generating bishop moves: {:?}", e));
                                }
                            },
                            PieceType::Rook => {
                                if let Err(e) = self.generate_rook_moves(board, &from, piece, &mut moves) {
                                    self.debug_print(&format!("Error generating rook moves: {:?}", e));
                                }
                            },
                            PieceType::Queen => {
                                if let Err(e) = self.generate_queen_moves(board, &from, piece, &mut moves) {
                                    self.debug_print(&format!("Error generating queen moves: {:?}", e));
                                }
                            },
                            PieceType::King => {
                                if let Err(e) = self.generate_king_moves(board, &from, piece, &mut moves) {
                                    self.debug_print(&format!("Error generating king moves: {:?}", e));
                                }
                            },
                        }
                    }
                }
            }
        }
        
        if found_pieces == 0 {
            self.debug_print(&format!("Warning: No pieces found for color {:?}", color));
        }
        
        self.debug_print(&format!("Generated {} moves for {:?}", moves.len(), color));
        Ok(moves)
    }
    
    // Generate moves for a pawn
    fn generate_pawn_moves(&self, board: &Board, from: &Position, piece: Piece, moves: &mut Vec<ChessMove>) -> Result<(), ChessError> {
        self.debug_print(&format!("Generating pawn moves from {} for {:?}", from, piece.color));
        
        let direction: i32 = if piece.color == Color::White { 1 } else { -1 };
        
        // Forward move - handle black's negative direction carefully
        let new_rank = (from.rank as i32) + direction;
        
        // Check if new rank is within bounds
        if new_rank >= 0 && new_rank < 8 {
            let to_rank = new_rank as u8;
            let to = Position::new(from.file, to_rank);
            
            self.debug_print(&format!("Checking forward move to {}", to));
            
            // Check if square is empty
            if board.get_piece(&to).is_none() {
                self.debug_print(&format!("Adding pawn move from {} to {}", from, to));
                moves.push(ChessMove::new(*from, to));
                
                // Double move from starting position
                if (piece.color == Color::White && from.rank == 1) || 
                   (piece.color == Color::Black && from.rank == 6) {
                    
                    let double_new_rank = (from.rank as i32) + 2 * direction;
                    
                    // Make sure double move rank is valid
                    if double_new_rank >= 0 && double_new_rank < 8 {
                        let double_to_rank = double_new_rank as u8;
                        let double_to = Position::new(from.file, double_to_rank);
                        
                        self.debug_print(&format!("Checking double move to {}", double_to));
                        
                        if board.get_piece(&double_to).is_none() {
                            self.debug_print(&format!("Adding pawn double move from {} to {}", from, double_to));
                            moves.push(ChessMove::new(*from, double_to));
                        }
                    }
                }
            }
        }
        
        // Captures
        if new_rank >= 0 && new_rank < 8 {
            let to_rank = new_rank as u8;
            
            for file_offset in [-1, 1].iter() {
                let new_file = (from.file as i32) + file_offset;
                
                if new_file >= 0 && new_file < 8 {
                    let to_file = new_file as u8;
                    let to = Position::new(to_file, to_rank);
                    
                    self.debug_print(&format!("Checking pawn capture to {}", to));
                    
                    if let Some(target) = board.get_piece(&to) {
                        if target.color != piece.color {
                            self.debug_print(&format!("Adding pawn capture from {} to {}", from, to));
                            moves.push(ChessMove::new(*from, to));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    // Generate moves for a knight
    fn generate_knight_moves(&self, board: &Board, from: &Position, piece: Piece, moves: &mut Vec<ChessMove>) -> Result<(), ChessError> {
        for &(dr, df) in &KNIGHT_MOVES {
            let to_rank = from.rank as i32 + dr;
            let to_file = from.file as i32 + df;
            
            // Check if the move is within the board
            if to_rank >= 0 && to_rank < 8 && to_file >= 0 && to_file < 8 {
                let to = Position::new(to_file as u8, to_rank as u8);
                
                // Check if the destination is empty or has an enemy piece
                if let Some(target) = board.get_piece(&to) {
                    if target.color != piece.color {
                        moves.push(ChessMove::new(*from, to));
                    }
                } else {
                    moves.push(ChessMove::new(*from, to));
                }
            }
        }
        
        Ok(())
    }
    
    // Generate moves for a bishop
    fn generate_bishop_moves(&self, board: &Board, from: &Position, piece: Piece, moves: &mut Vec<ChessMove>) -> Result<(), ChessError> {
        // Bishop moves along diagonals (4 directions)
        for &(dr, df) in &DIRECTIONS[4..8] {
            self.generate_sliding_moves(board, from, piece, dr, df, moves)?;
        }
        
        Ok(())
    }
    
    // Generate moves for a rook
    fn generate_rook_moves(&self, board: &Board, from: &Position, piece: Piece, moves: &mut Vec<ChessMove>) -> Result<(), ChessError> {
        // Rook moves along ranks and files (4 directions)
        for &(dr, df) in &DIRECTIONS[0..4] {
            self.generate_sliding_moves(board, from, piece, dr, df, moves)?;
        }
        
        Ok(())
    }
    
    // Generate moves for a queen
    fn generate_queen_moves(&self, board: &Board, from: &Position, piece: Piece, moves: &mut Vec<ChessMove>) -> Result<(), ChessError> {
        // Queen moves along ranks, files, and diagonals (8 directions)
        for &(dr, df) in &DIRECTIONS {
            self.generate_sliding_moves(board, from, piece, dr, df, moves)?;
        }
        
        Ok(())
    }
    
    // Generate moves for a king
    fn generate_king_moves(&self, board: &Board, from: &Position, piece: Piece, moves: &mut Vec<ChessMove>) -> Result<(), ChessError> {
        // King moves one square in any direction
        for &(dr, df) in &DIRECTIONS {
            let to_rank = from.rank as i32 + dr;
            let to_file = from.file as i32 + df;
            
            // Check if the move is within the board
            if to_rank >= 0 && to_rank < 8 && to_file >= 0 && to_file < 8 {
                let to = Position::new(to_file as u8, to_rank as u8);
                
                // Check if the destination is empty or has an enemy piece
                if let Some(target) = board.get_piece(&to) {
                    if target.color != piece.color {
                        moves.push(ChessMove::new(*from, to));
                    }
                } else {
                    moves.push(ChessMove::new(*from, to));
                }
            }
        }
        
        // TODO: Castling
        Ok(())
    }
    
    // Helper function for generating sliding moves (bishops, rooks, queens)
    fn generate_sliding_moves(&self, board: &Board, from: &Position, piece: Piece, dr: i32, df: i32, moves: &mut Vec<ChessMove>) -> Result<(), ChessError> {
        let mut to_rank = from.rank as i32 + dr;
        let mut to_file = from.file as i32 + df;
        
        // Continue in the given direction until we hit the edge of the board or another piece
        while to_rank >= 0 && to_rank < 8 && to_file >= 0 && to_file < 8 {
            let to = Position::new(to_file as u8, to_rank as u8);
            
            // Check if the destination has a piece
            if let Some(target) = board.get_piece(&to) {
                // If it's an enemy piece, we can capture it
                if target.color != piece.color {
                    moves.push(ChessMove::new(*from, to));
                }
                break; // Stop in this direction after hitting a piece
            } else {
                moves.push(ChessMove::new(*from, to));
            }
            
            // Continue in the same direction
            to_rank += dr;
            to_file += df;
        }
        
        Ok(())
    }
    
    // Evaluate the current board position
    fn evaluate_board(&self, board: &Board, color: Color) -> i32 {
        let mut score = 0;
        
        // Loop through all squares on the board
        for rank in 0..8 {
            for file in 0..8 {
                let pos = Position::new(file, rank);
                
                // Check if there's a piece at this position
                if let Some(piece) = board.get_piece(&pos) {
                    // Calculate the material value
                    let piece_value = match piece.piece_type {
                        PieceType::Pawn => PAWN_VALUE,
                        PieceType::Knight => KNIGHT_VALUE,
                        PieceType::Bishop => BISHOP_VALUE,
                        PieceType::Rook => ROOK_VALUE,
                        PieceType::Queen => QUEEN_VALUE,
                        PieceType::King => KING_VALUE,
                    };
                    
                    // Add value for the player's pieces, subtract for opponent's pieces
                    if piece.color == color {
                        score += piece_value;
                        
                        // Bonus for controlling the center (e4, d4, e5, d5)
                        if (file == 3 || file == 4) && (rank == 3 || rank == 4) {
                            score += CENTER_CONTROL_BONUS;
                        }
                        
                        // Development bonus for minor pieces
                        if (piece.piece_type == PieceType::Knight || piece.piece_type == PieceType::Bishop) &&
                           ((color == Color::White && rank > 0) || (color == Color::Black && rank < 7)) {
                            score += DEVELOPED_PIECE_BONUS;
                        }
                    } else {
                        score -= piece_value;
                        
                        // Bonus for opponent controlling the center
                        if (file == 3 || file == 4) && (rank == 3 || rank == 4) {
                            score -= CENTER_CONTROL_BONUS;
                        }
                        
                        // Development bonus for opponent's minor pieces
                        if (piece.piece_type == PieceType::Knight || piece.piece_type == PieceType::Bishop) &&
                           ((color == Color::Black && rank > 0) || (color == Color::White && rank < 7)) {
                            score -= DEVELOPED_PIECE_BONUS;
                        }
                    }
                }
            }
        }
        
        score
    }
    
    // Convert a move to standard algebraic notation (SAN)
    pub fn to_algebraic_notation(&self, chess_move: &ChessMove, board: &Board) -> String {
        // This is a simplified version - a full SAN implementation would be more complex
        let piece = match board.get_piece(&chess_move.from) {
            Some(p) => p,
            None => return String::from("???"),
        };
        
        let piece_letter = match piece.piece_type {
            PieceType::Pawn => "",
            PieceType::Knight => "N",
            PieceType::Bishop => "B",
            PieceType::Rook => "R",
            PieceType::Queen => "Q",
            PieceType::King => "K",
        };
        
        format!("{}{}", piece_letter, chess_move.to)
    }
    
    // Get statistics about the search
    pub fn get_stats(&self) -> (u32, u8) {
        (self.nodes_searched, self.depth)
    }
    
    // Public method for getting legal moves - to be used by Game
    pub fn get_legal_moves(&self, board: &Board, color: Color) -> Result<Vec<ChessMove>, ChessError> {
        self.generate_moves(board, color)
    }
}