use cosmic::{
    app::{Core, Settings, Task},
    executor,
    widget::{column, radio, text},
    Application,
};

use crate::{config::{Config, ConnectionMode}, streamer::Streamer, view::view_app};
use zconf2::ConfigManager;

pub fn run_ui() {

    cosmic::app::run::<AppState>(Settings::default(), ()).unwrap();

}

pub struct AppState {
    core: Core,
    audio_player: Option<cpal::Stream>,
     streamer: Option<Box<dyn Streamer>>,
    pub config: ConfigManager<Config>,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    ConnectionMode(ConnectionMode),
}

impl Application for AppState {
    type Executor = executor::Default;

    type Flags = ();

    type Message = AppMessage;

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
            audio_player: None,
            streamer: None,
            config
        };
        
        (app, Task::none())
    }
    
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        
        match message {
            AppMessage::ConnectionMode(connection_mode) => {
                self.config.update(|config| {
                    config.connection_mode = connection_mode.clone();
                });
            },
        }

        Task::none()
    }
    fn view(&self) -> cosmic::Element<Self::Message> {

        view_app(self)
        
    }
}
