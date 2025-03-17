use anyhow::Result;
use rtrb::Producer;
use tokio::process::Command;

use crate::streamer::tcp_streamer;

use super::{tcp_streamer::TcpStreamer, ConnectError, Status, StreamerTrait};

pub struct AdbStreamer {
    tcp_streamer: TcpStreamer,
}

async fn get_connected_devices() -> Result<Vec<String>, ConnectError> {
    let mut cmd = Command::new("adb");
    cmd.arg("devices");

    let output = exec_cmd(cmd).await?;
    let mut devices = Vec::new();

    // Skip the first line which is "List of devices attached"
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            devices.push(parts[0].to_string());
        }
    }

    Ok(devices)
}

async fn remove_adb_reverse_proxy(device_id: &str) -> Result<(), ConnectError> {
    let mut cmd = Command::new("adb");
    cmd.arg("-s")
        .arg(device_id)
        .arg("reverse")
        .arg("--remove")
        .arg("tcp:55555");

    exec_cmd(cmd).await?;

    Ok(())
}

async fn exec_cmd(mut cmd: Command) -> Result<String, ConnectError> {
    // https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000);

    let status = cmd.output().await.map_err(ConnectError::CommandFailed)?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr).to_string();

        return Err(ConnectError::StatusCommand {
            code: status.status.code(),
            stderr,
        });
    }
    let stdout = String::from_utf8_lossy(&status.stdout).trim().to_string();
    Ok(stdout)
}

pub async fn new(producer: Producer<u8>) -> Result<AdbStreamer, ConnectError> {
    let tcp_streamer = tcp_streamer::new(str::parse("127.0.0.1").unwrap(), producer).await?;

    let devices = get_connected_devices().await?;
    if devices.is_empty() {
        return Err(ConnectError::NoAdbDevice);
    }

    for device_id in &devices {
        if let Err(e) = remove_adb_reverse_proxy(device_id).await {
            warn!("cannot remove adb proxy for device {device_id}: {e}");
        }

        let mut cmd = Command::new("adb");
        cmd.arg("-s")
            .arg(device_id)
            .arg("reverse")
            .arg("tcp:55555")
            .arg(format!("tcp:{}", tcp_streamer.port));
        exec_cmd(cmd).await?;
    }

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
        tokio::task::spawn_blocking(|| async {
            let devices = get_connected_devices().await.unwrap_or_default();

            for device_id in devices {
                if let Err(e) = remove_adb_reverse_proxy(&device_id).await {
                    warn!("cannot remove adb proxy for device {device_id}: {e}");
                }
            }
        });
    }
}
