extern crate queens;

use queens::board::Board;
use queens::threadpool::ThreadPool;
use std::sync::{Arc, Mutex};

fn main() {
    let size = 15;
    let mut boards : Vec<Board> = Vec::new();
    let solved_boards = Arc::new(Mutex::new(0));
    let mut pool = ThreadPool::new(2);

    // Initialize, one queen per column
    for c in 0..size {
        let mut board = Board::new(size);
        board.set_queen(c);
        boards.push(board);
    }

    // Queue up the work pool
    while let Some(board) = boards.pop() {
        let solved_boards = Arc::clone(&solved_boards);
        pool.execute(|| {
            test_set_board(board, solved_boards);
        });
    }

    pool.wait();
    println!("Number of solutions: {}", *solved_boards.lock().unwrap());
}

/// Test a board, and set queens.
fn test_set_board(board: Board, solved_boards: Arc<Mutex<i32>>) {
    let mut boards : Vec<Board> = Vec::new();
    boards.push(board);

    while let Some(mut board) = boards.pop() {
        for c in 0..board.get_size() {
            if board.is_safe(c) {
                let mut newboard = board.clone();
                newboard.set_queen(c);
                if newboard.is_end_row() == true {
                    let mut solved_boards = solved_boards.lock().unwrap();
                    *solved_boards += 1;
                } else {
                    boards.push(newboard);
                }
            }
        }
    }
}

