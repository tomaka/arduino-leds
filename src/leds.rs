use core::{cmp, iter, time::Duration};

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

pub fn led_colors_lerp(
    mode1: Mode,
    mode2: Mode,
    since_mode_change: Duration,
    clock_value: Duration,
    strip: Strip,
) -> impl Iterator<Item = [u8; 3]> {
    const LERP_DURATION_MS: u32 = 1000;

    let mode2_weight = u8::try_from(cmp::min(
        u32::try_from(since_mode_change.as_millis())
            .unwrap_or(u32::max_value())
            .saturating_mul(LERP_DURATION_MS)
            / 1000,
        255,
    ))
    .unwrap();

    let colors1 = led_colors(mode1, clock_value, strip);
    let colors2 = led_colors(mode2, clock_value, strip);

    colors1.zip(colors2).map(move |(c1, c2)| {
        fn avg(a: u8, b: u8, b_weight: u8) -> u8 {
            // TODO: not correct, as the weight would need to be 256
            let res = u16::from(a)
                * u16::from(255 - b_weight).saturating_add(u16::from(b) * u16::from(b_weight));
            u8::try_from(res >> 8).unwrap()
        }
        [
            avg(c1[0], c2[0], mode2_weight),
            avg(c1[1], c2[1], mode2_weight),
            avg(c1[2], c2[2], mode2_weight),
        ]
    })
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
        (Mode::Off, Strip::NorthWest) => {
            ModeIter::OffNw(iter::repeat([0, 0, 0]).take(NORTH_LEDS + WEST_LEDS))
        }
        (Mode::Off, Strip::SouthEast) => {
            ModeIter::OffSe(iter::repeat([0, 0, 0]).take(SOUTH_LEDS + EAST_LEDS))
        }
        (Mode::Test, Strip::NorthWest) => ModeIter::TestNw(cursor_add_nw(
            clock_value,
            west_to_east_gradiant_modifier_nw(random_waves_modifier(
                clock_value,
                iter::repeat([128, 64, 0]).take(NORTH_LEDS + WEST_LEDS),
            )),
        )),
        (Mode::Test, Strip::SouthEast) => ModeIter::TestSe(cursor_add_se(
            clock_value,
            west_to_east_gradiant_modifier_se(
                iter::repeat([128, 64, 0]).take(SOUTH_LEDS + EAST_LEDS),
            ),
        )),
    }
}

fn west_to_east_gradiant_modifier_nw(
    iter: impl Iterator<Item = [u8; 3]>,
) -> impl Iterator<Item = [u8; 3]> {
    let mut n = 0;
    iter.map(move |item| {
        let intensity = if n < WEST_LEDS {
            16
        } else {
            16 + (256 - 16) * u16::try_from(n - WEST_LEDS).unwrap()
                / u16::try_from(NORTH_LEDS).unwrap()
        };

        n += 1;

        [
            (intensity * u16::from(item[0]) / 256) as u8,
            (intensity * u16::from(item[1]) / 256) as u8,
            (intensity * u16::from(item[2]) / 256) as u8,
        ]
    })
}

fn west_to_east_gradiant_modifier_se(
    iter: impl Iterator<Item = [u8; 3]>,
) -> impl Iterator<Item = [u8; 3]> {
    let mut n = 0;
    iter.map(move |item| {
        let intensity = if n < SOUTH_LEDS {
            16 + (256 - 16) * u16::try_from(n).unwrap() / u16::try_from(SOUTH_LEDS).unwrap()
        } else {
            256
        };

        n += 1;

        [
            (intensity * u16::from(item[0]) / 256) as u8,
            (intensity * u16::from(item[1]) / 256) as u8,
            (intensity * u16::from(item[2]) / 256) as u8,
        ]
    })
}

fn random_waves_modifier(
    clock_value: Duration,
    iter: impl Iterator<Item = [u8; 3]>,
) -> impl Iterator<Item = [u8; 3]> {
    // TODO: dummy
    iter.map(move |n| n)
}

fn cursor_add_nw(
    clock_value: Duration,
    iter: impl Iterator<Item = [u8; 3]>,
) -> impl Iterator<Item = [u8; 3]> {
    let cursor_pos = (((clock_value.as_millis() / 500) as u32)
        % u32::try_from(WEST_LEDS + NORTH_LEDS + EAST_LEDS + SOUTH_LEDS).unwrap())
        as u16;
    iter.enumerate().map(move |(pos, value)| {
        if pos == usize::from(cursor_pos) {
            [255, 0, 0]
        } else {
            value
        }
    })
}

fn cursor_add_se(
    clock_value: Duration,
    iter: impl Iterator<Item = [u8; 3]>,
) -> impl Iterator<Item = [u8; 3]> {
    let cursor_pos = (((clock_value.as_millis() / 500) as u32)
        % u32::try_from(WEST_LEDS + NORTH_LEDS + EAST_LEDS + SOUTH_LEDS).unwrap())
        as u16;
    let cursor_pos_adj = cursor_pos
        .checked_sub((WEST_LEDS + NORTH_LEDS) as u16)
        .map(|n| (SOUTH_LEDS + EAST_LEDS) as u16 - n)
        .unwrap_or(u16::max_value());
    iter.enumerate().map(move |(pos, value)| {
        if pos == usize::from(cursor_pos_adj) {
            [255, 0, 0]
        } else {
            value
        }
    })
}
