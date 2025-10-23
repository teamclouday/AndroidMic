use std::{
    fmt::{Debug, Display},
    net::{IpAddr, Ipv4Addr},
};

use cpal::{
    Device, Host,
    traits::{DeviceTrait, HostTrait},
};
use local_ip_address::list_afinet_netifas;
use notify_rust::Notification;
use rtrb::RingBuffer;
use tokio::sync::mpsc::Sender;

use cosmic::{
    Application, ApplicationExt, Element,
    app::{Core, Settings, Task},
    executor,
    iced::{Size, Subscription, futures::StreamExt, window},
    iced_widget::scrollable::{self, AbsoluteOffset},
    theme,
    widget::markdown,
};

use super::{
    message::{AppMsg, ConfigMsg},
    tray::{SystemTray, SystemTrayMsg, SystemTrayStream},
    view::{main_window, settings_window},
    wave::AudioWave,
};
use crate::{
    audio::{AudioPacketFormat, AudioProcessParams},
    config::{
        AppTheme, AudioFormat, ChannelCount, Config, ConnectionMode, NetworkAdapter, SampleRate,
    },
    fl,
    streamer::{self, ConnectOption, StreamerCommand, StreamerMsg},
    ui::view::{SCROLLABLE_ID, about_window},
    utils::APP_ID,
    window_icon,
};
use zconf::ConfigManager;

pub fn run_ui(config: ConfigManager<Config>, config_path: String, log_path: String) {
    let settings = Settings::default()
        .no_main_window(true)
        .theme(to_cosmic_theme(&config.data().theme));

    let flags = Flags {
        config,
        config_path,
        log_path,
    };

    cosmic::app::run::<AppState>(settings, flags).unwrap();
}

#[derive(Clone)]
pub struct AudioDevice {
    pub index: usize,
    pub device: Device,
    pub name: String,
}

impl Debug for AudioDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioDevice")
            .field("name", &self.name)
            .finish()
    }
}

impl Display for AudioDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for AudioDevice {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl AudioDevice {
    fn new(device: Device, index: usize) -> Self {
        Self {
            name: device.name().unwrap_or(fl!("none")),
            device,
            index,
        }
    }
}

fn get_audio_devices(audio_host: &Host) -> Vec<AudioDevice> {
    audio_host
        .output_devices()
        .unwrap()
        .enumerate()
        .map(|(pos, device)| AudioDevice::new(device, pos))
        .collect()
}

const SHARED_BUF_SIZE_S: f32 = 1.; // 0.15s

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Default,
    WaitingOnStatus,
    Connected,
    Listening,
}

pub struct Stream {
    pub stream: cpal::Stream,
    /// Runtime audio PC side configuration
    pub config: AudioPacketFormat,
}

pub struct AppState {
    core: Core,
    pub streamer: Option<Sender<StreamerCommand>>,
    pub config: ConfigManager<Config>,
    pub audio_host: Host,
    pub audio_devices: Vec<AudioDevice>,
    pub audio_device: Option<cpal::Device>,
    pub audio_stream: Option<Stream>,
    pub audio_wave: AudioWave,
    pub connection_state: ConnectionState,
    pub network_adapters: Vec<NetworkAdapter>,
    pub network_adapter: Option<NetworkAdapter>,
    pub main_window: Option<CustomWindow>,
    pub settings_window: Option<CustomWindow>,
    pub about_window: Option<CustomWindow>,
    pub logs: Vec<markdown::Item>,
    log_path: String,
    pub system_tray: Option<SystemTray>,
    pub system_tray_stream: Option<SystemTrayStream>,
    has_shown_minimize_notification: bool,
}

pub struct CustomWindow {
    pub window_id: window::Id,
}

impl AppState {
    fn send_command(&self, cmd: StreamerCommand) {
        self.streamer.as_ref().unwrap().blocking_send(cmd).unwrap();
    }

    fn update_audio_stream(&mut self) -> Task<AppMsg> {
        if self.connection_state != ConnectionState::Connected {
            return Task::none();
        }
        let (producer, consumer) = RingBuffer::<u8>::new(self.get_shared_buf_size());
        let config = self.config.data().clone();

        match self.start_audio_stream(consumer) {
            Ok(audio_config) => {
                self.send_command(StreamerCommand::ReconfigureStream {
                    buff: producer,
                    audio_params: AudioProcessParams::new(audio_config, config),
                });

                Task::none()
            }
            Err(e) => {
                error!("failed to start audio stream: {e}");
                let _ = self.disconnect();
                self.add_log(&e.to_string())
            }
        }
    }

    fn add_log(&mut self, log: &str) -> Task<AppMsg> {
        self.logs.extend(markdown::parse(log));
        scrollable::scroll_to(SCROLLABLE_ID.clone(), AbsoluteOffset { x: 0., y: f32::MAX })
    }

    fn get_shared_buf_size(&self) -> usize {
        let size = ((self.config.data().sample_rate.to_number() as f32
            * self.config.data().channel_count.to_number() as f32
            * self.config.data().audio_format.sample_size() as f32)
            * SHARED_BUF_SIZE_S)
            .ceil() as usize;
        info!("shared buf size: {size}");

        size
    }

    fn connect(&mut self) -> Task<AppMsg> {
        let config = self.config.data().clone();
        let (producer, consumer) = RingBuffer::<u8>::new(self.get_shared_buf_size());

        let audio_config = match self.start_audio_stream(consumer) {
            Ok(audio_config) => audio_config,
            Err(e) => {
                error!("failed to start audio stream: {e}");
                return self.add_log(&e.to_string());
            }
        };

        let connect_options = match config.connection_mode {
            ConnectionMode::Tcp => {
                let Some(ip) = config.ip_or_default() else {
                    let e = "no address ip found";

                    error!("failed to start audio stream: {e}");
                    return self.add_log(e);
                };

                ConnectOption::Tcp { ip }
            }
            ConnectionMode::Udp => {
                let Some(ip) = config.ip_or_default() else {
                    let e = "no address ip found";

                    error!("failed to start audio stream: {e}");
                    return self.add_log(e);
                };
                ConnectOption::Udp { ip }
            }
            #[cfg(feature = "adb")]
            ConnectionMode::Adb => ConnectOption::Adb,
            #[cfg(feature = "usb")]
            ConnectionMode::Usb => ConnectOption::Usb,
        };

        self.connection_state = ConnectionState::WaitingOnStatus;

        self.send_command(StreamerCommand::Connect {
            connect_options,
            buff: producer,
            audio_params: AudioProcessParams::new(audio_config, config),
        });

        Task::none()
    }

    fn disconnect(&mut self) -> Task<AppMsg> {
        self.send_command(StreamerCommand::Stop);
        self.connection_state = ConnectionState::Default;
        self.audio_stream = None;
        self.audio_wave.clear();

        if let Some(system_tray) = self.system_tray.as_mut() {
            system_tray.update_menu_state(true, &fl!("state_disconnected"));
        }

        Task::none()
    }
}

pub struct Flags {
    config: ConfigManager<Config>,
    config_path: String,
    log_path: String,
}

// used because the markdown parsing only detect https links
const HTTPS_PREFIX_WORKAROUND: &str = "https://-file-";

const LOG_PATH_WORKAROUND: &str = constcat::concat!(HTTPS_PREFIX_WORKAROUND, "log");
const CONFIG_PATH_WORKAROUND: &str = constcat::concat!(HTTPS_PREFIX_WORKAROUND, "config");

impl Application for AppState {
    type Executor = executor::Default;

    type Flags = Flags;

    type Message = AppMsg;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::style::iced::application::appearance(
            &cosmic::theme::Theme::dark(),
        ))
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::app::Core,
        flags: Self::Flags,
    ) -> (Self, cosmic::app::Task<Self::Message>) {
        // initialize audio device
        let audio_host = cpal::default_host();
        let audio_devices = get_audio_devices(&audio_host);
        let audio_device = match &flags.config.data().device_name {
            Some(name) => {
                match audio_devices
                    .iter()
                    .find(|audio_device| &audio_device.name == name)
                {
                    Some(audio_device) => Some(audio_device.device.clone()),
                    None => {
                        warn!("can't find audio device name {}", name);
                        audio_host.default_output_device()
                    }
                }
            }
            None => audio_host.default_output_device(),
        };

        // initialize network adapter
        let network_adapters = list_afinet_netifas()
            .unwrap()
            .iter()
            .filter(|(_, ip)| ip.is_ipv4())
            .map(|(name, ip)| NetworkAdapter {
                name: name.clone(),
                ip: *ip,
            })
            .collect::<Vec<_>>();
        let network_adapter = match &flags.config.data().ip_or_default() {
            Some(ip) => match network_adapters.iter().find(|adapter| adapter.ip == *ip) {
                Some(adapter) => Some(adapter.clone()),
                None => {
                    warn!("can't find network adapter for IP {}", ip);
                    network_adapters.first().cloned()
                }
            },
            None => None,
        };

        // initialize system tray
        let (system_tray, system_tray_stream) = match SystemTray::new() {
            Ok((mut tray, stream)) => {
                tray.update_menu_state(true, &fl!("state_disconnected"));
                (Some(tray), Some(stream))
            }
            Err(e) => {
                error!("failed to create system tray: {e}");
                (None, None)
            }
        };

        // configure window settings
        let settings = window::Settings {
            size: Size::new(800.0, 600.0),
            position: window::Position::Centered,
            icon: window_icon!("icon"),
            ..Default::default()
        };

        let mut commands = Vec::new();

        let main_window = if flags.config.data().start_minimized {
            None
        } else {
            let (new_id, command) = cosmic::iced::window::open(settings);

            commands.push(command.map(|_| cosmic::action::Action::None));

            Some(CustomWindow { window_id: new_id })
        };

        let mut app = Self {
            core,
            audio_stream: None,
            streamer: None,
            config: flags.config,
            audio_device,
            audio_host,
            audio_devices,
            audio_wave: AudioWave::new(),
            connection_state: ConnectionState::Default,
            network_adapters,
            network_adapter,
            main_window,
            settings_window: None,
            about_window: None,
            logs: Vec::new(),
            log_path: flags.log_path.clone(),
            system_tray,
            system_tray_stream,
            has_shown_minimize_notification: false,
        };

        commands.push(
            app.add_log(
                format!(
                    "config path: [{}]({CONFIG_PATH_WORKAROUND})",
                    flags.config_path
                )
                .as_str(),
            ),
        );
        commands.push(
            app.add_log(format!("log path: [{}]({LOG_PATH_WORKAROUND})", flags.log_path).as_str()),
        );
        info!("config path: {}", flags.config_path);
        info!("log path: {}", flags.log_path);

        if let Some(main_window) = &app.main_window {
            commands.push(app.set_window_title(fl!("main_window_title"), main_window.window_id));
        }

        (app, Task::batch(commands))
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let config = self.config.data();

        match message {
            AppMsg::ChangeConnectionMode(connection_mode) => {
                self.config.update(|config| {
                    config.connection_mode = connection_mode;
                });
            }
            AppMsg::RefreshAudioDevices => {
                let audio_host = cpal::default_host();
                self.audio_devices = get_audio_devices(&audio_host);
            }
            AppMsg::RefreshNetworkAdapters => {
                let network_adapters = list_afinet_netifas()
                    .unwrap()
                    .iter()
                    .filter(|(_, ip)| ip.is_ipv4())
                    .map(|(name, ip)| NetworkAdapter {
                        name: name.clone(),
                        ip: *ip,
                    })
                    .collect::<Vec<_>>();
                self.network_adapters = network_adapters;
            }
            AppMsg::Streamer(streamer_msg) => match streamer_msg {
                StreamerMsg::Error(e) => {
                    self.connection_state = ConnectionState::Default;
                    self.audio_stream = None;
                    self.audio_wave.clear();
                    return self.add_log(&e);
                }
                StreamerMsg::Listening { ip, port } => {
                    if let Some(system_tray) = self.system_tray.as_mut() {
                        system_tray.update_menu_state(false, &fl!("state_listening"));
                    }

                    if self.main_window.is_none() {
                        let address = format!(
                            "{}:{}",
                            ip.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
                            port.unwrap_or_default()
                        );
                        // show notification when app is minimized
                        let _ = Notification::new()
                            .summary("AndroidMic")
                            .body(format!("Listening on {address}").as_str())
                            .auto_icon()
                            .show()
                            .map_err(|e| {
                                error!("failed to show notification: {e}");
                            });
                    }

                    self.connection_state = ConnectionState::Listening;
                    if let (Some(ip), Some(port)) = (ip, port) {
                        info!("listening on {ip}:{port}");
                        return self.add_log(format!("Listening on `{ip}:{port}`").as_str());
                    }
                }
                StreamerMsg::Connected { ip, port, mode } => {
                    if let Some(system_tray) = self.system_tray.as_mut() {
                        system_tray.update_menu_state(false, &fl!("state_connected"));
                    }

                    if self.main_window.is_none() {
                        let address = format!(
                            "{}:{}",
                            ip.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
                            port.unwrap_or_default()
                        );

                        if mode != ConnectionMode::Udp {
                            // show notification when app is minimized
                            let _ = Notification::new()
                                .summary("AndroidMic")
                                .body(format!("Connected on {address}").as_str())
                                .auto_icon()
                                .show()
                                .map_err(|e| {
                                    error!("failed to show notification: {e}");
                                });
                        }
                    }

                    self.connection_state = ConnectionState::Connected;
                    if let (Some(ip), Some(port)) = (ip, port) {
                        info!("connected on {ip}:{port}");
                        return self.add_log(format!("Connected on `{ip}:{port}`").as_str());
                    }
                }
                StreamerMsg::UpdateAudioWave { data } => {
                    self.audio_wave.write_chunk(&data);
                }
                StreamerMsg::Ready(sender) => {
                    self.streamer = Some(sender);
                    if config.auto_connect {
                        return self.connect();
                    }
                }
            },
            AppMsg::Device(audio_device) => {
                self.audio_device = Some(audio_device.device.clone());
                self.config
                    .update(|c| c.device_name = Some(audio_device.name.clone()));
                return self.update_audio_stream();
            }
            AppMsg::Adapter(adapter) => {
                self.config.update(|c| c.ip = Some(adapter.ip));
                self.network_adapter = Some(adapter.clone());
                return self.add_log(format!("Selected network adapter: {adapter}").as_str());
            }
            AppMsg::Connect => {
                return self.connect();
            }
            AppMsg::Stop => {
                return self.disconnect();
            }
            AppMsg::ToggleSettingsWindow => match &self.settings_window {
                Some(settings_window) => {
                    let id = settings_window.window_id;
                    self.settings_window = None;
                    return cosmic::iced::runtime::task::effect(
                        cosmic::iced::runtime::Action::Window(window::Action::Close(id)),
                    );
                }
                None => {
                    let settings = window::Settings {
                        size: Size::new(500.0, 600.0),
                        position: window::Position::Centered,
                        icon: window_icon!("icon"),
                        ..Default::default()
                    };

                    let (new_id, command) = cosmic::iced::window::open(settings);
                    self.settings_window = Some(CustomWindow { window_id: new_id });
                    let set_window_title =
                        self.set_window_title(fl!("settings_window_title"), new_id);
                    return command
                        .map(|_| cosmic::action::Action::None)
                        .chain(set_window_title);
                }
            },
            AppMsg::Config(msg) => match msg {
                ConfigMsg::SampleRate(sample_rate) => {
                    self.config.update(|s| s.sample_rate = sample_rate);
                    return self.update_audio_stream();
                }
                ConfigMsg::ChannelCount(channel_count) => {
                    self.config.update(|s| s.channel_count = channel_count);
                    return self.update_audio_stream();
                }
                ConfigMsg::AudioFormat(audio_format) => {
                    self.config.update(|s| s.audio_format = audio_format);
                    return self.update_audio_stream();
                }
                ConfigMsg::StartAtLogin(start_at_login) => {
                    crate::start_at_login::start_at_login(start_at_login, &mut self.config);
                }
                ConfigMsg::AutoConnect(auto_connect) => {
                    self.config.update(|s| s.auto_connect = auto_connect);
                }
                ConfigMsg::UseRecommendedFormat => {
                    if let Some(device) = &self.audio_device
                        && let Ok(format) = device.default_output_config()
                    {
                        info!(
                            "using recommended audio format: sample rate = {}, channels = {}, audio format = {}",
                            format.sample_rate().0,
                            format.channels(),
                            format.sample_format()
                        );
                        self.config.update(|s| {
                            s.sample_rate =
                                SampleRate::from_number(format.sample_rate().0).unwrap_or_default();
                            s.channel_count =
                                ChannelCount::from_number(format.channels()).unwrap_or_default();
                            s.audio_format = AudioFormat::from_cpal_format(format.sample_format())
                                .unwrap_or_default();
                        });
                        return self.update_audio_stream();
                    }
                }
                ConfigMsg::ResetDenoiseSettings => {
                    self.config.update(|c| c.reset_denoise_settings());
                }
                ConfigMsg::DeNoise(denoise) => {
                    self.config.update(|c| c.denoise = denoise);
                    return self.update_audio_stream();
                }
                ConfigMsg::Theme(app_theme) => {
                    let cmd = cosmic::command::set_theme(to_cosmic_theme(&app_theme));
                    self.config.update(|s| s.theme = app_theme);
                    return cmd;
                }
                ConfigMsg::ToggleAboutWindow => match &self.about_window {
                    Some(about_window) => {
                        let id = about_window.window_id;
                        self.about_window = None;
                        return cosmic::iced::runtime::task::effect(
                            cosmic::iced::runtime::Action::Window(window::Action::Close(id)),
                        );
                    }
                    None => {
                        let settings = window::Settings {
                            size: Size::new(500.0, 600.0),
                            position: window::Position::Centered,
                            icon: window_icon!("icon"),
                            ..Default::default()
                        };

                        let (new_id, command) = cosmic::iced::window::open(settings);
                        self.about_window = Some(CustomWindow { window_id: new_id });
                        let set_window_title = self.set_window_title(fl!("about"), new_id);
                        return command
                            .map(|_| cosmic::action::Action::None)
                            .chain(set_window_title);
                    }
                },
                ConfigMsg::Amplify(amplify) => {
                    self.config.update(|c| c.amplify = amplify);
                    return self.update_audio_stream();
                }
                ConfigMsg::AmplifyValue(amplify_value) => {
                    self.config.update(|c| c.amplify_value = amplify_value);
                    return self.update_audio_stream();
                }
                ConfigMsg::DeNoiseKind(denoise_kind) => {
                    self.config.update(|c| c.denoise_kind = denoise_kind);
                    return self.update_audio_stream();
                }
                ConfigMsg::SpeexNoiseSuppress(speex_noise_suppress) => {
                    self.config
                        .update(|c| c.speex_noise_suppress = speex_noise_suppress);
                    return self.update_audio_stream();
                }
                ConfigMsg::SpeexVADEnabled(speex_vad_enabled) => {
                    self.config
                        .update(|c| c.speex_vad_enabled = speex_vad_enabled);
                    return self.update_audio_stream();
                }
                ConfigMsg::SpeexVADThreshold(speex_vad_threshold) => {
                    self.config
                        .update(|c| c.speex_vad_threshold = speex_vad_threshold as u32);
                    return self.update_audio_stream();
                }
                ConfigMsg::SpeexAGCEnabled(speex_agc_enabled) => {
                    self.config
                        .update(|c| c.speex_agc_enabled = speex_agc_enabled);
                    return self.update_audio_stream();
                }
                ConfigMsg::SpeexAGCTarget(speex_agc_target) => {
                    self.config
                        .update(|c| c.speex_agc_target = speex_agc_target as u32);
                    return self.update_audio_stream();
                }
                ConfigMsg::SpeexDereverbEnabled(speex_dereverb_enabled) => {
                    self.config
                        .update(|c| c.speex_dereverb_enabled = speex_dereverb_enabled);
                    return self.update_audio_stream();
                }
                ConfigMsg::SpeexDereverbLevel(speex_dereverb_level) => {
                    self.config
                        .update(|c| c.speex_dereverb_level = speex_dereverb_level);
                    return self.update_audio_stream();
                }
                ConfigMsg::StartMinimized(start_minimized) => {
                    self.config.update(|s| s.start_minimized = start_minimized);
                }
            },
            AppMsg::HideWindow => {
                let mut effects = Vec::new();

                if let Some(main_window) = &self.main_window {
                    effects.push(cosmic::iced_runtime::task::effect(
                        cosmic::iced::runtime::Action::Window(window::Action::Close(
                            main_window.window_id,
                        )),
                    ));
                    self.main_window = None;
                }
                if let Some(settings_window) = &self.settings_window {
                    effects.push(cosmic::iced_runtime::task::effect(
                        cosmic::iced::runtime::Action::Window(window::Action::Close(
                            settings_window.window_id,
                        )),
                    ));
                    self.settings_window = None;
                }
                if let Some(about_window) = &self.about_window {
                    effects.push(cosmic::iced_runtime::task::effect(
                        cosmic::iced::runtime::Action::Window(window::Action::Close(
                            about_window.window_id,
                        )),
                    ));
                    self.about_window = None;
                }

                if !self.config.data().start_minimized && !self.has_shown_minimize_notification {
                    let _ = Notification::new()
                        .summary("AndroidMic")
                        .body(&fl!("minimized_to_tray"))
                        .auto_icon()
                        .show()
                        .map_err(|e| {
                            error!("failed to show notification: {e}");
                        });
                    self.has_shown_minimize_notification = true;
                }

                return cosmic::iced_runtime::Task::batch(effects);
            }
            AppMsg::Menu(menu_msg) => match menu_msg {
                super::message::MenuMsg::ClearLogs => self.logs.clear(),
            },
            AppMsg::LinkClicked(mut url) => {
                if url.starts_with(CONFIG_PATH_WORKAROUND) {
                    url = self.config.path().to_str().unwrap_or_default().to_string();
                }

                if url.starts_with(LOG_PATH_WORKAROUND) {
                    url = self.log_path.clone();
                }

                info!("open: {url}");

                if let Err(e) = open::that(url) {
                    error!("{e}");
                }
            }
            AppMsg::SystemTray(tray_msg) => match tray_msg {
                SystemTrayMsg::Show => {
                    if let Some(main_window) = &self.main_window {
                        // avoid duplicate window
                        return cosmic::iced_runtime::task::effect(
                            cosmic::iced::runtime::Action::Window(window::Action::GainFocus(
                                main_window.window_id,
                            )),
                        );
                    } else {
                        let settings = window::Settings {
                            size: Size::new(800.0, 600.0),
                            position: window::Position::Centered,
                            icon: window_icon!("icon"),
                            ..Default::default()
                        };

                        let (new_id, command) = cosmic::iced::window::open(settings);
                        self.main_window = Some(CustomWindow { window_id: new_id });
                        let set_window_title =
                            self.set_window_title(fl!("main_window_title"), new_id);

                        return command
                            .map(|_| cosmic::action::Action::None)
                            .chain(set_window_title)
                            .chain(cosmic::iced_runtime::task::effect(
                                cosmic::iced::runtime::Action::Window(window::Action::GainFocus(
                                    new_id,
                                )),
                            ));
                    }
                }
                SystemTrayMsg::Exit => {
                    return cosmic::iced_runtime::task::effect(cosmic::iced::runtime::Action::Exit);
                }
                SystemTrayMsg::Connect => return self.connect(),
                SystemTrayMsg::Disconnect => return self.disconnect(),
            },
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.view_window(self.core.main_window_id().unwrap())
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Self::Message> {
        if let Some(window) = &self.settings_window
            && window.window_id == id
        {
            return settings_window(self).map(AppMsg::Config);
        }
        if let Some(window) = &self.main_window
            && window.window_id == id
        {
            return main_window(self);
        }

        if let Some(window) = &self.about_window
            && window.window_id == id
        {
            return about_window();
        }

        cosmic::widget::text(format!("no view for window {id:?}")).into()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        let mut subscriptions = vec![Subscription::run(|| streamer::sub().map(AppMsg::Streamer))];

        if let Some(system_tray_stream) = &self.system_tray_stream {
            subscriptions.push(Subscription::run_with_id(
                "system-tray",
                system_tray_stream.clone().sub().map(AppMsg::SystemTray),
            ));
        }

        Subscription::batch(subscriptions)
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        if let Some(window) = &self.settings_window
            && window.window_id == id
        {
            return Some(AppMsg::ToggleSettingsWindow);
        }
        if let Some(window) = &self.about_window
            && window.window_id == id
        {
            return Some(AppMsg::Config(ConfigMsg::ToggleAboutWindow));
        }
        if let Some(window) = &self.main_window
            && window.window_id == id
        {
            // close the app
            return Some(AppMsg::HideWindow);
        }

        None
    }
}

fn to_cosmic_theme(theme: &AppTheme) -> theme::Theme {
    match theme {
        AppTheme::Dark => theme::Theme::dark(),
        AppTheme::Light => theme::Theme::light(),
        AppTheme::HighContrastDark => theme::Theme::dark_hc(),
        AppTheme::HighContrastLight => theme::Theme::light_hc(),
        AppTheme::System => theme::system_preference(),
    }
}
