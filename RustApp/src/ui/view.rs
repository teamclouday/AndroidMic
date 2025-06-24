use std::{collections::HashMap, sync::LazyLock};

use cosmic::{
    Element,
    iced::{Length, alignment::Horizontal, widget::pick_list},
    widget::{
        button, canvas, column, container, context_menu, menu, radio, row, scrollable, settings,
        text, toggler, vertical_space,
    },
};
use cpal::traits::DeviceTrait;

use super::{
    app::{AppState, ConnectionState},
    message::{AppMsg, ConfigMsg},
};
use crate::{
    config::{AppTheme, AudioFormat, ChannelCount, ConnectionMode, SampleRate},
    fl,
    ui::message::MenuMsg,
    widget_icon_button,
};

pub static SCROLLABLE_ID: LazyLock<cosmic::widget::Id> = LazyLock::new(cosmic::widget::Id::unique);

pub fn main_window(app: &AppState) -> Element<'_, AppMsg> {
    row()
        .padding(50)
        .spacing(50)
        .push(
            column()
                .width(Length::FillPortion(2))
                .height(Length::Fill)
                .spacing(50)
                .push(logs(app))
                .push(wave(app)),
        )
        .push(
            column()
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .push(audio(app))
                .push(vertical_space())
                .push(connection_type(app)),
        )
        .into()
}

fn logs(app: &AppState) -> Element<'_, AppMsg> {
    context_menu(
        container(scrollable(text(app.logs.clone()).width(Length::Fill)).id(SCROLLABLE_ID.clone()))
            .width(Length::Fill)
            .height(Length::FillPortion(3))
            .padding(13)
            .class(cosmic::theme::Container::Card),
        Some(menu::items(
            &HashMap::new(),
            vec![menu::Item::Button(
                fl!("clear_logs"),
                None,
                MenuMsg::ClearLogs,
            )],
        )),
    )
    .into()
}

fn wave(app: &AppState) -> Element<'_, AppMsg> {
    container(canvas(&app.audio_wave).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::FillPortion(1))
        .into()
}

fn audio(app: &AppState) -> Element<'_, AppMsg> {
    let selected = app
        .audio_device
        .as_ref()
        .and_then(|d| d.name().ok())
        .and_then(|name| app.audio_devices.iter().find(|d| d.name == name));

    column()
        .spacing(20)
        .align_x(Horizontal::Center)
        .push(text::title4(fl!("audio_device")))
        .push(
            row()
                .width(Length::Fill)
                .spacing(5)
                .push(
                    pick_list(app.audio_devices.clone(), selected, AppMsg::Device)
                        .width(Length::Fill),
                )
                .push(
                    widget_icon_button!("refresh24")
                        .on_press(AppMsg::RefreshAudioDevices)
                        .class(cosmic::theme::Button::Text)
                        .width(Length::Shrink),
                ),
        )
        .push(button::text(fl!("settings")).on_press(AppMsg::ToggleSettingsWindow))
        .into()
}

fn connection_type(app: &AppState) -> Element<'_, AppMsg> {
    let connection_mode = &app.config.data().connection_mode;

    #[cfg(not(feature = "usb"))]
    let usb: Option<Element<_>> = None;

    #[cfg(feature = "usb")]
    let usb = Some(radio(
        "USB Serial",
        &ConnectionMode::Usb,
        Some(connection_mode),
        |mode| AppMsg::ChangeConnectionMode(*mode),
    ));

    column()
        .spacing(20)
        .align_x(Horizontal::Center)
        .push(text::title4(fl!("connection")))
        .push(
            column()
                .push(radio(
                    "WIFI / LAN (TCP)",
                    &ConnectionMode::Tcp,
                    Some(connection_mode),
                    |mode| AppMsg::ChangeConnectionMode(*mode),
                ))
                .push(radio(
                    "WIFI / LAN (UDP)",
                    &ConnectionMode::Udp,
                    Some(connection_mode),
                    |mode| AppMsg::ChangeConnectionMode(*mode),
                ))
                .push_maybe(usb)
                .push(radio(
                    "USB Adb",
                    &ConnectionMode::Adb,
                    Some(connection_mode),
                    |mode| AppMsg::ChangeConnectionMode(*mode),
                )),
        )
        .push(connect_button(app))
        .into()
}

fn connect_button(app: &AppState) -> Element<'_, AppMsg> {
    match app.connection_state {
        ConnectionState::Default => button::text(fl!("connect")).on_press(AppMsg::Connect),
        ConnectionState::Listening => button::text(fl!("listening")).on_press(AppMsg::Stop),
        ConnectionState::Connected => button::destructive(fl!("disconnect")).on_press(AppMsg::Stop),
        ConnectionState::WaitingOnStatus => button::text(fl!("waiting")),
    }
    .into()
}

pub fn settings_window(app: &AppState) -> Element<'_, ConfigMsg> {
    let config = app.config.data();

    scrollable(
        column()
            .padding(50)
            .spacing(20)
            .push(settings::section().title(fl!("sample_rate")).add(pick_list(
                SampleRate::VALUES,
                Some(&config.sample_rate),
                ConfigMsg::SampleRate,
            )))
            .push(
                settings::section()
                    .title(fl!("channel_count"))
                    .add(pick_list(
                        ChannelCount::VALUES,
                        Some(&config.channel_count),
                        ConfigMsg::ChannelCount,
                    )),
            )
            .push(
                settings::section()
                    .title(fl!("audio_format"))
                    .add(pick_list(
                        AudioFormat::VALUES,
                        Some(&config.audio_format),
                        ConfigMsg::AudioFormat,
                    )),
            )
            .push(
                settings::section()
                    .title(fl!("denoise"))
                    .add(toggler(config.denoise).on_toggle(ConfigMsg::DeNoise)),
            )
            .push(button::text("Use Recommended Format").on_press(ConfigMsg::UseRecommendedFormat))
            .push_maybe(if cfg!(target_os = "windows") {
                Some(
                    settings::section()
                        .title(fl!("start_at_login"))
                        .add(toggler(config.start_at_login).on_toggle(ConfigMsg::StartAtLogin)),
                )
            } else {
                None
            })
            .push(
                settings::section()
                    .title(fl!("auto_connect"))
                    .add(toggler(config.auto_connect).on_toggle(ConfigMsg::AutoConnect)),
            )
            .push(settings::section().title(fl!("theme")).add(pick_list(
                AppTheme::VALUES,
                Some(&config.theme),
                ConfigMsg::Theme,
            ))),
    )
    .into()
}
