use cosmic::{
    widget::{button, column, dropdown, horizontal_space, radio, row, text, vertical_space},
    Element,
};
use cpal::traits::DeviceTrait;

use crate::{
    app::{AdvancedWindow, AppMsg, AppState, State},
    config::ConnectionMode,
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
    let device_pos = app
        .audio_device
        .as_ref()
        .and_then(|d| d.name().ok())
        .and_then(|name| app.audio_devices.iter().position(|d| d.name == name));

    column()
        .push(text::title4(fl!("audio_device")))
        .push(dropdown(&app.audio_devices, device_pos, AppMsg::Device))
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
    let (name, message) = match app.state {
        State::Default => (fl!("connect"), Some(AppMsg::Connect)),
        State::Listening => (fl!("listening"), Some(AppMsg::Stop)),
        State::Connected => (fl!("connected"), Some(AppMsg::Stop)),
        State::WaitingOnStatus => (fl!("waiting"), None),
    };

    button::text(name).on_press_maybe(message).into()
}

pub fn advanced_window<'a>(
    _app: &'a AppState,
    _advanced_window: &'a AdvancedWindow,
) -> Element<'a, AppMsg> {
    text("todo").into()
}
