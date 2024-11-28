use cosmic::{
    iced_widget::pick_list,
    widget::{button, column, dropdown, radio, row, settings, text},
    Element,
};
use cpal::traits::DeviceTrait;

use crate::{
    app::{AppMsg, AppState, AudioHost, State},
    config::ConnectionMode,
};

pub fn view_app(app: &AppState) -> Element<'_, AppMsg> {
    column()
        .push(audio(app))
        .push(connection_type(&app.config.settings().connection_mode))
        .push(connect_button(app))
        .into()
}

fn audio(app: &AppState) -> Element<'_, AppMsg> {
    let device_pos = app
        .audio_device
        .as_ref()
        .and_then(|d| d.name().ok())
        .and_then(|name| app.audio_devices.iter().position(|d| d.name == name));

    column()
        .push(text::title2("Audio"))
        .push(settings::section().title("Host").add(
            row().push(text(app.audio_host.id().name())).push(pick_list(
                app.audio_hosts.clone(),
                Some(AudioHost {
                    id: app.audio_host.id(),
                    name: "",
                }),
                |a| AppMsg::Host(a.id),
            )),
        ))
        .push(settings::section().title("Device").add(dropdown(
            &app.audio_devices,
            device_pos,
            AppMsg::Device,
        )))
        .into()
}

fn connection_type(connection_mode: &ConnectionMode) -> Element<'_, AppMsg> {
    column()
        .push(text("Connection"))
        .push(radio(
            "TCP",
            &ConnectionMode::Tcp,
            Some(connection_mode),
            |mode| AppMsg::ConnectionMode(*mode),
        ))
        .push(radio(
            "UDP",
            &ConnectionMode::Udp,
            Some(connection_mode),
            |mode| AppMsg::ConnectionMode(*mode),
        ))
        .push(radio(
            "ADB/USB",
            &ConnectionMode::Adb,
            Some(connection_mode),
            |mode| AppMsg::ConnectionMode(*mode),
        ))
        .into()
}

fn connect_button(app: &AppState) -> Element<'_, AppMsg> {
    let (name, message) = match app.state {
        State::Default => ("Connect", Some(AppMsg::Connect)),
        State::Listening => ("Listening", Some(AppMsg::Stop)),
        State::Connected => ("Connected", Some(AppMsg::Stop)),
        State::WaitingOnStatus => ("Waiting", None),
    };

    button::text(name).on_press_maybe(message).into()
}
