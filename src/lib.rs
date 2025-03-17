use std::io;

use crossterm::{execute, terminal};
use editor::RunningEditor;

mod buffer;
mod cursor;
mod editor;
mod input;
mod render;

static FILE: &str = include_str!("./editor.rs");

pub fn enter() {
    terminal::enable_raw_mode().unwrap();
    execute!(io::stdout(), terminal::EnterAlternateScreen).unwrap();
}

pub fn exit() {
    execute!(io::stdout(), terminal::LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
}

pub fn run() {
    let buf = buffer::Buffer {
        content: FILE.to_string(),
    };
    let ed = RunningEditor::with_buf(buf);
    ed.run();
}
