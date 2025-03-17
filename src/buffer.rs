#[derive(Debug, Default)]
pub struct Buffer {
    pub content: String,
}

impl Buffer {
    pub fn row_len(&self, row: usize) -> Option<usize> {
        Some(self.content.lines().skip(row).next()?.len())
    }

    pub fn num_rows(&self) -> usize {
        self.content.lines().count()
    }
}
