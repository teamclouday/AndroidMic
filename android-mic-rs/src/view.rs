use cosmic::{
    iced_widget::pick_list,
    widget::{button, column, dropdown, radio, settings, text},
    Element,
};

use crate::{
    app::{AppMsg, AppState, AudioHost},
    config::ConnectionMode,
};

pub fn view_app<'a>(app: &'a AppState) -> Element<'a, AppMsg> {
    column()
        .push(audio(&app))
        .push(connection_type(&app.config.settings().connection_mode))
        .push(button::text("Connect"))
        .into()
}

fn audio<'a>(app: &'a AppState) -> Element<'a, AppMsg> {
    column()
        .push(text::title2("Audio"))
        .push(settings::section().title("Host").add(pick_list(
            app.audio_hosts.clone(),
            Some(AudioHost {
                id: app.audio_host.id(),
                name: "",
            }),
            |a| AppMsg::Host(a.id),
        )))
        .push(settings::section().title("Device").add(dropdown(
            &app.audio_devices,
            None, // no way to compare device ?
            |index| AppMsg::Device(index),
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
