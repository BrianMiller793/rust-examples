/// Chess board structure for constraint-based solution.
///

#[derive(Debug, Clone)]
pub struct Board {
    board: Vec<char>,
    row: u32,
    pub size: u32,
}

impl Board {
    pub fn new(size: u32) -> Board {
        Board {
            board : vec![' '; (size * size) as usize],
            row : 0,
            size: size,
        }
    }

    /// Get the size of the chess board.
    pub fn get_size(&mut self) -> u32 {
        self.size
    }

    /// Check if a column is safe.
    pub fn is_safe(&mut self, column: u32) -> bool {
        self.row < self.size &&
        self.board[(self.row * self.size + column) as usize ] == ' '
    }

    /// Check if the current row is the last row.
    pub fn is_end_row(&mut self) -> bool {
        self.row == self.size
    }

    /// Set a queen on the board, and mark its attack vectors.
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
