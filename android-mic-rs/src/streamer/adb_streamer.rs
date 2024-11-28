use std::{
    process::{Child, Command},
};

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

    let android_port= 6000;

    let child = Command::new(&adb_exe_path)
        .arg("reverse")
        .arg(format!("tcp:{android_port}"))
        .arg(format!("tcp:{}", tcp_streamer.port))
        .spawn()
        .map_err(ConnectError::CommandFailed)?;

    let streamer = AdbStreamer {
        tcp_streamer,
        adb_process: child,
    };
    Ok(streamer)
}

impl StreamerTrait for AdbStreamer {

    async fn listen(&mut self) -> Result<() ,ConnectError> {
        self.tcp_streamer.listen().await
    }

    fn port(&self) -> Option<u16> {
        self.tcp_streamer.port()
    }

    async fn process(&mut self, shared_buf: &mut Producer<u8>) -> Result<usize, WriteError> {
        self.tcp_streamer.process(shared_buf).await
    }

    
}

impl Drop for AdbStreamer {
    fn drop(&mut self) {
        if let Err(e) = self.adb_process.kill() {
            error!("{e}")
        }
    }
}
