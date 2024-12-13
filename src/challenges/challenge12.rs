use std::fmt;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Json, Router,
};
use tokio::sync::RwLock;

use std::{ops::DerefMut, sync::Arc, time::Duration};
use thiserror::Error;

type GameBoardType = Arc<RwLock<GameBoard>>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum BoardLocation {
    Cookie,
    Empty,
    Milk,
    Wall,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Illegal move")]
    IllegalMove,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::IllegalMove => (StatusCode::BAD_REQUEST, "Illegal move"),
        };

        (status, error_message).into_response()
    }
}

impl fmt::Display for BoardLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardLocation::Cookie => write!(f, "🍪"),
            BoardLocation::Empty => write!(f, "⬛"),
            BoardLocation::Milk => write!(f, "🥛"),
            BoardLocation::Wall => write!(f, "⬜"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct GameBoard {
    rows: usize,
    columns: usize,
    board: Vec<Vec<BoardLocation>>,
}

impl GameBoard {
    fn new(rows: usize, columns: usize) -> Self {
        let mut board = vec![vec![BoardLocation::Empty; columns]; rows];
        for row in 0..rows {
            for column in 0..columns {
                if (row == 0 && column == 0)
                    || (row == 0 && column == columns - 1)
                    || row == rows - 1
                    || column == 0
                    || column == columns - 1
                {
                    board[row][column] = BoardLocation::Wall;
                }
            }
        }
        Self {
            rows,
            columns,
            board,
        }
    }

    fn set_cell(&mut self, col: usize, value: BoardLocation) -> Result<(), String> {
        if (col > self.columns - 1) || (col < 1) {
            return Err("Column index out of bounds".to_string());
        }

        for row in (0..self.rows - 1).rev() {
            if self.board[row + 1][col] == BoardLocation::Empty {
                self.board[row + 1][col] = value;
                return Ok(()); // Successfully placed the piece
            }
        }

        Err("Column is full".to_string()) // No empty space found in the column
    }
}

impl fmt::Display for GameBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.board {
            for cell in row {
                write!(f, "{}", cell)?; // Use write! to the formatter
            }
            writeln!(f)?; // Newline at the end of each row
        }
        Ok(())
    }
}
async fn get_board(State(board): State<GameBoardType>) -> String {
    let board_state = board.read().await;
    return format!("{}", board_state.to_string());
}

async fn add_piece(State(board): State<GameBoardType>) -> String {
    let mut write_board = board.write().await;
    _ = write_board.set_cell(7, BoardLocation::Milk);
    return format!("{}", write_board.to_string());
}

async fn reset_board(State(board): State<GameBoardType>) -> String {
    let new_board = GameBoard::new(5, 6);
    let mut write_board = board.write().await;
    *write_board = new_board;

    return format!("Board reset to:\n{}", write_board.to_string());
}

pub fn router() -> Router {
    let board = Arc::new(RwLock::new(GameBoard::new(5, 6)));
    Router::new()
        .route("/board", get(get_board))
        .route("/reset", post(reset_board))
        .route("/addone", post(add_piece))
        .with_state(board.clone())
}