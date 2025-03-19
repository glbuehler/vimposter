use std::{
    io::{self, Write},
    iter,
    sync::{Arc, RwLock, mpsc},
    thread,
};

use crate::editor;

const NORMAL_CURSOR: &[u8] = b"\x1B[2 q";
const INSERT_CURSOR: &[u8] = b"\x1B[5 q";

const SET_00: &[u8] = b"\x1B[0;0H";
const CURSOR_HIDE: &[u8] = b"\x1B[?25l";
const CURSOR_SHOW: &[u8] = b"\x1B[?25h";
macro_rules! SET_CURSOR {
    ($l1:expr, $l2:expr) => {
        format!("\x1B[{};{}H", $l2 + 1, $l1 + 1)
    };
}

pub fn spawn_render_thread(ch: mpsc::Receiver<()>, ed: Arc<RwLock<editor::RunningEditor>>) {
    thread::spawn(move || {
        while let Ok(_) = ch.recv() {
            let set;
            let mut render_buf: Vec<u8>;
            {
                let ed = ed.read().expect("unable to get read lock");
                let (w, h) = ed.window_size;
                let (scroll_x, scroll_y) = ed.scroll;
                let buf = &ed.buffers[ed.cur_buf];
                let (cursor_x, cursor_y) = ed.cursor.relative_to(scroll_x, scroll_y);
                set = Vec::from(SET_CURSOR!(cursor_x, cursor_y));

                render_buf = Vec::with_capacity(
                    w * h
                        + NORMAL_CURSOR.len()
                        + CURSOR_HIDE.len()
                        + CURSOR_SHOW.len()
                        + SET_00.len()
                        + set.len(),
                );
                render_buf.extend(match ed.mode {
                    editor::Mode::Normal => NORMAL_CURSOR,
                    editor::Mode::Insert => INSERT_CURSOR,
                });
                render_buf.extend(CURSOR_HIDE);
                render_buf.extend(SET_00);

                let mut line_count = 0;
                for l in buf.content.lines().skip(scroll_y).take(h) {
                    let mut l = match l.char_indices().nth(scroll_x).map(|(i, _)| i) {
                        Some(n) => l.get(n..).unwrap(),
                        None => "",
                    };
                    let llen = l.chars().count();
                    if llen > w {
                        let wc = l.char_indices().nth(w).unwrap().0;
                        l = l.get(..wc).unwrap();
                    }
                    let padding = w - l.chars().count();
                    render_buf.extend(l.as_bytes());
                    render_buf.extend(iter::repeat_n(b' ', padding));
                    render_buf.extend(b"\r\n");
                    line_count += 1;
                }

                for _ in 0..ed.window_size.1 - line_count {
                    render_buf.extend(vec![b' '; ed.window_size.0]);
                    render_buf.extend(b"\r\n");
                }
                render_buf.pop();
            }
            render_buf.extend(set);
            render_buf.extend(CURSOR_SHOW);

            if let Err(e) = io::stdout().write_all(&render_buf) {
                panic!("{e}");
            }
            if let Err(e) = io::stdout().flush() {
                panic!("{e}");
            }
        }
    });
}
