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
    enum ModeIter<A, B, C, D> {
        OffNw(A),
        OffSe(B),
        TestNw(C),
        TestSe(D),
    }

    impl<
            A: Iterator<Item = [u8; 3]>,
            B: Iterator<Item = [u8; 3]>,
            C: Iterator<Item = [u8; 3]>,
            D: Iterator<Item = [u8; 3]>,
        > Iterator for ModeIter<A, B, C, D>
    {
        type Item = [u8; 3];

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                ModeIter::OffNw(i) => i.next(),
                ModeIter::OffSe(i) => i.next(),
                ModeIter::TestNw(i) => i.next(),
                ModeIter::TestSe(i) => i.next(),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match self {
                ModeIter::OffNw(i) => i.size_hint(),
                ModeIter::OffSe(i) => i.size_hint(),
                ModeIter::TestNw(i) => i.size_hint(),
                ModeIter::TestSe(i) => i.size_hint(),
            }
        }
    }

    match (mode, strip) {
        (Mode::Off, Strip::NorthWest) => ModeIter::OffNw(off_mode_colors_nw()),
        (Mode::Off, Strip::SouthEast) => ModeIter::OffSe(off_mode_colors_se()),
        (Mode::Test, Strip::NorthWest) => ModeIter::TestNw(test_mode_colors_nw(clock_value)),
        (Mode::Test, Strip::SouthEast) => ModeIter::TestSe(test_mode_colors_se(clock_value)),
    }
}

fn test_mode_colors_nw(clock_value: Duration) -> impl Iterator<Item = [u8; 3]> {
    let mut n = 0u8;
    iter::repeat([128, 128, 0]).take(EAST_LEDS).chain(
        iter::from_fn(move || {
            n += 1;
            let intensity = (128 * u16::from(n) / u16::try_from(NORTH_LEDS).unwrap()) as u8;
            Some([intensity, intensity / 2, 0])
        })
        .take(NORTH_LEDS),
    )
}

fn test_mode_colors_se(clock_value: Duration) -> impl Iterator<Item = [u8; 3]> {
    let mut n = 0u8;
    iter::from_fn(move || {
        n += 1;
        let intensity = (128 * u16::from(n) / u16::try_from(SOUTH_LEDS).unwrap()) as u8;
        Some([intensity, intensity / 2, 0])
    })
    .take(SOUTH_LEDS)
    .chain(iter::repeat([0, 0, 0]).take(EAST_LEDS))
}

fn off_mode_colors_nw() -> impl Iterator<Item = [u8; 3]> {
    iter::repeat([0, 0, 0]).take(NORTH_LEDS + WEST_LEDS)
}

fn off_mode_colors_se() -> impl Iterator<Item = [u8; 3]> {
    iter::repeat([0, 0, 0]).take(SOUTH_LEDS + EAST_LEDS)
}
