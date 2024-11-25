use cosmic::{
    app::{Core, Settings, Task},
    executor,
    widget::text,
    Application,
};

pub fn run_ui() {

    cosmic::app::run::<App>(Settings::default(), ()).unwrap();

}

struct App {
    core: Core,
}

#[derive(Debug, Clone)]
enum Message {}

impl Application for App {
    type Executor = executor::Default;

    type Flags = ();

    type Message = Message;

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
        (Self { core }, Task::none())
    }

    fn view(&self) -> cosmic::Element<Self::Message> {
        text("hello").into()
    }
}
