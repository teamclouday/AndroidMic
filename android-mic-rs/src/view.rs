use cosmic::{
    widget::{button, column, radio, text},
    Element,
};

use crate::{
    app::{AppMsg, AppState},
    config::ConnectionMode,
};

pub fn view_app<'a>(app: &'a AppState) -> Element<'a, AppMsg> {
    column()
        .push(connection_type(&app.config.settings().connection_mode))
        .push(button::text("Connect"))
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
