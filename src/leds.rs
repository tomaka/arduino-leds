use core::{iter, time::Duration};

const WEST_LEDS: usize = 22;
const NORTH_LEDS: usize = 62;
const SOUTH_LEDS: usize = 64; // Note: it's actually 64.5, as the corner cuts it in half, a bit annoying
const EAST_LEDS: usize = 25;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Off,
    Test,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Strip {
    NorthWest,
    SouthEast,
}

pub fn led_colors(
    mode: Mode,
    clock_value: Duration,
    strip: Strip,
) -> impl Iterator<Item = [u8; 3]> {
    enum ModeIter<A, B> {
        Off(A),
        Test(B),
    }

    impl<A: Iterator<Item = [u8; 3]>, B: Iterator<Item = [u8; 3]>> Iterator for ModeIter<A, B> {
        type Item = [u8; 3];

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                ModeIter::Off(i) => i.next(),
                ModeIter::Test(i) => i.next(),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match self {
                ModeIter::Off(i) => i.size_hint(),
                ModeIter::Test(i) => i.size_hint(),
            }
        }
    }

    match mode {
        Mode::Off => ModeIter::Off(off_mode_colors(strip)),
        Mode::Test => ModeIter::Test(test_mode_colors(clock_value, strip)),
    }
}

fn test_mode_colors(clock_value: Duration, nw_strip: Strip) -> impl Iterator<Item = [u8; 3]> {
    let south_leds_color = [((clock_value.as_millis() * 10) & 0xff) as u8 / 16, 0, 0];
    let east_leds_color = [0, 50 / 4, 0];

    let mut n = 0u8;
    iter::from_fn(move || {
        n += 1;
        let intensity = (128 * u16::from(n) / u16::try_from(SOUTH_LEDS).unwrap()) as u8;
        Some([intensity, intensity / 2, 0])
    })
    .take(SOUTH_LEDS)
    .chain(iter::repeat([0, 0, 0]).take(EAST_LEDS))
}

fn off_mode_colors(strip: Strip) -> impl Iterator<Item = [u8; 3]> {
    let count = match strip {
        Strip::NorthWest => NORTH_LEDS + WEST_LEDS,
        Strip::SouthEast => SOUTH_LEDS + EAST_LEDS,
    };

    iter::repeat([0, 0, 0]).take(count)
}
