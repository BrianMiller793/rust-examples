#[derive(Debug, Clone)]
pub struct Board {
    board: Vec<char>,
    row: u32,
    size: u32,
}

impl Board {
    pub fn new(size: u32) -> Board {
        Board {
            board : vec![' '; (size * size) as usize],
            row : 0,
            size: size,
        }
    }

    pub fn is_safe(&mut self, column: u32) -> bool {
        self.row < self.size &&
        self.board[(self.row * self.size + column) as usize ] == ' '
    }

    pub fn is_end_row(&mut self) -> bool {
        return self.row == self.size;
    }

    pub fn set_queen(&mut self, column: u32) {
        let mut x: i32;

        self.board[(self.size * self.row + column) as usize] = 'Q';
        self.row = self.row + 1;

        // Mark attack vectors
        if self.row < self.size {
            // diagonal left
            x = column as i32;
            for y in self.row..self.size {
                x = x - 1;
                if x < 0 {
                    break;
                }

                self.board[(self.size * y + x as u32) as usize] = 'r';
            }

            // diagonal right
            x = column as i32;
            for y in self.row..self.size {
                x = x + 1;
                if x as u32 == self.size {
                    break;
                }

                self.board[(self.size * y + x as u32) as usize] = 'r';
            }

            // vertical
            for y in self.row..self.size {
                self.board[(self.size * y + column) as usize] = 'r';
            }
        }
    }
}

fn main() {
    let size = 15;
    let mut boards : Vec<Board> = Vec::new();

    // init
    for c in 0..size {
        let mut board = Board::new(size);
        board.set_queen(c);
        boards.push(board);
    }

    loop {
        if let Some(mut board) = boards.pop() {
            for c in 0..size {
                if board.is_safe(c) {
                    let mut newboard = board.clone();
                    newboard.set_queen(c);
                    if newboard.is_end_row() == true {
                        println!("{:?}", newboard);
                    } else {
                        boards.push(newboard);
                    }
                }
            }
        } else {
            break;
        }
    }
}
