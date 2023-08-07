use std::{sync::mpsc::Sender, thread};

use crossterm::event::{self, Event, KeyCode, KeyEvent};

pub enum UserAction {
    Quit,
}

#[allow(clippy::single_match)]
pub fn start_listening(tx: Sender<UserAction>) {
    let _join = thread::spawn(move || loop {
        match event::read() {
            Ok(event) => match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => tx.send(UserAction::Quit).unwrap(),
                _ => {}
            },
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    });
}



pub fn print_avaible_action() {
    println!("Available options:");
    println!("quit: q");
    println!("");
}