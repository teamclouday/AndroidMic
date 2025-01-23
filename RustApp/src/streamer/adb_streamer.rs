use std::process::Command;

use anyhow::Result;
use rtrb::Producer;

use crate::streamer::tcp_streamer;

use super::{tcp_streamer::TcpStreamer, ConnectError, Status, StreamerTrait};

pub struct AdbStreamer {
    tcp_streamer: TcpStreamer,
}

fn remove_all_adb_reverse_proxy() -> Result<(), ConnectError> {
    let mut cmd = Command::new("adb");
    cmd.arg("reverse").arg("--remove-all");

    exec_cmd(cmd)?;

    Ok(())
}

fn exec_cmd(mut cmd: Command) -> Result<(), ConnectError> {
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;

    // https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000);

    let status = cmd.output().map_err(ConnectError::CommandFailed)?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr).to_string();

        return Err(ConnectError::StatusCommand {
            code: status.status.code(),
            stderr,
        });
    }
    Ok(())
}

pub async fn new(producer: Producer<u8>) -> Result<AdbStreamer, ConnectError> {
    let tcp_streamer = tcp_streamer::new(str::parse("127.0.0.1").unwrap(), producer).await?;

    remove_all_adb_reverse_proxy()?;

    let mut cmd = Command::new("adb");
    cmd.arg("reverse")
        .arg(format!("tcp:{}", 6000))
        .arg(format!("tcp:{}", tcp_streamer.port));

    exec_cmd(cmd)?;

    let streamer = AdbStreamer { tcp_streamer };
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
        if let Err(e) = remove_all_adb_reverse_proxy() {
            error!("drop AdbStreamer: {e}");
        }
    }
}
