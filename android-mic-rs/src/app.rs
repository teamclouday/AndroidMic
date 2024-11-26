use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host, HostId,
};
use tokio::sync::mpsc::Sender;

use cosmic::{
    app::{Core, Settings, Task},
    executor,
    iced::{futures::StreamExt, Subscription},
    widget::{column, dropdown, radio, text},
    Application,
};

use crate::{
    config::{Config, ConnectionMode},
    streamer::{self, Status, Streamer},
    streamer_sub::{self, StreamerCommand, StreamerMsg},
    view::view_app,
};
use zconf2::ConfigManager;

pub fn run_ui() {
    cosmic::app::run::<AppState>(Settings::default(), ()).unwrap();
}

#[derive(Debug, Clone)]
pub struct AudioHost {
    pub id: HostId,
    pub name: &'static str,
}

impl ToString for AudioHost {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

impl PartialEq for AudioHost {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
            name: device.name().unwrap_or("None".into()),
            device,
        }
    }
}

pub struct AppState {
    core: Core,
    pub audio_host: Host,
    pub audio_device: Option<cpal::Device>,
    pub audio_stream: Option<cpal::Stream>,
    pub streamer: Option<Sender<StreamerCommand>>,
    pub config: ConfigManager<Config>,
    pub status: Status,
    pub audio_hosts: Vec<AudioHost>,
    pub audio_devices: Vec<AudioDevice>,
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    ConnectionMode(ConnectionMode),
    Streamer(StreamerMsg),
    Host(HostId),
    Device(usize),
}

impl Application for AppState {
    type Executor = executor::Default;

    type Flags = ();

    type Message = AppMsg;

    const APP_ID: &'static str = "io.github.wiiznokes.android-mic";

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::app::Core,
        flags: Self::Flags,
    ) -> (Self, cosmic::app::Task<Self::Message>) {
        let config = ConfigManager::new("io.github", "wiiznokes", "android-mic").unwrap();

        let audio_host = cpal::default_host();
        let audio_hosts = cpal::available_hosts()
            .into_iter()
            .map(|id| AudioHost {
                id,
                name: id.name(),
            })
            .collect();

        let audio_device = audio_host.default_output_device();

        let audio_devices = audio_host
            .output_devices()
            .unwrap()
            .map(|d| AudioDevice::new(d))
            .collect();

        let app = Self {
            core,
            audio_stream: None,
            streamer: None,
            config,
            audio_device,
            status: Status::Default,
            audio_host,
            audio_hosts,
            audio_devices,
        };

        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            AppMsg::ConnectionMode(connection_mode) => {
                self.config.update(|config| {
                    config.connection_mode = connection_mode.clone();
                });
            }
            AppMsg::Streamer(streamer_msg) => match streamer_msg {
                StreamerMsg::Status(status) => self.status = status,
                StreamerMsg::Ready(sender) => self.streamer = Some(sender),
            },
            AppMsg::Host(host_id) => match cpal::host_from_id(host_id) {
                Ok(host) => self.audio_host = host,
                Err(e) => error!("{e}"),
            },
            AppMsg::Device(pos) => {
                self.audio_device = Some(self.audio_devices[pos].device.clone());
            }
        }

        Task::none()
    }
    fn view(&self) -> cosmic::Element<Self::Message> {
        view_app(self)
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        Subscription::run(|| streamer_sub::sub().map(AppMsg::Streamer))
    }
}
