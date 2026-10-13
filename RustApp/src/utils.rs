use std::path::PathBuf;

use cargo_packager_resource_resolver as resource_resolver;
use file_rotate::{ContentLimit, FileRotate, compression::Compression, suffix::AppendCount};
use log::error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use cached::cached;
use constcat::concat;

pub const APP_ID: &str = concat!(QUALIFIER, ".", ORG, ".", APP);

pub const QUALIFIER: &str = "io.github";
pub const ORG: &str = "teamclouday";
pub const APP: &str = "AndroidMic";

#[cached]
pub fn resource_dir(os_dependent: bool) -> PathBuf {
    if cfg!(ANDROID_MIC_FORMAT = "flatpak") {
        PathBuf::from(format!("/app/share/{APP_ID}/res"))
    } else {
        match resource_resolver::current_format() {
            Ok(format) => resource_resolver::resources_dir(format).unwrap(),
            Err(e) => {
                if matches!(e, resource_resolver::Error::UnkownPackageFormat) {
                    let path = std::fs::canonicalize("res").unwrap();
                    if os_dependent {
                        if cfg!(target_os = "linux") {
                            path.join("linux")
                        } else if cfg!(target_family = "windows") {
                            path.join("windows")
                        } else if cfg!(target_os = "macos") {
                            path.join("macos")
                        } else {
                            panic!("unsupported os")
                        }
                    } else {
                        path
                    }
                } else {
                    error!("{e}");
                    panic!()
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn setup_wgpu() {
    unsafe {
        std::env::set_var("WGPU_BACKEND", "gl");
    }
    debug!("WGPU: Forcing OpenGL backend for graphics.");
}

#[cfg(not(target_os = "windows"))]
pub fn setup_wgpu() {
    debug!("WGPU: Using default graphics backend.");
}

pub fn initialize_logfile(log_file_path: &PathBuf) -> WorkerGuard {
    // configure size limit and 2 backup for rotation
    let rotator = FileRotate::new(
        log_file_path,
        AppendCount::new(2),                   // 2 backups
        ContentLimit::Bytes(10 * 1024 * 1024), // 10 MB
        Compression::None,
        None,
    );

    // run on background threads
    let (non_blocking_file, guard) = tracing_appender::non_blocking(rotator);

    // set up log filter that can be overwrite by RUST_LOG
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    // set up subscriber with dual outputs
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stdout).with_target(true))
        .with(
            fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking_file)
                .with_target(true),
        )
        .init();

    guard
}
