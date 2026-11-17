use anyhow::Result;
use std::time::Duration;
use tokio::process::Command;

use crate::{
    config::ConnectionMode,
    streamer::{StreamerMsg, tcp_streamer},
};

use super::{
    AudioStream, ConnectError, StreamerTrait,
    tcp_streamer::{TcpStreamer, TcpStreamerState},
};

pub struct AdbStreamer {
    tcp_streamer: TcpStreamer,
}

/// attempts to recreate the reverse proxy after the TCP listener came back
const ADB_REVERSE_ATTEMPTS: u32 = 3;
/// base delay between reverse proxy rebuild attempts
const ADB_REVERSE_RETRY_DELAY: Duration = Duration::from_millis(500);

async fn get_connected_devices() -> Result<Vec<String>, ConnectError> {
    let mut cmd = Command::new("adb");
    cmd.arg("devices");

    let output = exec_cmd(cmd).await?;
    let mut devices = Vec::new();

    // Skip the first line which is "List of devices attached"
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            devices.push(parts[0].to_string());
        }
    }

    Ok(devices)
}

async fn remove_adb_reverse_proxy(device_id: &str, port: u16) -> Result<(), ConnectError> {
    let mut cmd = Command::new("adb");
    cmd.arg("-s")
        .arg(device_id)
        .arg("reverse")
        .arg("--remove")
        .arg(format!("tcp:{}", port));

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

        return Err(ConnectError::AdbStatusCommand {
            code: status.status.code(),
            stderr,
        });
    }
    let stdout = String::from_utf8_lossy(&status.stdout).trim().to_string();
    Ok(stdout)
}

/// Recreate the reverse proxy for every currently attached device.
/// `new()` and the post-drop recovery path share this sequence.
async fn recreate_reverse_proxies(port: u16) -> Result<(), ConnectError> {
    let devices = get_connected_devices().await?;
    if devices.is_empty() {
        return Err(ConnectError::NoAdbDevice);
    }

    for device_id in &devices {
        if let Err(e) = remove_adb_reverse_proxy(device_id, port).await
            && !e.to_string().contains("not found")
        {
            warn!("cannot remove adb proxy for device {device_id}: {e}");
        }

        let mut cmd = Command::new("adb");
        cmd.arg("-s")
            .arg(device_id)
            .arg("reverse")
            .arg(format!("tcp:{}", port))
            .arg(format!("tcp:{}", port));
        exec_cmd(cmd).await?;
    }

    Ok(())
}

/// Rebuild the reverse proxy with a few bounded retries. A `adb kill-server`
/// or a replug can remove the tunnel while the local TCP listener stays alive.
async fn rebuild_reverse_proxies(port: u16) -> Result<(), ConnectError> {
    let mut last_error = None;

    for attempt in 0..ADB_REVERSE_ATTEMPTS {
        match recreate_reverse_proxies(port).await {
            Ok(()) => {
                info!("adb reverse proxy rebuilt");
                return Ok(());
            }
            Err(e) => {
                warn!(
                    "can't rebuild adb reverse proxy (attempt {}/{}): {e}",
                    attempt + 1,
                    ADB_REVERSE_ATTEMPTS
                );
                last_error = Some(e);
                if attempt + 1 < ADB_REVERSE_ATTEMPTS {
                    tokio::time::sleep(ADB_REVERSE_RETRY_DELAY * (attempt + 1)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or(ConnectError::NoAdbDevice))
}

pub async fn new(port: u16, stream_config: AudioStream) -> Result<AdbStreamer, ConnectError> {
    let tcp_streamer = tcp_streamer::new("127.0.0.1".parse().unwrap(), port, stream_config).await?;

    recreate_reverse_proxies(tcp_streamer.port).await?;

    let streamer = AdbStreamer { tcp_streamer };
    Ok(streamer)
}

impl StreamerTrait for AdbStreamer {
    async fn next(&mut self) -> Result<Option<StreamerMsg>, ConnectError> {
        let was_streaming = matches!(&self.tcp_streamer.state, TcpStreamerState::Streaming { .. });
        let msg = self.tcp_streamer.next().await?;

        if was_streaming && matches!(msg, Some(StreamerMsg::Listening { .. })) {
            info!("rebuilding adb reverse proxy after the connection dropped");
            rebuild_reverse_proxies(self.tcp_streamer.port).await?;
        }

        Ok(msg)
    }

    fn reconfigure_stream(&mut self, config: AudioStream) {
        self.tcp_streamer.reconfigure_stream(config)
    }

    fn status(&self) -> StreamerMsg {
        match &self.tcp_streamer.state {
            TcpStreamerState::Listening { .. } => StreamerMsg::Listening {
                ip: None,
                port: None,
            },
            TcpStreamerState::Streaming { .. } => StreamerMsg::Connected {
                ip: None,
                port: None,
                mode: ConnectionMode::Adb,
            },
        }
    }
}

impl Drop for AdbStreamer {
    fn drop(&mut self) {
        let port = self.tcp_streamer.port;
        tokio::spawn(async move {
            let devices: Vec<String> = get_connected_devices().await.unwrap_or_default();

            for device_id in devices {
                if let Err(e) = remove_adb_reverse_proxy(&device_id, port).await {
                    warn!("cannot remove adb proxy for device {device_id}: {e}");
                }
            }
        });
    }
}
