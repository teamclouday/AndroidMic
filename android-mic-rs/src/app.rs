use tokio::sync::mpsc::Sender;

use cosmic::{
    app::{Core, Settings, Task},
    executor,
    iced::{futures::StreamExt, Subscription},
    widget::{column, radio, text},
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

pub struct AppState {
    core: Core,
    audio_device: Option<cpal::Device>,
    audio_stream: Option<cpal::Stream>,
    streamer: Option<Sender<StreamerCommand>>,
    pub config: ConfigManager<Config>,
    status: Status,
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    ConnectionMode(ConnectionMode),
    Streamer(StreamerMsg),
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

        let app = Self {
            core,
            audio_stream: None,
            streamer: None,
            config,
            audio_device: None,
            status: Status::Default,
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
