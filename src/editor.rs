use std::{
    sync::{Arc, RwLock, mpsc},
    time,
};

use crossterm::{event, terminal};

use crate::{buffer, cursor, input, render};

const MAX_RENDER_DELAY: time::Duration = time::Duration::from_millis(50);

#[derive(Debug)]
pub struct RunningEditor {
    pub running: bool,
    pub buffers: Vec<buffer::Buffer>,
    pub cur_buf: usize,
    pub cursor: cursor::Cursor,
    pub window_size: (usize, usize),
    pub scroll: (usize, usize),
    pub screen_dirty: bool,
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
            cursor: Default::default(),
            scroll: (0, 0),
        }
    }

    pub fn run(mut self) {
        const R_LOCK_FAIL: &str = "unable to acquire read lock";

        self.running = true;
        let self_arc = Arc::new(RwLock::new(self));
        let (render_sender, r) = mpsc::channel();
        render::spawn_render_thread(r, self_arc.clone());

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
                    Self::handle_input(&self_arc, e);
                }
            }

            let running = self_arc.read().expect(R_LOCK_FAIL).running;
            if !running {
                break;
            }

            let dirty = self_arc.read().expect(R_LOCK_FAIL).screen_dirty;
            if dirty {
                let ok = render_sender.send(());
                assert!(ok.is_ok());
            }
        }
    }

    fn handle_input(self_arc: &Arc<RwLock<Self>>, e: event::Event) {
        const W_LOCK_FAIL: &str = "unable to acquire write lock";
        match e {
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Esc,
                ..
            }) => {
                self_arc.write().expect(W_LOCK_FAIL).running = false;
            }
            event::Event::Resize(w, h) => {
                self_arc.write().expect(W_LOCK_FAIL).window_size = (w as usize, h as usize)
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('j'),
                ..
            }) => {
                let mut w = self_arc.write().expect(W_LOCK_FAIL);
                let buf = &w.buffers[w.cur_buf];
                let row_len = buf.row_len(w.cursor.row + 1).unwrap_or(0);
                let num_rows = buf.num_rows();
                if w.cursor.move_down(row_len, num_rows) {
                    if w.cursor.row - w.scroll.1 >= w.window_size.1 {
                        w.scroll.1 = w.cursor.row - w.window_size.1;
                    }
                    w.mark_dirty();
                }
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('k'),
                ..
            }) => {
                let mut w = self_arc.write().expect(W_LOCK_FAIL);
                let buf = &w.buffers[w.cur_buf];
                let row_len = if w.cursor.row == 0 {
                    0
                } else {
                    buf.row_len(w.cursor.row - 1).unwrap_or(0)
                };
                if w.cursor.move_up(row_len) {
                    if w.cursor.row < w.scroll.1 {
                        w.scroll.1 = w.cursor.row;
                    }
                    w.mark_dirty();
                }
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('h'),
                ..
            }) => {
                let mut w = self_arc.write().expect(W_LOCK_FAIL);
                if w.cursor.move_left() {
                    if w.cursor.col < w.scroll.0 {
                        w.scroll.0 = w.cursor.col;
                    }
                    w.mark_dirty();
                }
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('l'),
                ..
            }) => {
                let mut w = self_arc.write().expect(W_LOCK_FAIL);
                let buf = &w.buffers[w.cur_buf];
                let row_len = buf.row_len(w.cursor.row).unwrap_or(0);
                if w.cursor.move_right(row_len) {
                    if w.cursor.col - w.scroll.0 > w.window_size.0 {
                        w.scroll.0 = w.cursor.col - w.window_size.0 - 1;
                    }
                    w.mark_dirty();
                }
            }
            event::Event::Key(kev) => {
                self_arc.write().expect(W_LOCK_FAIL).screen_dirty = true;
            }
            _ => (),
        }
    }

    fn mark_dirty(&mut self) {
        self.screen_dirty = true;
    }
}

impl Default for RunningEditor {
    fn default() -> Self {
        Self::with_buf(Default::default())
    }
}
