use std::{collections::HashMap, sync::LazyLock};

use cosmic::{
    Element,
    iced::{
        Length,
        alignment::{Horizontal, Vertical},
        widget::pick_list,
    },
    widget::{
        self, about::About, button, canvas, column, container, context_menu, horizontal_space,
        markdown, menu, radio, row, scrollable, settings, text, toggler, vertical_space,
    },
};
use cpal::traits::DeviceTrait;

use super::{
    app::{AppState, ConnectionState},
    message::{AppMsg, ConfigMsg},
};
use crate::{
    config::{AppTheme, AudioFormat, ChannelCount, ConnectionMode, DenoiseKind, SampleRate},
    fl,
    ui::message::MenuMsg,
    utils::APP,
    widget_icon_button, widget_icon_handle,
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
        container(
            scrollable(
                container(
                    markdown::view(
                        &app.logs,
                        markdown::Settings::with_text_size(12),
                        markdown::Style::from_palette(
                            cosmic::iced::Theme::TokyoNightStorm.palette(),
                        ),
                    )
                    .map(|url| AppMsg::LinkClicked(url.to_string())),
                )
                .width(Length::Fill),
            )
            .id(SCROLLABLE_ID.clone()),
        )
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
                .push_maybe({
                    #[cfg(feature = "usb")]
                    {
                        Some(radio(
                            "USB Serial",
                            &ConnectionMode::Usb,
                            Some(connection_mode),
                            |mode| AppMsg::ChangeConnectionMode(*mode),
                        ))
                    }

                    #[cfg(not(feature = "usb"))]
                    {
                        Option::<Element<AppMsg>>::None
                    }
                })
                .push_maybe({
                    #[cfg(feature = "adb")]
                    {
                        Some(radio(
                            "USB Adb",
                            &ConnectionMode::Adb,
                            Some(connection_mode),
                            |mode| AppMsg::ChangeConnectionMode(*mode),
                        ))
                    }

                    #[cfg(not(feature = "adb"))]
                    {
                        Option::<Element<AppMsg>>::None
                    }
                }),
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
            .push(
                settings::section()
                    .title("Audio format")
                    .add(
                        row()
                            .align_y(Vertical::Center)
                            .push(text(fl!("sample_rate")))
                            .push(horizontal_space())
                            .push(pick_list(
                                SampleRate::VALUES,
                                Some(&config.sample_rate),
                                ConfigMsg::SampleRate,
                            )),
                    )
                    .add(
                        row()
                            .align_y(Vertical::Center)
                            .push(text(fl!("channel_count")))
                            .push(horizontal_space())
                            .push(pick_list(
                                ChannelCount::VALUES,
                                Some(&config.channel_count),
                                ConfigMsg::ChannelCount,
                            )),
                    )
                    .add(
                        row()
                            .align_y(Vertical::Center)
                            .push(text(fl!("audio_format")))
                            .push(horizontal_space())
                            .push(pick_list(
                                AudioFormat::VALUES,
                                Some(&config.audio_format),
                                ConfigMsg::AudioFormat,
                            )),
                    )
                    .add(
                        row()
                            .push(horizontal_space())
                            .push(
                                button::text("Use Recommended Audio Format")
                                    .on_press(ConfigMsg::UseRecommendedFormat),
                            )
                            .push(horizontal_space()),
                    ),
            )
            .push(
                settings::section()
                    .title(fl!("denoise"))
                    .add(
                        row()
                            .align_y(Vertical::Center)
                            .push(text("Enabled"))
                            .push(horizontal_space())
                            .push(toggler(config.denoise).on_toggle(ConfigMsg::DeNoise)),
                    )
                    .add_maybe((config.denoise).then(|| {
                        row()
                            .align_y(Vertical::Center)
                            .push(text("Type"))
                            .push(horizontal_space())
                            .push(pick_list(
                                DenoiseKind::VALUES,
                                Some(&config.denoise_kind),
                                ConfigMsg::DeNoiseKind,
                            ))
                    }))
                    .add_maybe(
                        (config.denoise && config.denoise_kind == DenoiseKind::Speexdsp).then(
                            || {
                                row()
                                    .align_y(Vertical::Center)
                                    .spacing(10)
                                    .push(text(format!("{} db", config.speex_noise_suppress)))
                                    .push(
                                        widget::slider(
                                            -100..=0,
                                            config.speex_noise_suppress,
                                            ConfigMsg::SpeexNoiseSuppress,
                                        )
                                        .step(1),
                                    )
                            },
                        ),
                    ),
            )
            .push(
                settings::section()
                    .title("Voice Activity Detection (VAD)")
                    .add(
                        row()
                            .align_y(Vertical::Center)
                            .push(text("Enabled"))
                            .push(horizontal_space())
                            .push(
                                toggler(config.speex_vad_enabled)
                                    .on_toggle(ConfigMsg::SpeexVADEnabled),
                            ),
                    )
                    .add_maybe((config.speex_vad_enabled).then(|| {
                        row()
                            .align_y(Vertical::Center)
                            .spacing(10)
                            .push(text(format!("{}", config.speex_vad_threshold)))
                            .push(
                                widget::slider(
                                    0..=100,
                                    config.speex_vad_threshold as i32,
                                    ConfigMsg::SpeexVADThreshold,
                                )
                                .step(1),
                            )
                    })),
            )
            .push(
                settings::section()
                    .title("Gain Control")
                    .add(
                        row()
                            .align_y(Vertical::Center)
                            .push(text("Automatic Gain Control (AGC)"))
                            .push(horizontal_space())
                            .push(
                                toggler(config.speex_agc_enabled)
                                    .on_toggle(ConfigMsg::SpeexAGCEnabled),
                            ),
                    )
                    .add_maybe((config.speex_agc_enabled).then(|| {
                        row()
                            .align_y(Vertical::Center)
                            .spacing(10)
                            .push(text(format!("{}", config.speex_agc_target)))
                            .push(
                                widget::slider(
                                    8000..=65535,
                                    config.speex_agc_target as i32,
                                    ConfigMsg::SpeexAGCTarget,
                                )
                                .step(1),
                            )
                    }))
                    .add(
                        row()
                            .align_y(Vertical::Center)
                            .push(text(fl!("amplify")))
                            .push(horizontal_space())
                            .push(toggler(config.amplify).on_toggle(ConfigMsg::Amplify)),
                    )
                    .add_maybe((config.amplify).then(|| {
                        row()
                            .align_y(Vertical::Center)
                            .spacing(10)
                            .push(text(format!("{:.1}", config.amplify_value)))
                            .push(
                                widget::slider(
                                    0.0..=10.0,
                                    config.amplify_value,
                                    ConfigMsg::AmplifyValue,
                                )
                                .step(0.1),
                            )
                    })),
            )
            .push(
                settings::section()
                    .title("Dereverberation")
                    .add(
                        row()
                            .align_y(Vertical::Center)
                            .push(text("Enabled"))
                            .push(horizontal_space())
                            .push(
                                toggler(config.speex_dereverb_enabled)
                                    .on_toggle(ConfigMsg::SpeexDereverbEnabled),
                            ),
                    )
                    .add_maybe((config.speex_dereverb_enabled).then(|| {
                        row()
                            .align_y(Vertical::Center)
                            .spacing(10)
                            .push(text(format!("{:.1}", config.speex_dereverb_level)))
                            .push(
                                widget::slider(
                                    0.0..=1.0,
                                    config.speex_dereverb_level,
                                    ConfigMsg::SpeexDereverbLevel,
                                )
                                .step(0.1),
                            )
                    })),
            )
            .push(button::text("Reset Denoise Settings").on_press(ConfigMsg::ResetDenoiseSettings))
            .push(
                settings::section()
                    .title("App")
                    .add_maybe(if cfg!(target_os = "windows") {
                        Some(
                            row()
                                .align_y(Vertical::Center)
                                .push(text(fl!("start_at_login")))
                                .push(horizontal_space())
                                .push(
                                    toggler(config.start_at_login)
                                        .on_toggle(ConfigMsg::StartAtLogin),
                                ),
                        )
                    } else {
                        None
                    })
                    .add(
                        row()
                            .align_y(Vertical::Center)
                            .push(text(fl!("auto_connect")))
                            .push(horizontal_space())
                            .push(toggler(config.auto_connect).on_toggle(ConfigMsg::AutoConnect)),
                    )
                    .add(
                        row()
                            .align_y(Vertical::Center)
                            .push(text(fl!("theme")))
                            .push(horizontal_space())
                            .push(pick_list(
                                AppTheme::VALUES,
                                Some(&config.theme),
                                ConfigMsg::Theme,
                            )),
                    )
                    .add(
                        widget::settings::item::builder(fl!("about"))
                            .control(button::text("open").on_press(ConfigMsg::ToggleAboutWindow)),
                    ),
            ),
    )
    .into()
}

static ABOUT: LazyLock<About> = LazyLock::new(|| {
    About::default()
        .name(APP)
        .icon(widget_icon_handle!("icon"))
        .license("GPL-3.0-only")
        .links([
            (
                fl!("repository"),
                "https://github.com/teamclouday/AndroidMic",
            ),
            (
                fl!("issues_tracker"),
                "https://github.com/teamclouday/AndroidMic/issues",
            ),
        ])
        .developers([
            ("wiiznokes", "wiiznokes2@gmail.com"),
            ("teamclouday", "teamclouday@gmail.com"),
        ])
        .version(format!(
            "{}-{}",
            env!("CARGO_PKG_VERSION"),
            env!("ANDROID_MIC_COMMIT")
        ))
});

pub fn about_window() -> Element<'static, AppMsg> {
    container(widget::about(&ABOUT, AppMsg::LinkClicked))
        .padding(20)
        .into()
}
