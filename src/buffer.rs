use std::iter;

#[derive(Debug, Default, Clone)]
pub struct Buffer {
    pub content: String,
}

impl Buffer {
    pub fn row_len(&self, row: usize) -> usize {
        assert!(row < self.num_rows());
        self.content.lines().nth(row).unwrap().chars().count()
    }

    pub fn num_rows(&self) -> usize {
        self.content.lines().count()
    }

    pub fn insert(&mut self, col: usize, row: usize, ch: char) {
        let i = self.col_row_index(col, row).expect("invalid position");
        self.content.insert(i, ch);
    }

    pub fn remove(&mut self, col: usize, row: usize) {
        let i = self
            .col_row_index(col, row)
            .map(|i| i - 1)
            .expect("invalid position");
        self.content.remove(i);
    }

    fn col_row_index(&self, col: usize, mut row: usize) -> Option<usize> {
        let mut idx = 0;
        for (i, c) in self.content.char_indices() {
            if row == 0 {
                idx = i;
                break;
            }
            if c == '\n' {
                row -= 1;
            }
        }
        if idx + col >= self.content.len() {
            return None;
        }
        Some(idx + col)
    }
}
