use std::{sync::mpsc, time};

use crossterm::{event, terminal};

use crate::{buffer, input, render};

const MAX_RENDER_DELAY: time::Duration = time::Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
}

#[derive(Debug)]
pub struct RunningEditor {
    pub running: bool,
    pub buffers: Vec<buffer::Buffer>,
    pub cur_buf: usize,
    pub cursor: (usize, usize),
    pub wanted_col: usize,
    pub window_size: (usize, usize),
    pub scroll: (usize, usize),
    pub screen_dirty: bool,
    pub mode: Mode,
}

impl RunningEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_buf(buf: buffer::Buffer) -> Self {
        let (w, h) = terminal::size().expect("failed to get terinal size");
        let window_size = (w as usize, h as usize);
        Self {
            buffers: vec![buf],
            cur_buf: 0,
            screen_dirty: true,
            running: false,
            window_size,
            cursor: (0, 0),
            wanted_col: 0,
            scroll: (0, 0),
            mode: Mode::Normal,
        }
    }

    pub fn run(mut self) {
        self.running = true;
        let render_sender = render::spawn_render_thread();

        let (send, input_receiver) = mpsc::channel();
        input::spawn_input_thread(send);

        loop {
            let start = time::Instant::now();
            loop {
                let elapsed = time::Instant::now() - start;
                if elapsed >= MAX_RENDER_DELAY {
                    break;
                }
                if let Ok(e) = input_receiver.recv_timeout(MAX_RENDER_DELAY - elapsed) {
                    self.handle_input(e);
                }
            }

            if !self.running {
                break;
            }

            if self.screen_dirty {
                let ok = render_sender.send((&self).into());
                assert!(ok.is_ok());
            }
        }
    }

    fn handle_input(&mut self, e: event::Event) {
        match (e, self.mode) {
            (event::Event::Key(event::KeyEvent { code, .. }), m) => match (code, m) {
                (event::KeyCode::Esc, Mode::Normal) => self.running = false,
                (event::KeyCode::Esc, Mode::Insert) => {
                    self.mode = Mode::Normal;
                    let buf = &self.buffers[self.cur_buf];
                    let row_len = buf.row_len(self.cursor.1);
                    if self.cursor.0 >= row_len {
                        self.cursor.0 = row_len.checked_sub(1).unwrap_or(0);
                    }
                }
                (event::KeyCode::Char('i'), Mode::Normal) => self.mode = Mode::Insert,
                (event::KeyCode::Char('a'), Mode::Normal) => {
                    self.mode = Mode::Insert;
                    let buf = &self.buffers[self.cur_buf];
                    self.move_cursor_right();
                }
                (event::KeyCode::Char('j'), Mode::Normal) => {
                    if self.move_cursor_down() {
                        self.mark_dirty();
                    }
                }
                (event::KeyCode::Char('k'), Mode::Normal) => {
                    if self.move_cursor_up() {
                        self.mark_dirty();
                    }
                }
                (event::KeyCode::Char('h'), Mode::Normal) => {
                    if self.move_cursor_left() {
                        self.mark_dirty();
                    }
                }
                (event::KeyCode::Char('l'), Mode::Normal) => {
                    if self.move_cursor_right() {
                        self.mark_dirty();
                    }
                }
                (event::KeyCode::Enter, Mode::Insert) => {
                    let cur = self.cur_buf;
                    let (col, row) = (self.cursor.0, self.cursor.1);
                    let buf = &mut self.buffers[cur];
                    buf.insert(col, row, '\n');
                    self.move_cursor_down();
                    self.cursor.0 = 0;
                    self.wanted_col = 0;
                    self.mark_dirty();
                }
                (event::KeyCode::Char(c), Mode::Insert) => {
                    let cur = self.cur_buf;
                    let (col, row) = (self.cursor.0, self.cursor.1);
                    let buf = &mut self.buffers[cur];
                    buf.insert(col, row, c);
                    self.move_cursor_right();
                    self.mark_dirty();
                }
                _ => (),
            },
            (event::Event::Resize(width, height), _) => {
                self.window_size = (width as usize, height as usize)
            }
            _ => (),
        }
    }

    fn mark_dirty(&mut self) {
        self.screen_dirty = true;
    }

    fn move_cursor_up(&mut self) -> bool {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
            let buf = &self.buffers[self.cur_buf];
            let row_len = buf.row_len(self.cursor.1);
            self.cursor.0 = self.wanted_col.min(row_len.checked_sub(1).unwrap_or(0));

            self.check_scroll_up();
            self.check_scroll_left();
            self.check_scroll_right();

            return true;
        }
        false
    }

    fn move_cursor_down(&mut self) -> bool {
        let buf = &self.buffers[self.cur_buf];
        let num_rows = buf.num_rows();
        if self.cursor.1 + 1 < num_rows {
            self.cursor.1 += 1;
            let row_len = buf.row_len(self.cursor.1);
            self.cursor.0 = self.wanted_col.min(row_len.checked_sub(1).unwrap_or(0));

            self.check_scroll_down();
            self.check_scroll_left();
            self.check_scroll_right();

            return true;
        }
        false
    }

    fn move_cursor_left(&mut self) -> bool {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.wanted_col = self.cursor.0;

            self.check_scroll_left();

            return true;
        }
        false
    }

    fn move_cursor_right(&mut self) -> bool {
        let buf = &self.buffers[self.cur_buf];
        let row_len = buf.row_len(self.cursor.1) + if self.mode == Mode::Insert { 1 } else { 0 };
        if self.cursor.0 < row_len - 1 {
            self.cursor.0 += 1;
            self.wanted_col = self.cursor.0;

            self.check_scroll_right();

            return true;
        }
        false
    }

    fn check_scroll_up(&mut self) {
        if self.cursor.1 < self.scroll.1 {
            self.scroll.1 = self.cursor.1;
        }
    }

    fn check_scroll_down(&mut self) {
        if self.cursor.1 >= self.scroll.1 + self.window_size.1 {
            self.scroll.1 = self.cursor.1 - self.window_size.1 + 1;
        }
    }

    fn check_scroll_left(&mut self) {
        if self.cursor.0 < self.scroll.0 {
            self.scroll.0 = self.cursor.0;
        }
    }

    fn check_scroll_right(&mut self) {
        if self.cursor.0 >= self.scroll.0 + self.window_size.0 {
            self.scroll.0 = self.cursor.0 - self.window_size.0 + 1;
        }
    }
}

impl Default for RunningEditor {
    fn default() -> Self {
        Self::with_buf(Default::default())
    }
}
