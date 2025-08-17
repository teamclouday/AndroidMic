use super::app::AudioDevice;
use crate::{
    config::{AppTheme, AudioFormat, ChannelCount, ConnectionMode, DenoiseKind, SampleRate},
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
    Menu(MenuMsg),
    LinkClicked(String),
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
    DeNoiseKind(DenoiseKind),
    Theme(AppTheme),
    Amplify(bool),
    AmplifyValue(String),
    ToggleAboutWindow,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum MenuMsg {
    ClearLogs,
}

use cosmic::widget::menu::action::MenuAction;

impl MenuAction for MenuMsg {
    type Message = AppMsg;

    fn message(&self) -> Self::Message {
        AppMsg::Menu(*self)
    }
}
