#[derive(Debug, Clone)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
    pub wanted_col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_up(&mut self, row_len: usize) -> bool {
        if self.row > 0 {
            self.row -= 1;
            self.col = self.wanted_col.min(row_len.checked_sub(1).unwrap_or(0));
        }
        self.row > 0
    }

    pub fn move_right(&mut self, row_len: usize) -> bool {
        if self.col + 1 < row_len {
            self.col += 1;
            self.wanted_col = self.col;
        }
        self.col + 1 < row_len
    }

    pub fn move_down(&mut self, row_len: usize, num_rows: usize) -> bool {
        if self.row + 1 < num_rows {
            self.row += 1;
            self.col = self.wanted_col.min(row_len.checked_sub(1).unwrap_or(0));
        }
        self.row + 1 < num_rows
    }

    pub fn move_left(&mut self) -> bool {
        if self.col > 0 {
            self.col -= 1;
            self.wanted_col = self.col;
        }
        self.col > 0
    }

    pub fn relative_to(&self, x: usize, y: usize) -> (usize, usize) {
        (
            self.col.checked_sub(x).unwrap_or(0),
            self.row.checked_sub(y).unwrap_or(0),
        )
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            col: 0,
            row: 0,
            wanted_col: 0,
        }
    }
}
