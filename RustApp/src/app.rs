use std::{
    fmt::{Debug, Display},
    path::Path,
};

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
    config::{AudioFormat, ChannelCount, Config, ConnectionMode, SampleRate},
    fl,
    streamer::{self, ConnectOption, Status, StreamerCommand, StreamerMsg},
    utils::{APP, APP_ID, ORG, QUALIFIER},
    view::{advanced_window, view_app, AudioWave},
};
use zconf::ConfigManager;

use directories::ProjectDirs;

pub fn run_ui() {
    cosmic::app::run::<AppState>(Settings::default(), ()).unwrap();
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

const SHARED_BUF_SIZE_S: f32 = 0.05; // 0.05s

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub audio_wave: AudioWave,
    pub state: State,
    pub advanced_window: Option<AdvancedWindow>,
    pub logs: String,
}

pub struct AdvancedWindow {
    pub window_id: window::Id,
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    ChangeConnectionMode(ConnectionMode),
    Streamer(StreamerMsg),
    Device(AudioDevice),
    Connect,
    Stop,
    AdvancedOptions,
    ChangeSampleRate(SampleRate),
    ChangeChannelCount(ChannelCount),
    ChangeAudioFormat(AudioFormat),
}

impl AppState {
    fn send_command(&self, cmd: StreamerCommand) {
        self.streamer.as_ref().unwrap().blocking_send(cmd).unwrap();
    }

    fn update_audio_stream(&mut self) {
        if self.state != State::Connected {
            return;
        }
        let (producer, consumer) = RingBuffer::<u8>::new(self.get_shared_buf_size());

        match self.start_audio_stream(consumer) {
            Ok(stream) => self.audio_stream = Some(stream),
            Err(e) => {
                error!("{e}");
                self.add_log(&e.to_string());
            }
        }

        self.send_command(StreamerCommand::ChangeBuff(producer));
    }

    fn add_log(&mut self, log: &str) {
        if !self.logs.is_empty() {
            self.logs.push('\n');
        }
        self.logs.push_str(log);
        // todo: scroll to bottom
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
            .enumerate()
            .map(|(pos, device)| AudioDevice::new(device, pos))
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
            audio_wave: AudioWave::new(),
            state: State::Default,
            advanced_window: None,
            logs: String::new(),
        };

        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let config = self.config.data();

        match message {
            AppMsg::ChangeConnectionMode(connection_mode) => {
                self.config.update(|config| {
                    config.connection_mode = connection_mode;
                });
            }
            AppMsg::Streamer(streamer_msg) => match streamer_msg {
                StreamerMsg::Status(status) => match status {
                    Status::Error(e) => {
                        self.add_log(&e);
                        self.state = State::Default;
                        self.audio_stream = None;
                        self.audio_wave.clear();
                    }
                    Status::Listening { port } => {
                        if self.state != State::Listening {
                            let port = port.unwrap_or(0);
                            info!("listening: {port:?}");
                            self.add_log(format!("Listening on port {port:?}").as_str());
                            self.state = State::Listening;
                        }
                    }
                    Status::Connected => {
                        self.state = State::Connected;
                    }
                    Status::UpdateAudioWave { data } => {
                        self.audio_wave.write_chunk(&data);
                    }
                },
                StreamerMsg::Ready(sender) => self.streamer = Some(sender),
            },
            AppMsg::Device(audio_device) => {
                self.audio_device = Some(audio_device.device.clone());
                self.config
                    .update(|c| c.device_name = Some(audio_device.name.clone()));
                self.update_audio_stream();
            }
            AppMsg::Connect => {
                self.state = State::WaitingOnStatus;
                let (producer, consumer) = RingBuffer::<u8>::new(self.get_shared_buf_size());

                let connect_option = match config.connection_mode {
                    ConnectionMode::Tcp => {
                        let ip = config.ip.unwrap_or(local_ip().unwrap());
                        self.add_log(format!("Listening on ip {ip:?}").as_str());
                        ConnectOption::Tcp { ip }
                    }
                    ConnectionMode::Udp => {
                        let ip = config.ip.unwrap_or(local_ip().unwrap());
                        self.add_log(format!("Listening on ip {ip:?}").as_str());
                        ConnectOption::Udp { ip }
                    }
                    ConnectionMode::Adb => ConnectOption::Adb,
                    ConnectionMode::Usb => ConnectOption::Usb,
                };

                self.send_command(StreamerCommand::Connect(connect_option, producer));

                match self.start_audio_stream(consumer) {
                    Ok(stream) => self.audio_stream = Some(stream),
                    Err(e) => {
                        self.add_log(&e.to_string());
                        error!("{e}")
                    }
                }
            }
            AppMsg::Stop => {
                self.send_command(StreamerCommand::Stop);
                self.state = State::Default;
                self.audio_stream = None;
                self.audio_wave.clear();
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
                        size: Size::new(500.0, 700.0),
                        resizable: false,
                        ..Default::default()
                    };

                    let (new_id, command) = cosmic::iced::runtime::window::open(settings);
                    self.advanced_window = Some(AdvancedWindow { window_id: new_id });
                    return command.map(|_| cosmic::app::Message::None);
                }
            },
            AppMsg::ChangeSampleRate(sample_rate) => {
                self.config.update(|s| s.sample_rate = sample_rate);
                self.update_audio_stream();
            }
            AppMsg::ChangeChannelCount(channel_count) => {
                self.config.update(|s| s.channel_count = channel_count);
                self.update_audio_stream();
            }
            AppMsg::ChangeAudioFormat(audio_format) => {
                self.config.update(|s| s.audio_format = audio_format);
                self.update_audio_stream();
            }
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

        cosmic::widget::text(format!("no view for window {id:?}")).into()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        Subscription::run(|| streamer::sub().map(AppMsg::Streamer))
    }
}
