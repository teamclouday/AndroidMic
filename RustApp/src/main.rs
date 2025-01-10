// to not launch a console on Windows, only in release because it blocks all logs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::Path;

use app::run_ui;
use clap::Parser;
use config::{Args, Config};
use directories::ProjectDirs;
use utils::{APP, ORG, QUALIFIER};
use zconf::ConfigManager;

#[macro_use]
extern crate log;

mod app;
mod audio;
mod config;
mod map_bytes;
mod message;
mod streamer;
mod usb;
mod utils;
mod view;

#[macro_use]
mod localize;

fn main() {
    env_logger::try_init().unwrap_or_else(|_| {
        env_logger::builder()
            .filter_level(log::LevelFilter::Warn)
            .filter_module("android_mic", log::LevelFilter::Debug)
            .init()
    });

    info!("hello");

    let project_dirs = ProjectDirs::from(QUALIFIER, ORG, APP).unwrap();

    let config_path = if cfg!(debug_assertions) {
        Path::new("config")
    } else {
        project_dirs.config_dir()
    };

    let mut config: ConfigManager<Config> =
        ConfigManager::new(config_path.join(format!("{APP}.toml")));

    let args = Args::parse();

    config.update_without_write(|config| {
        if let Some(ip) = args.ip {
            config.ip.replace(ip);
        }

        if let Some(connection_mode) = args.connection_mode {
            config.connection_mode = connection_mode;
        }

        if let Some(output_device) = args.output_device {
            config.device_name.replace(output_device);
        }

        if let Some(audio_format) = args.audio_format {
            config.audio_format = audio_format;
        }

        if let Some(channel_count) = args.channel_count {
            config.channel_count = channel_count;
        }
        if let Some(sample_rate) = args.sample_rate {
            config.sample_rate = sample_rate;
        }
    });

    localize::localize();
    run_ui(config)
}
