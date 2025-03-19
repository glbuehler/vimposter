use std::{
    io::{self, Write},
    iter,
    sync::mpsc,
    thread,
};

use crate::{buffer, editor};

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

pub(crate) struct RenderInfo {
    pub window_size: (usize, usize),
    pub cursor: (usize, usize),
    pub scroll: (usize, usize),
    pub buf: buffer::Buffer,
    pub mode: editor::Mode,
}

impl From<&editor::RunningEditor> for RenderInfo {
    fn from(value: &editor::RunningEditor) -> Self {
        Self {
            window_size: value.window_size,
            cursor: value.cursor,
            scroll: value.scroll,
            buf: value.buffers[value.cur_buf].clone(),
            mode: value.mode,
        }
    }
}

pub(crate) fn spawn_render_thread() -> mpsc::Sender<RenderInfo> {
    let (s, recv) = mpsc::channel::<RenderInfo>();
    thread::spawn(move || {
        for info in recv.into_iter() {
            let (w, h) = info.window_size;
            let (scroll_x, scroll_y) = info.scroll;
            let (cursor_x, cursor_y) = (info.cursor.0 - scroll_x, info.cursor.1 - scroll_y);
            let set_cursor = Vec::from(SET_CURSOR!(cursor_x, cursor_y));

            let mut render_buf = Vec::with_capacity(w * h);
            render_buf.extend(match info.mode {
                editor::Mode::Normal => NORMAL_CURSOR,
                editor::Mode::Insert => INSERT_CURSOR,
            });
            render_buf.extend(CURSOR_HIDE);
            render_buf.extend(SET_00);

            write_buf_content(&mut render_buf, &info.buf, scroll_x, scroll_y, w, h);

            render_buf.extend(set_cursor);
            render_buf.extend(CURSOR_SHOW);

            if let Err(e) = io::stdout().write_all(&render_buf) {
                panic!("{e}");
            }
            if let Err(e) = io::stdout().flush() {
                panic!("{e}");
            }
        }
    });
    s
}

fn write_buf_content(
    render_buf: &mut Vec<u8>,
    buf: &buffer::Buffer,
    scroll_x: usize,
    scroll_y: usize,
    window_x: usize,
    window_y: usize,
) {
    let mut y_padding = window_y;
    for l in buf.content.lines().skip(scroll_y).take(window_y) {
        let start = l.char_indices().nth(scroll_x).map(|(i, _)| i).unwrap_or(0);
        let l = &l[start..];
        let end = l
            .char_indices()
            .nth(scroll_x + window_x)
            .map(|(i, _)| i)
            .unwrap_or(l.len());
        let l = &l[..end];
        let line_padding = window_x - l.chars().count();
        render_buf.extend(l.as_bytes());
        render_buf.extend(iter::repeat_n(b' ', line_padding));
        render_buf.extend(b"\r\n");

        y_padding -= 1;
    }

    for _ in 0..y_padding {
        render_buf.extend(vec![b' '; window_x]);
        render_buf.extend(b"\r\n");
    }
    render_buf.pop();
    render_buf.pop();
}
