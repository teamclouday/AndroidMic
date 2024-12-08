use cosmic::{
    iced::{
        alignment::Horizontal, mouse, widget::pick_list, Color, Length, Point, Rectangle, Renderer,
        Size,
    },
    widget::{
        button, canvas, column, container, horizontal_space, radio, row, scrollable, settings,
        text, vertical_space,
    },
    Element,
};
use cpal::traits::DeviceTrait;

use crate::{
    app::{AdvancedWindow, AppMsg, AppState, State},
    config::{AudioFormat, ChannelCount, ConnectionMode, SampleRate},
    fl,
};

pub fn view_app(app: &AppState) -> Element<'_, AppMsg> {
    row()
        .padding(50)
        .push(
            column()
                .push(logs(app))
                .push(vertical_space())
                .push(wave(app)),
        )
        .push(horizontal_space())
        .push(
            column()
                .align_x(Horizontal::Center)
                .push(audio(app))
                .push(vertical_space())
                .push(connection_type(app)),
        )
        .into()
}

fn logs(app: &AppState) -> Element<'_, AppMsg> {
    container(scrollable(text(&app.logs).width(Length::Fill)))
        .width(Length::FillPortion(2))
        .height(Length::FillPortion(2))
        .padding(13)
        .class(cosmic::theme::Container::Card)
        .into()
}

fn wave(app: &AppState) -> Element<'_, AppMsg> {
    container(canvas(&app.audio_wave).width(Length::Fill))
        .width(Length::FillPortion(2))
        .height(Length::FillPortion(1))
        .padding(13)
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
        .push(pick_list(
            app.audio_devices.clone(),
            selected,
            AppMsg::Device,
        ))
        .push(button::text(fl!("advanced")).on_press(AppMsg::AdvancedOptions))
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
                .push(radio(
                    "USB Serial",
                    &ConnectionMode::Usb,
                    Some(connection_mode),
                    |mode| AppMsg::ChangeConnectionMode(*mode),
                ))
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
    match app.state {
        State::Default => button::text(fl!("connect")).on_press(AppMsg::Connect),
        State::Listening => button::text(fl!("listening")).on_press(AppMsg::Stop),
        State::Connected => button::destructive(fl!("disconnect")).on_press(AppMsg::Stop),
        State::WaitingOnStatus => button::text(fl!("waiting")),
    }
    .into()
}

pub fn advanced_window<'a>(
    app: &'a AppState,
    _advanced_window: &'a AdvancedWindow,
) -> Element<'a, AppMsg> {
    let config = app.config.data();

    column()
        .push(settings::section().title(fl!("sample_rate")).add(pick_list(
            SampleRate::VALUES,
            Some(&config.sample_rate),
            AppMsg::ChangeSampleRate,
        )))
        .push(
            settings::section()
                .title(fl!("channel_count"))
                .add(pick_list(
                    ChannelCount::VALUES,
                    Some(&config.channel_count),
                    AppMsg::ChangeChannelCount,
                )),
        )
        .push(
            settings::section()
                .title(fl!("audio_format"))
                .add(pick_list(
                    AudioFormat::VALUES,
                    Some(&config.audio_format),
                    AppMsg::ChangeAudioFormat,
                )),
        )
        .into()
}

#[derive(Debug)]
pub struct AudioWave {
    data: [(f32, f32); 512],
    end_index: usize,
    cache: canvas::Cache,
}

// implement for widget
impl<Message, Theme> canvas::Program<Message, Theme> for AudioWave {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // draw the wave as rectangles
        let width = bounds.width / self.data.len() as f32;
        let max_height = bounds.height / 2.0;

        let geom = self.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                canvas::Fill {
                    style: canvas::Style::Solid(Color::BLACK),
                    rule: canvas::fill::Rule::NonZero,
                },
            );

            // fill horizontal line
            frame.fill_rectangle(
                Point::new(0.0, max_height),
                Size::new(bounds.width, 1.0),
                canvas::Fill {
                    style: canvas::Style::Solid(Color::WHITE),
                    rule: canvas::fill::Rule::NonZero,
                },
            );

            for i in 0..self.data.len() {
                // remap index
                let val = self.data[(self.end_index + i) % self.data.len()];
                let x = i as f32 * width;

                let top = val.0 * 1.5; // amplify the wave
                let bottom = val.1 * 1.5;

                let height = (top - bottom).abs() * max_height;
                let y = (1.0 - top) * max_height;

                frame.fill_rectangle(
                    Point { x, y },
                    Size::new(width, height.abs()),
                    canvas::Fill {
                        style: canvas::Style::Solid(Color::WHITE),
                        rule: canvas::fill::Rule::NonZero,
                    },
                );
            }
        });

        vec![geom]
    }
}

impl AudioWave {
    pub fn new() -> Self {
        Self {
            data: [(0.0, 0.0); 512],
            end_index: 0,
            cache: canvas::Cache::default(),
        }
    }

    pub fn write(&mut self, val: (f32, f32)) {
        self.data[self.end_index] = val;
        self.end_index = (self.end_index + 1) % self.data.len();

        self.cache.clear();
    }

    pub fn write_chunk(&mut self, chunk: &Vec<(f32, f32)>) {
        for &val in chunk {
            self.write(val);
        }

        self.cache.clear();
    }

    pub fn clear(&mut self) {
        self.data = [(0.0, 0.0); 512];
        self.end_index = 0;

        self.cache.clear();
    }
}
