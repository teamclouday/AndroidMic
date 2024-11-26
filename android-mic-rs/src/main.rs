use app::run_ui;

#[macro_use]
extern crate log;

mod audio;
mod streamer;
// mod tcp_streamer;
// mod udp_streamer;
mod adb_streamer;
mod app;
mod config;
mod message;
mod streamer_sub;
mod tcp_streamer_async;
mod user_action;
mod view;

fn main() {
    env_logger::init();

    run_ui()
}
