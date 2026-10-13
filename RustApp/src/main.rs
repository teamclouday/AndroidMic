// to not launch a console on Windows, only in release because it blocks all logs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use android_mic::{localize, single_instance};
use std::io::Write;
use std::{fs::File, path::Path};

use android_mic::config::{Args, Config};
use android_mic::ui::app::run_ui;
use android_mic::utils::{self, APP, ORG, QUALIFIER, initialize_logfile};
use clap::Parser;
use directories::ProjectDirs;
use fslock::LockFile;
use zconf::ConfigManager;

use android_mic::ui::app::Flags;

#[macro_use]
extern crate log;

struct DualWriter {
    file: Box<File>,
}

impl Write for DualWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes_written = self.file.write(buf)?;
        std::io::stdout().write_all(buf)?;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()?;
        std::io::stdout().flush()
    }
}

fn main() {
    let _ = fix_path_env::fix();
    utils::setup_wgpu();

    let project_dirs = ProjectDirs::from(QUALIFIER, ORG, APP).unwrap();

    let log_path = if cfg!(debug_assertions) {
        Path::new("log").to_path_buf()
    } else {
        project_dirs.cache_dir().to_path_buf()
    };
    std::fs::create_dir_all(&log_path).expect("Failed to create log directory");
    let log_file_path = log_path.join(format!("{}.log", APP));

    // NOTE: log guard should be alive for entire main lifespan
    let _log_guard = initialize_logfile(&log_file_path);

    // ensure single instance
    let instance_lock_path = if cfg!(debug_assertions) {
        std::path::PathBuf::from("log").join("app.lock")
    } else {
        project_dirs.cache_dir().join("app.lock")
    };
    let mut app_lock = LockFile::open(&instance_lock_path).expect("Failed to open app lock file");
    if !app_lock.try_lock_with_pid().unwrap_or(false) {
        info!(
            "Another instance is already running. PID can be found in {:?}",
            instance_lock_path
        );

        if let Err(e) = single_instance::send_event(single_instance::IpcEvent::Show) {
            error!("can't send ipc event {e}");
        }
        return;
    }

    // generated from https://patorjk.com/software/taag/#p=display&h=2&f=Doom&t=AndroidMic
    info!(
        r"
   ___              _              _      _ ___  ___ _
  / _ \            | |            (_)    | ||  \/  |(_)
 / /_\ \ _ __    __| | _ __  ___   _   __| || .  . | _   ___
 |  _  || '_ \  / _` || '__|/ _ \ | | / _` || |\/| || | / __|
 | | | || | | || (_| || |  | (_) || || (_| || |  | || || (__
 \_| |_/|_| |_| \__,_||_|   \___/ |_| \__,_|\_|  |_/|_| \___|
    "
    );

    let config_path = if cfg!(debug_assertions) {
        Path::new("config")
    } else {
        project_dirs.config_dir()
    };
    std::fs::create_dir_all(config_path).expect("Failed to create config directory");
    let config_file_path = config_path.join(format!("{APP}.toml"));

    let mut config: ConfigManager<Config> = ConfigManager::new(config_file_path.clone());

    let args = Args::parse();

    config.update_without_write(|config| {
        if let Some(ip) = args.ip {
            config.ip.replace(ip);
        }

        if let Some(connection_mode) = args.connection_mode {
            config.connection_mode = connection_mode;
        }

        if let Some(output_device) = args.output_device {
            config.device_id.replace(output_device);
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

    let flags = Flags {
        config,
        config_path: config_file_path.to_string_lossy().to_string(),
        log_path: log_file_path.to_string_lossy().to_string(),
        launched_automatically: args.launched_automatically,
    };

    run_ui(flags)
}
