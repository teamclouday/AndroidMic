use cosmic::{
    iced::{Color, Point, Rectangle, Renderer, Size, mouse},
    widget::canvas,
};

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
