use core::{iter, time::Duration};

const WEST_LEDS: usize = 22;
const NORTH_LEDS: usize = 62;
const SOUTH_LEDS: usize = 64;  // Note: it's actually 64.5, as the corner cuts it in half, a bit annoying
const EAST_LEDS: usize = 26;  // TODO: maybe not correct

pub enum Mode {
    Test,
}

pub fn led_colors(
    _mode: Mode,
    clock_value: Duration,
    strip_num: u8,
) -> impl Iterator<Item = [u8; 3]> {
    let south_leds_color = [((clock_value.as_millis() * 10) & 0xff) as u8, 0, 0];
    let east_leds_color = [0, 50, 0];

    iter::repeat(south_leds_color)
        .take(SOUTH_LEDS)
        .chain(iter::repeat(east_leds_color).take(EAST_LEDS))
        .chain(iter::once([0, 0, 0]))
}
