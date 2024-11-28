use std::process::{Child, Command};

use rtrb::Producer;

use crate::{streamer::tcp_streamer_async, utils};

use super::{tcp_streamer_async::TcpStreamer, ConnectError, Status, StreamerTrait, WriteError};

pub struct AdbStreamer {
    tcp_streamer: TcpStreamer,
    adb_process: Child,
}

pub async fn new(producer: Producer<u8>) -> Result<AdbStreamer, ConnectError> {
    let tcp_streamer = tcp_streamer_async::new(str::parse("127.0.0.1").unwrap(), producer).await?;

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

    let android_port = 6000;

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
    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        self.tcp_streamer.next().await
    }

    fn set_buff(&mut self, buff: Producer<u8>) {
        self.tcp_streamer.set_buff(buff)
    }
    
    fn status(&self) -> Option<Status> {
        self.tcp_streamer.status()
    }
}

impl Drop for AdbStreamer {
    fn drop(&mut self) {
        if let Err(e) = self.adb_process.kill() {
            error!("{e}")
        }
    }
}
