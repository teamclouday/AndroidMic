use std::{io, sync::mpsc::Sender, thread};

use clap::Parser;
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
    println!();
    println!("Available options:");
    println!("quit: q");
    println!();
}

#[derive(Parser, Debug)]
#[clap(author = "wiiznokes", version, about = "AndroidMic", long_about = None)]
pub struct SettingsArg {
    #[arg(long = "ip")]
    pub ip: Option<String>,

    #[arg(short = 'm', long = "mode", value_name = "CONNECTION MODE (UPD/TCP)")]
    pub mode: Option<String>,
}

pub fn ask_ip() -> String {
    println!("Please enter the ip of the host (The IP of your PC)");
    println!("Help: something like: 192.168.1.79");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().into()
}

#[derive(Debug, Clone)]
pub enum ConnectionMode {
    Udp,
    Tcp,
}

pub fn str_to_connection_mode(str: &str) -> Option<ConnectionMode> {
    match str {
        "UDP" => Some(ConnectionMode::Udp),
        "TCP" => Some(ConnectionMode::Tcp),
        _ => None,
    }
}

pub fn ask_connection_mode() -> ConnectionMode {
    loop {
        println!("Please enter the connection mode (UDP/TCP):");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimmed_input = input.trim();
        if let Some(connection_mode) = str_to_connection_mode(trimmed_input) {
            return connection_mode;
        } else {
            println!("Invalid input. Please enter 'UDP' or 'TCP'.");
        }
    }
}
