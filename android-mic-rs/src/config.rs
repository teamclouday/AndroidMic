use light_enum::Values;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub connection_mode: ConnectionMode,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionMode {
    #[default]
    Tcp,
    Udp,
    Adb,
}
