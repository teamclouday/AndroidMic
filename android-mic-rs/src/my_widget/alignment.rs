/// ```text
/// +-----------+-----------+-----------+
/// | TopStart  |   Top     |  TopEnd   |
/// +-----------+-----------+-----------+
/// |  Start    |           |   End     |
/// +-----------+-----------+-----------+
/// |BottomStart|  Bottom   | BottomEnd |
/// +-----------+-----------+-----------+
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Alignment {
    TopStart,
    Top,
    TopEnd,

    End,

    BottomEnd,
    Bottom,
    BottomStart,

    Start,
}
