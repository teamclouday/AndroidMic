use std::{
    fs,
    process::{Child, Command},
};

use crate::streamer::streamer_trait::DEFAULT_PORT;
use rtrb::Producer;

use crate::utils;

use super::{
    streamer_trait::{ConnectError, StreamerTrait, WriteError},
    tcp_streamer_async::{self, TcpStreamer},
};

pub struct AdbStreamer {
    tcp_streamer: TcpStreamer,
    adb_process: Child,
}

pub async fn new() -> Result<AdbStreamer, ConnectError> {
 
    let tcp_streamer = tcp_streamer_async::new(str::parse("127.0.0.1").unwrap()).await?;

    let adb_exe_path = utils::resource_dir(true).join("adb/adb");
    dbg!(&utils::resource_dir(true).display());
    dbg!(adb_exe_path.display());


    let status = Command::new(&adb_exe_path)
        .arg("reverse")
        .arg("--remove-all")
        .output()
        .map_err(ConnectError::CommandFailed)?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr).to_string();

        return Err(ConnectError::StatusCommand {
            code: status.status.code(),
            stderr,
        });
    }

    let child = Command::new(&adb_exe_path)
        .arg("reverse")
        .arg(format!("tcp:{DEFAULT_PORT} tcp:{}", tcp_streamer.port))
        .spawn()
        .map_err(ConnectError::CommandFailed)?;

    let streamer = AdbStreamer {
        tcp_streamer,
        adb_process: child,
    };
    Ok(streamer)
}

impl StreamerTrait for AdbStreamer {
    async fn process(&mut self, shared_buf: &mut Producer<u8>) -> Result<usize, WriteError> {
        self.tcp_streamer.process(shared_buf).await
    }

    fn port(&self) -> Option<u16> {
        self.tcp_streamer.port()
    }
}

impl Drop for AdbStreamer {
    fn drop(&mut self) {
        if let Err(e) = self.adb_process.kill() {
            error!("{e}")
        }
    }
}
