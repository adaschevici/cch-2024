use std::fmt;
use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response, Result},
    routing::{get, post},
    Json, Router,
};
use tokio::sync::RwLock;

use std::{ops::DerefMut, sync::Arc, time::Duration};
use thiserror::Error;

type GameBoardType = Arc<RwLock<GameBoard>>;

#[derive(Error, Debug)]
enum AppError {
    #[error("Out of bounds")]
    OutOfBounds,
    #[error("Column overflow")]
    ColumnOverflow,
    #[error("Invalid piece")]
    InvalidPiece,
    #[error("Game over")]
    GameOver(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::OutOfBounds => (StatusCode::BAD_REQUEST, "Out of bounds".to_string()),
            AppError::ColumnOverflow => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Column overflow".to_string(),
            ),
            AppError::InvalidPiece => (StatusCode::NOT_FOUND, "Invalid piece".to_string()),
            AppError::GameOver(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
        };

        (status, error_message).into_response()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum BoardLocation {
    Empty,
    Milk,
    Wall,
    Cookie,
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

impl FromStr for BoardLocation {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "🍪" => Ok(BoardLocation::Cookie),
            "⬛" => Ok(BoardLocation::Empty),
            "🥛" => Ok(BoardLocation::Milk),
            "⬜" => Ok(BoardLocation::Wall),
            "cookie" => Ok(BoardLocation::Cookie),
            "milk" => Ok(BoardLocation::Milk),
            _ => Err(AppError::InvalidPiece),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct GameBoard {
    rows: usize,
    columns: usize,
    board: Vec<Vec<BoardLocation>>,
    game_status: GameResult,
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
            game_status: GameResult::InProgress,
        }
    }

    fn check_horizontal(&self, row: usize, col: usize) -> (bool, BoardLocation) {
        let player = self.board[row][col].clone();
        if player == BoardLocation::Wall || player == BoardLocation::Empty {
            return (false, player);
        }
        let mut count = 0;
        for i in (0..=3)
            .map(|i| col as i32 - i)
            .filter(|&x| x >= 0 && x <= self.columns as i32)
            .map(|x| x as usize)
        {
            if self.board[row][i] == player {
                count += 1;
            } else {
                break;
            }
        }
        for i in 1..=3 {
            let new_col = col + i;
            if new_col <= self.columns && self.board[row][new_col] == player {
                count += 1;
            } else {
                break;
            }
        }
        if count >= 4 {
            return (true, player);
        }
        return (false, player);
    }

    fn check_vertical(&self, row: usize, col: usize) -> (bool, BoardLocation) {
        let player = self.board[row][col].clone();
        if player == BoardLocation::Wall || player == BoardLocation::Empty {
            return (false, player);
        }
        let mut count = 0;
        for i in (0..=3)
            .map(|i| row as i32 - i)
            .filter(|&x| x >= 0 && x <= self.rows as i32)
            .map(|x| x as usize)
        {
            if self.board[i][col] == player {
                count += 1;
            } else {
                break;
            }
        }
        for i in 1..=3 {
            let new_row = row + i;
            if new_row <= self.rows && self.board[new_row][col] == player {
                count += 1;
            } else {
                break;
            }
        }
        if count >= 4 {
            return (true, player);
        }
        return (false, player);
    }

    fn check_diagonal(&self, row: usize, col: usize) -> (bool, BoardLocation) {
        let player = self.board[row][col].clone();
        if player == BoardLocation::Wall || player == BoardLocation::Empty {
            return (false, player);
        }
        let mut count = 0;
        for i in (0..=3)
            .map(|i| (row as i32 - i, col as i32 - i))
            .filter(|&(x, y)| x >= 0 && x <= self.rows as i32 && y >= 1 && y <= self.columns as i32)
            .map(|(x, y)| (x as usize, y as usize))
        {
            if self.board[i.0][i.1] == player {
                count += 1;
            } else {
                break;
            }
        }
        for i in (1..=3)
            .map(|i| (row as i32 + i, col as i32 + i))
            .filter(|&(x, y)| x >= 1 && x <= self.rows as i32 && y >= 1 && y <= self.columns as i32)
            .map(|(x, y)| (x as usize, y as usize))
        {
            if self.board[i.0][i.1] == player {
                count += 1;
            } else {
                break;
            }
        }
        if count >= 4 {
            return (true, player);
        }

        // Check diagonal (bottom-left to top-right)
        let mut count = 0;
        for i in (0..=3)
            .map(|i| (row as i32 + i, col as i32 - i))
            .filter(|&(x, y)| x >= 0 && x <= self.rows as i32 && y >= 1 && y <= self.columns as i32)
            .map(|(x, y)| (x as usize, y as usize))
        {
            if self.board[i.0][i.1] == player {
                count += 1;
            } else {
                break;
            }
        }
        for i in (1..=3)
            .map(|i| (row as i32 - i, col as i32 + i))
            .filter(|&(x, y)| x >= 0 && x <= self.rows as i32 && y >= 1 && y <= self.columns as i32)
            .map(|(x, y)| (x as usize, y as usize))
        {
            if self.board[i.0][i.1] == player {
                count += 1;
            } else {
                break;
            }
        }
        if count >= 4 {
            return (true, player);
        }
        return (false, player);
    }
    fn set_game_status(&mut self, game_status: GameResult) {
        self.game_status = game_status;
    }
    fn check(&self, starting_position: Option<(usize, usize)>) -> GameResult {
        let empty_cells = self
            .board
            .iter()
            .flat_map(|row| row.iter())
            .filter(|x| x == &&BoardLocation::Empty)
            .count();
        if let Some((row, col)) = starting_position {
            let (horizontal_win, player) = self.check_horizontal(row, col);
            if horizontal_win {
                return GameResult::Win(player);
            }
            let (vertical_win, player) = self.check_vertical(row, col);
            if vertical_win {
                return GameResult::Win(player);
            }
            let (diagonal_win, player) = self.check_diagonal(row, col);
            if diagonal_win {
                return GameResult::Win(player);
            }
        }
        if starting_position.is_none() {
            for row in 1..=self.rows - 1 {
                for col in 1..=self.columns - 1 {
                    let (horizontal_win, player) = self.check_horizontal(row, col);
                    if horizontal_win {
                        return GameResult::Win(player);
                    }
                    let (vertical_win, player) = self.check_vertical(row, col);
                    if vertical_win {
                        return GameResult::Win(player);
                    }
                    let (diagonal_win, player) = self.check_diagonal(row, col);
                    if diagonal_win {
                        return GameResult::Win(player);
                    }
                }
            }
        }
        if empty_cells == 0 {
            return GameResult::Draw;
        }
        GameResult::InProgress
    }

    fn set_cell(&mut self, col: usize, value: BoardLocation) -> Result<(usize, usize), AppError> {
        if (col > self.columns - 1) || (col < 1) {
            return Err(AppError::OutOfBounds);
        }

        for row in (1..self.rows).rev() {
            if self.board[row - 1][col] == BoardLocation::Empty {
                self.board[row - 1][col] = value;
                return Ok((row - 1, col)); // Successfully placed the piece
            }
        }

        Err(AppError::ColumnOverflow) // No empty space found in the column
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

#[derive(Debug, PartialEq, Eq, Clone)]
enum GameResult {
    Win(BoardLocation),
    Draw,
    InProgress,
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameResult::Win(player) => write!(f, "{} wins!", player),
            GameResult::Draw => write!(f, "No winner."),
            GameResult::InProgress => write!(f, ""),
        }
    }
}

async fn get_board(State(board): State<GameBoardType>) -> String {
    let board_state = board.read().await;
    let game_status = board_state.game_status.clone();
    match game_status {
        GameResult::Win(_) | GameResult::Draw => {
            return format!("{}{}\n", board_state.to_string(), game_status);
        }
        _ => {
            return format!("{}{}", board_state.to_string(), game_status);
        }
    }
}

async fn place_piece(
    Path((team, column)): Path<(String, usize)>,
    State(board): State<GameBoardType>,
) -> Result<String, AppError> {
    let piece = BoardLocation::from_str(&team)?;
    let mut write_board = board.write().await;
    if write_board.game_status == GameResult::Draw
        || write_board.game_status == GameResult::Win(BoardLocation::Cookie)
        || write_board.game_status == GameResult::Win(BoardLocation::Milk)
    {
        return Err(AppError::GameOver(format!(
            "{}{}\n",
            write_board.to_string(),
            write_board.game_status
        )));
    }
    let current_move = write_board.set_cell(column, piece);
    if let Ok(play) = current_move {
        let game_status = write_board.check(Some(play));
        match game_status {
            GameResult::Win(player) => {
                write_board.set_game_status(GameResult::Win(player.clone()));
                return Ok(format!("{}{} wins!\n", write_board.to_string(), player));
            }
            GameResult::Draw => {
                write_board.set_game_status(GameResult::Draw);
                return Ok(format!("{}No winner.\n", write_board.to_string()));
            }
            GameResult::InProgress => {
                return Ok(format!("{}", write_board.to_string()));
            }
        }
    } else {
        return Err(AppError::ColumnOverflow);
    }
}

async fn reset_board(State(board): State<GameBoardType>) -> String {
    let new_board = GameBoard::new(5, 6);
    let mut write_board = board.write().await;
    *write_board = new_board;

    return format!("{}", write_board.to_string());
}

pub fn router() -> Router {
    let board = Arc::new(RwLock::new(GameBoard::new(5, 6)));
    Router::new()
        .route("/board", get(get_board))
        .route("/reset", post(reset_board))
        .route("/place/:team/:column", post(place_piece))
        .with_state(board.clone())
}
