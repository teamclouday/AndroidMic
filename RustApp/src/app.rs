use std::path::Path;

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host,
};
use local_ip_address::local_ip;
use rtrb::RingBuffer;
use tokio::sync::mpsc::Sender;

use cosmic::{
    app::{Core, Settings, Task},
    executor,
    iced::{futures::StreamExt, window, Size, Subscription},
    iced_runtime::Action,
    Application, Element,
};

use crate::{
    config::{Config, ConnectionMode},
    fl,
    streamer::{self, ConnectOption, Status, StreamerCommand, StreamerMsg},
    utils::{APP, APP_ID, ORG, QUALIFIER},
    view::{advanced_window, view_app},
};
use zconf::ConfigManager;

use directories::ProjectDirs;

pub fn run_ui() {
    cosmic::app::run::<AppState>(Settings::default(), ()).unwrap();
}

#[derive(Clone)]
pub struct AudioDevice {
    pub device: Device,
    pub name: String,
}

impl AsRef<str> for AudioDevice {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl AudioDevice {
    fn new(device: Device) -> Self {
        Self {
            name: device.name().unwrap_or(fl!("none")),
            device,
        }
    }
}

const SHARED_BUF_SIZE: usize = 5 * 1024;

pub enum State {
    Default,
    WaitingOnStatus,
    Connected,
    Listening,
}

pub struct AppState {
    core: Core,
    pub streamer: Option<Sender<StreamerCommand>>,
    pub config: ConfigManager<Config>,
    pub audio_host: Host,
    pub audio_devices: Vec<AudioDevice>,
    pub audio_device: Option<cpal::Device>,
    pub audio_stream: Option<cpal::Stream>,
    pub state: State,
    pub advanced_window: Option<AdvancedWindow>,
}

pub struct AdvancedWindow {
    pub window_id: window::Id,
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    ChangeConnectionMode(ConnectionMode),
    Streamer(StreamerMsg),
    Device(usize),
    Connect,
    Stop,
    AdvancedOptions,
}

impl AppState {
    fn send_command(&self, cmd: StreamerCommand) {
        self.streamer.as_ref().unwrap().blocking_send(cmd).unwrap();
    }

    fn update_audio_buf(&mut self) {
        let (producer, consumer) = RingBuffer::<u8>::new(SHARED_BUF_SIZE);

        match self.start_audio_stream(consumer) {
            Ok(stream) => self.audio_stream = Some(stream),
            Err(e) => {
                error!("{e}")
            }
        }

        self.send_command(StreamerCommand::ChangeBuff(producer));
    }
}

impl Application for AppState {
    type Executor = executor::Default;

    type Flags = ();

    type Message = AppMsg;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::app::Core,
        _flags: Self::Flags,
    ) -> (Self, cosmic::app::Task<Self::Message>) {
        let project_dirs = ProjectDirs::from(QUALIFIER, ORG, APP).unwrap();

        let config_path = if cfg!(debug_assertions) {
            Path::new("config")
        } else {
            project_dirs.config_dir()
        };

        let config: ConfigManager<Config> =
            ConfigManager::new(config_path.join(format!("{APP}.toml")));

        let audio_host = cpal::default_host();

        let audio_devices = audio_host
            .output_devices()
            .unwrap()
            .map(AudioDevice::new)
            .collect::<Vec<_>>();

        let audio_device = match &config.data().device_name {
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

        let app = Self {
            core,
            audio_stream: None,
            streamer: None,
            config,
            audio_device,
            audio_host,
            audio_devices,
            state: State::Default,
            advanced_window: None,
        };

        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let settings = self.config.data();

        match message {
            AppMsg::ChangeConnectionMode(connection_mode) => {
                self.config.update(|config| {
                    config.connection_mode = connection_mode;
                });
            }
            AppMsg::Streamer(streamer_msg) => match streamer_msg {
                StreamerMsg::Status(status) => match status {
                    Status::Error(_e) => {
                        self.state = State::Default;
                        self.audio_stream = None;
                    }
                    Status::Listening { port } => {
                        info!("listening: {port:?}");
                        self.state = State::Listening;
                    }
                    Status::Connected => {
                        self.state = State::Connected;
                    }
                },
                StreamerMsg::Ready(sender) => self.streamer = Some(sender),
            },
            AppMsg::Device(pos) => {
                let audio_device = &self.audio_devices[pos];

                self.audio_device = Some(audio_device.device.clone());
                self.config
                    .update(|c| c.device_name = Some(audio_device.name.clone()));
                self.update_audio_buf();
            }
            AppMsg::Connect => {
                self.state = State::WaitingOnStatus;
                let (producer, consumer) = RingBuffer::<u8>::new(SHARED_BUF_SIZE);

                let connect_option = match settings.connection_mode {
                    ConnectionMode::Tcp => ConnectOption::Tcp {
                        ip: settings.ip.unwrap_or(local_ip().unwrap()),
                    },
                    ConnectionMode::Udp => ConnectOption::Udp {
                        ip: settings.ip.unwrap_or(local_ip().unwrap()),
                    },
                    ConnectionMode::Adb => ConnectOption::Adb,
                };

                self.send_command(StreamerCommand::Connect(connect_option, producer));

                match self.start_audio_stream(consumer) {
                    Ok(stream) => self.audio_stream = Some(stream),
                    Err(e) => {
                        error!("{e}")
                    }
                }
            }
            AppMsg::Stop => {
                self.send_command(StreamerCommand::Stop);
                self.state = State::Default;
                self.audio_stream = None;
            }
            AppMsg::AdvancedOptions => match &self.advanced_window {
                Some(advanced_window) => {
                    let id = advanced_window.window_id;
                    self.advanced_window = None;
                    return cosmic::iced::runtime::task::effect(Action::Window(
                        window::Action::Close(id),
                    ));
                }
                None => {
                    let settings = window::Settings {
                        size: Size::new(300.0, 200.0),
                        resizable: false,
                        ..Default::default()
                    };

                    let (new_id, command) = cosmic::iced::runtime::window::open(settings);
                    self.advanced_window = Some(AdvancedWindow { window_id: new_id });
                    return command.map(|_| cosmic::app::Message::None);
                }
            },
        }

        Task::none()
    }
    fn view(&self) -> Element<Self::Message> {
        view_app(self)
    }

    fn view_window(&self, id: window::Id) -> Element<Self::Message> {
        if let Some(window) = &self.advanced_window {
            if window.window_id == id {
                return advanced_window(self, window);
            }
        }

        cosmic::widget::text("no view for window {id:?}").into()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        Subscription::run(|| streamer::sub().map(AppMsg::Streamer))
    }
}
