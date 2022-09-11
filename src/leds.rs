use core::{iter, time::Duration};

const WEST_LEDS: usize = 22;
const NORTH_LEDS: usize = 62;

pub enum Mode {
    Test,
}

pub fn led_colors(
    _mode: Mode,
    clock_value: Duration,
    strip_num: u8,
) -> impl Iterator<Item = [u8; 3]> {
    let west_leds_color = [((clock_value.as_millis() * 10) & 0xff) as u8, 0, 0];
    let north_leds_color = [0, 50, 0];

    iter::repeat(west_leds_color)
        .take(WEST_LEDS)
        .chain(iter::repeat(north_leds_color).take(NORTH_LEDS))
        .chain(iter::once([0, 0, 0]))
}
