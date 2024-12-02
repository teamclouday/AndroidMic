use cosmic::{
    iced_widget::pick_list,
    widget::{button, column, horizontal_space, radio, row, settings, text, vertical_space},
    Element,
};
use cpal::traits::DeviceTrait;

use crate::{
    app::{AdvancedWindow, AppMsg, AppState, State},
    config::{AudioFormat, ChannelCount, ConnectionMode, SampleRate},
    fl,
};

pub fn view_app(app: &AppState) -> Element<'_, AppMsg> {
    row()
        .padding(50)
        .push(
            column()
                .push(logs(app))
                .push(vertical_space())
                .push(audio_wave(app)),
        )
        .push(horizontal_space())
        .push(
            column()
                .push(audio(app))
                .push(vertical_space())
                .push(connection_type(app)),
        )
        .into()
}

fn logs(_app: &AppState) -> Element<'_, AppMsg> {
    text("log").into()
}

fn audio_wave(_app: &AppState) -> Element<'_, AppMsg> {
    text("audio wave").into()
}

fn audio(app: &AppState) -> Element<'_, AppMsg> {
    let selected = app
        .audio_device
        .as_ref()
        .and_then(|d| d.name().ok())
        .and_then(|name| app.audio_devices.iter().find(|d| d.name == name));

    column()
        .push(text::title4(fl!("audio_device")))
        .push(pick_list(
            app.audio_devices.clone(),
            selected,
            AppMsg::Device,
        ))
        .push(button::text(fl!("advanced")).on_press(AppMsg::AdvancedOptions))
        .into()
}

fn connection_type(app: &AppState) -> Element<'_, AppMsg> {
    let connection_mode = &app.config.data().connection_mode;

    column()
        .push(text::title4(fl!("connection")))
        .push(radio(
            "TCP",
            &ConnectionMode::Tcp,
            Some(connection_mode),
            |mode| AppMsg::ChangeConnectionMode(*mode),
        ))
        // .push(radio(
        //     "UDP",
        //     &ConnectionMode::Udp,
        //     Some(connection_mode),
        //     |mode| AppMsg::ChangeConnectionMode(*mode),
        // ))
        .push(radio(
            "USB (ADB)",
            &ConnectionMode::Adb,
            Some(connection_mode),
            |mode| AppMsg::ChangeConnectionMode(*mode),
        ))
        .push(connect_button(app))
        .into()
}

fn connect_button(app: &AppState) -> Element<'_, AppMsg> {
    match app.state {
        State::Default => button::text(fl!("connect")).on_press(AppMsg::Connect),
        State::Listening => button::text(fl!("listening")).on_press(AppMsg::Stop),
        State::Connected => button::destructive(fl!("disconnect")).on_press(AppMsg::Stop),
        State::WaitingOnStatus => button::text(fl!("waiting")),
    }
    .into()
}

pub fn advanced_window<'a>(
    app: &'a AppState,
    _advanced_window: &'a AdvancedWindow,
) -> Element<'a, AppMsg> {
    let config = app.config.data();

    column()
        .push(settings::section().title(fl!("sample_rate")).add(pick_list(
            SampleRate::VALUES,
            Some(&config.sample_rate),
            AppMsg::ChangeSampleRate,
        )))
        .push(
            settings::section()
                .title(fl!("channel_count"))
                .add(pick_list(
                    ChannelCount::VALUES,
                    Some(&config.channel_count),
                    AppMsg::ChangeChannelCount,
                )),
        )
        .push(
            settings::section()
                .title(fl!("audio_format"))
                .add(pick_list(
                    AudioFormat::VALUES,
                    Some(&config.audio_format),
                    AppMsg::ChangeAudioFormat,
                )),
        )
        .into()
}
