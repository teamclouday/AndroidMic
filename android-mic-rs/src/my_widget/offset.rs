use cosmic::iced_core::Point;

#[derive(Copy, Clone, Debug)]
pub struct Offset {
    /// Offset on the x-axis
    pub x: f32,
    /// Offset on the y-axis
    pub y: f32,
}

impl Offset {
    pub fn new(x: f32, y: f32) -> Self {
        Offset { x, y }
    }
}

impl From<f32> for Offset {
    fn from(float: f32) -> Self {
        Self { x: float, y: float }
    }
}

impl From<[f32; 2]> for Offset {
    fn from(array: [f32; 2]) -> Self {
        Self {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Offset> for Point {
    fn from(offset: Offset) -> Self {
        Self::new(offset.x, offset.y)
    }
}

impl From<&Offset> for Point {
    fn from(offset: &Offset) -> Self {
        Self::new(offset.x, offset.y)
    }
}
