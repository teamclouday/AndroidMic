use super::app::AudioDevice;
use crate::{
    config::{AppTheme, AudioFormat, ChannelCount, ConnectionMode, SampleRate},
    streamer::StreamerMsg,
};

#[derive(Debug, Clone)]
pub enum AppMsg {
    ChangeConnectionMode(ConnectionMode),
    Streamer(StreamerMsg),
    Device(AudioDevice),
    Connect,
    Stop,
    ToggleSettingsWindow,
    Config(ConfigMsg),
    RefreshAudioDevices,
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum ConfigMsg {
    SampleRate(SampleRate),
    ChannelCount(ChannelCount),
    AudioFormat(AudioFormat),
    UseRecommendedFormat,
    StartAtLogin(bool),
    AutoConnect(bool),
    DeNoise(bool),
    Theme(AppTheme),
}
