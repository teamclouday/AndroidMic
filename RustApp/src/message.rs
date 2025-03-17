use crate::{
    app::AudioDevice,
    config::{AudioFormat, ChannelCount, ConnectionMode, SampleRate},
    streamer::StreamerMsg,
};

#[derive(Debug, Clone)]
pub enum AppMsg {
    ChangeConnectionMode(ConnectionMode),
    Streamer(StreamerMsg),
    Device(AudioDevice),
    Connect,
    Stop,
    AdvancedOptions,
    Config(ConfigMsg),
    RefreshAudioDevices,
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum ConfigMsg {
    SampleRate(SampleRate),
    ChannelCount(ChannelCount),
    AudioFormat(AudioFormat),
    StartAtLogin(bool),
    AutoConnect(bool),
}
