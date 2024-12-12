use std::fmt;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Json, Router,
};

#[derive(Debug, Clone)]
enum BoardLocation {
    Cookie,
    Empty,
    Milk,
    Wall,
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

#[derive(Debug)]
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
async fn get_board() -> String {
    let board = GameBoard::new(5, 6);
    return format!("{}", board.to_string());
}

async fn reset_board() -> &'static str {
    "Board reset!"
}

pub fn router() -> Router {
    Router::new()
        .route("/board", get(get_board))
        .route("/reset", post(reset_board))
}
