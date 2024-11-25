use serde::{Deserialize, Serialize};
use light_enum::Values;
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

