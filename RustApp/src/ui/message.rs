use super::app::AudioDevice;
#[cfg(not(target_os = "linux"))]
use super::tray::SystemTrayMsg;
use crate::{
    config::{
        AppTheme, AudioEffect, AudioFormat, ChannelCount, ConnectionMode, DenoiseKind,
        NetworkAdapter, SampleRate,
    },
    streamer::StreamerMsg,
};

#[derive(Debug, Clone)]
pub enum AppMsg {
    ChangeConnectionMode(ConnectionMode),
    Streamer(StreamerMsg),
    Device(AudioDevice),
    Adapter(NetworkAdapter),
    Connect,
    Stop,
    ToggleSettingsWindow,
    Config(ConfigMsg),
    RefreshAudioDevices,
    RefreshNetworkAdapters,
    HideWindow,
    ShowWindow,
    Menu(MenuMsg),
    LinkClicked(String),
    #[cfg(not(target_os = "linux"))]
    SystemTray(SystemTrayMsg),
    Exit,
}

#[derive(Debug, Clone)]
pub enum ConfigMsg {
    SampleRate(SampleRate),
    ChannelCount(ChannelCount),
    AudioFormat(AudioFormat),
    UseRecommendedFormat,
    ResetDenoiseSettings,
    StartAtLogin(bool),
    StartMinimized(bool),
    AutoConnect(bool),
    DeNoise(bool),
    DeNoiseKind(DenoiseKind),
    SpeexNoiseSuppress(i32),
    SpeexVADEnabled(bool),
    SpeexVADThreshold(i32),
    SpeexAGCEnabled(bool),
    SpeexAGCTarget(i32),
    SpeexDereverbEnabled(bool),
    SpeexDereverbLevel(f32),
    Theme(AppTheme),
    Amplify(bool),
    AmplifyValue(f32),
    ToggleAboutWindow,
    PortTextInput(String),
    PortSave,
    PostAudioEffect(AudioEffect),
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
