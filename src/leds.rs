use core::{iter, time::Duration};

pub enum Mode {
    Test,
}

pub fn led_colors(
    _mode: Mode,
    clock_value: Duration,
    strip_num: u8,
) -> impl Iterator<Item = [u8; 3]> {
    [[(clock_value.as_millis() & 0xff) as u8, 0, 0], [0, 64, 0]]
        .into_iter()
        .chain(iter::repeat([0, 50, 0]).take(50))
}
