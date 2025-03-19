#[derive(Debug, Default)]
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

    pub fn insert(&mut self, mut col: usize, mut row: usize, ch: char) {
        col += 2;
        let i = self
            .content
            .char_indices()
            .take_while(|(_, c)| {
                if row == 0 && col > 0 {
                    col -= 1;
                } else if c == &'\n' {
                    row -= 1;
                }
                col > 0 || row > 0
            })
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);

        assert!(i < self.content.len());
        self.content.insert(i, ch as char);
    }
}
