use app::run_ui;

#[macro_use]
extern crate log;

mod app;
mod audio;
mod config;
mod message;
mod streamer;
// mod user_action;
mod utils;
mod view;

fn main() {
    env_logger::init();

    run_ui()
}
