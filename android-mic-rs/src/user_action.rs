use std::{io, sync::mpsc::Sender, thread};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use strum::{Display, EnumString};

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
pub struct Args {
    #[arg(short, long, help = "example: -i 192.168.1.79")]
    pub ip: Option<String>,

    #[arg(short = 'm', long = "mode", id = "connection mode", help = "UDP or TCP", default_value_t = ConnectionMode::Udp)]
    pub connection_mode: ConnectionMode,

    #[arg(short = 'c', long = "channel", id = "channel count",  help = "1 or 2", default_value_t = ChannelCount::Mono)]
    pub channel_count: ChannelCount,
}

// todo: parse it
pub fn ask_ip() -> String {
    println!("Please enter the ip of the host (The IP of your PC)");
    println!("Help: something like: 192.168.1.79");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().into()
}

#[derive(Debug, Clone, EnumString, PartialEq, Display)]
pub enum ConnectionMode {
    #[strum(serialize = "udp", serialize = "UDP")]
    Udp,
    #[strum(serialize = "tcp", serialize = "TCP")]
    Tcp,
}

#[derive(Debug, Clone, EnumString, PartialEq, Display)]
pub enum ChannelCount {
    #[strum(serialize = "mono", serialize = "MONO", serialize = "1")]
    Mono,
    #[strum(serialize = "stereo", serialize = "STEREO", serialize = "2")]
    Stereo,
}
