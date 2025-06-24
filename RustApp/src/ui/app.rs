use std::fmt::{Debug, Display};

use cpal::{
    Device, Host,
    traits::{DeviceTrait, HostTrait},
};
use local_ip_address::local_ip;
use rtrb::RingBuffer;
use tokio::sync::mpsc::Sender;

use cosmic::{
    Application, ApplicationExt, Element,
    app::{Core, Settings, Task},
    executor,
    iced::{Size, Subscription, futures::StreamExt, window},
    iced_runtime::Action,
    iced_widget::scrollable::{self, AbsoluteOffset},
    theme,
};

use super::{
    message::{AppMsg, ConfigMsg},
    view::{main_window, settings_window},
    wave::AudioWave,
};
use crate::{
    audio::AudioPacketFormat,
    config::{AppTheme, AudioFormat, ChannelCount, Config, ConnectionMode, SampleRate},
    fl,
    streamer::{self, ConnectOption, Status, StreamConfig, StreamerCommand, StreamerMsg},
    ui::view::SCROLLABLE_ID,
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
    pub main_window: Option<CustomWindow>,
    pub settings_window: Option<CustomWindow>,
    pub logs: String,
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

        match self.start_audio_stream(consumer) {
            Ok(audio_config) => {
                self.send_command(StreamerCommand::ReconfigureStream(StreamConfig {
                    buff: producer,
                    audio_config,
                    denoise: self.config.data().denoise,
                }));
                Task::none()
            }
            Err(e) => {
                error!("failed to start audio stream: {e}");
                self.send_command(StreamerCommand::Stop);
                self.add_log(&e.to_string())
            }
        }
    }

    fn add_log(&mut self, log: &str) -> Task<AppMsg> {
        if !self.logs.is_empty() {
            self.logs.push('\n');
        }
        self.logs.push_str(log);

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

        let (connect_option, log) = match config.connection_mode {
            ConnectionMode::Tcp => {
                let ip = config.ip.unwrap_or(local_ip().unwrap());
                (
                    ConnectOption::Tcp { ip },
                    Some(format!("Listening on ip {ip:?}")),
                )
            }
            ConnectionMode::Udp => {
                let ip = config.ip.unwrap_or(local_ip().unwrap());
                (
                    ConnectOption::Udp { ip },
                    Some(format!("Listening on ip {ip:?}")),
                )
            }
            ConnectionMode::Adb => (ConnectOption::Adb, None),
            #[cfg(feature = "usb")]
            ConnectionMode::Usb => (ConnectOption::Usb, None),
        };

        self.connection_state = ConnectionState::WaitingOnStatus;

        self.send_command(StreamerCommand::Connect(
            connect_option,
            StreamConfig {
                buff: producer,
                audio_config,
                denoise: self.config.data().denoise,
            },
        ));

        match &log {
            Some(log) => self.add_log(log),
            None => Task::none(),
        }
    }
}

pub struct Flags {
    config: ConfigManager<Config>,
    config_path: String,
    log_path: String,
}

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
                        error!("can't find audio device name {}", name);
                        audio_host.default_output_device()
                    }
                }
            }
            None => audio_host.default_output_device(),
        };

        let settings = window::Settings {
            size: Size::new(800.0, 600.0),
            position: window::Position::Centered,
            exit_on_close_request: true,
            icon: window_icon!("icon"),
            ..Default::default()
        };

        let mut commands = Vec::new();

        let (new_id, command) = cosmic::iced::window::open(settings);

        commands.push(command.map(|_| cosmic::action::Action::None));

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
            main_window: Some(CustomWindow { window_id: new_id }),
            settings_window: None,
            logs: String::new(),
        };

        commands.push(app.add_log(format!("config path: {}", flags.config_path).as_str()));
        commands.push(app.add_log(format!("log path: {}", flags.log_path).as_str()));
        info!("config path: {}", flags.config_path);
        info!("log path: {}", flags.log_path);

        commands.push(app.set_window_title(fl!("main_window_title"), new_id));

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
            AppMsg::Streamer(streamer_msg) => match streamer_msg {
                StreamerMsg::Status(status) => match status {
                    Status::Error(e) => {
                        self.connection_state = ConnectionState::Default;
                        self.audio_stream = None;
                        self.audio_wave.clear();
                        return self.add_log(&e);
                    }
                    Status::Listening { port } => {
                        if self.connection_state != ConnectionState::Listening {
                            let port = port.unwrap_or(0);
                            info!("listening: {port:?}");
                            self.connection_state = ConnectionState::Listening;
                            return self.add_log(format!("Listening on port {port:?}").as_str());
                        }
                    }
                    Status::Connected => {
                        self.connection_state = ConnectionState::Connected;
                    }
                    Status::UpdateAudioWave { data } => {
                        self.audio_wave.write_chunk(&data);
                    }
                },
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
            AppMsg::Connect => {
                return self.connect();
            }
            AppMsg::Stop => {
                self.send_command(StreamerCommand::Stop);
                self.connection_state = ConnectionState::Default;
                self.audio_stream = None;
                self.audio_wave.clear();
            }
            AppMsg::ToggleSettingsWindow => match &self.settings_window {
                Some(settings_window) => {
                    let id = settings_window.window_id;
                    self.settings_window = None;
                    return cosmic::iced::runtime::task::effect(Action::Window(
                        window::Action::Close(id),
                    ));
                }
                None => {
                    let settings = window::Settings {
                        size: Size::new(500.0, 600.0),
                        position: window::Position::Centered,
                        exit_on_close_request: true,
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
                    if let Some(device) = &self.audio_device {
                        if let Ok(format) = device.default_output_config() {
                            info!(
                                "using recommended audio format: sample rate = {}, channels = {}, audio format = {}",
                                format.sample_rate().0,
                                format.channels(),
                                format.sample_format()
                            );
                            self.config.update(|s| {
                                s.sample_rate = SampleRate::from_number(format.sample_rate().0)
                                    .unwrap_or_default();
                                s.channel_count = ChannelCount::from_number(format.channels())
                                    .unwrap_or_default();
                                s.audio_format =
                                    AudioFormat::from_cpal_format(format.sample_format())
                                        .unwrap_or_default();
                            });
                            return self.update_audio_stream();
                        }
                    }
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
            },
            AppMsg::Shutdown => {
                return cosmic::iced_runtime::task::effect(Action::Exit);
            }
            AppMsg::Menu(menu_msg) => match menu_msg {
                super::message::MenuMsg::ClearLogs => self.logs.clear(),
            },
        }

        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.view_window(self.core.main_window_id().unwrap())
    }

    fn view_window(&self, id: window::Id) -> Element<Self::Message> {
        if let Some(window) = &self.settings_window {
            if window.window_id == id {
                return settings_window(self).map(AppMsg::Config);
            }
        }
        if let Some(window) = &self.main_window {
            if window.window_id == id {
                return main_window(self);
            }
        }

        cosmic::widget::text(format!("no view for window {id:?}")).into()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        Subscription::run(|| streamer::sub().map(AppMsg::Streamer))
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        if let Some(window) = &self.settings_window {
            if window.window_id == id {
                return Some(AppMsg::ToggleSettingsWindow);
            }
        }
        if let Some(window) = &self.main_window {
            if window.window_id == id {
                // close the app
                return Some(AppMsg::Shutdown);
            }
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
