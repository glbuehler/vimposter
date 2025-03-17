use std::{sync::mpsc, thread};

use crossterm::event;

pub fn spawn_input_thread(ch: mpsc::Sender<event::Event>) {
    thread::spawn(move || {
        loop {
            match event::read() {
                Ok(e) => {
                    if let Err(_) = ch.send(e) {
                        break;
                    }
                }
                Err(e) => {
                    println!("{e}");
                }
            }
        }
    });
}
