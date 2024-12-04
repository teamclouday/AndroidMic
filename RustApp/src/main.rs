// to not launch a console on Windows, only in release because it blocks all logs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::run_ui;

#[macro_use]
extern crate log;

mod app;
mod audio;
mod config;
mod map_bytes;
mod streamer;
mod utils;
mod view;

#[macro_use]
mod localize;

fn main() {
    env_logger::init();
    localize::localize();
    run_ui()
}
