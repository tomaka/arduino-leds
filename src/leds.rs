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
) -> impl Iterator<Item = [u8; 3]> + Clone {
    const LERP_DURATION_MS: u32 = 1000;

    let mode2_weight = u8::try_from(cmp::min(
        u32::try_from(since_mode_change.as_millis())
            .unwrap_or(u32::max_value())
            .saturating_mul(255)
            / LERP_DURATION_MS,
        255,
    ))
    .unwrap();

    let colors1 = led_colors(mode1, clock_value, strip);
    let colors2 = led_colors(mode2, clock_value, strip);

    colors1.zip(colors2).map(move |(c1, c2)| {
        fn avg(a: u8, b: u8, b_weight: u8) -> u8 {
            let res = u16::from(a) * u16::from(255 - b_weight) + u16::from(b) * u16::from(b_weight);
            u8::try_from(res / 255).unwrap()
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
) -> impl Iterator<Item = [u8; 3]> + Clone {
    #[derive(Clone)]
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
        (Mode::Test, Strip::NorthWest) => ModeIter::TestNw(seemingly_random_vibration_nw(
            clock_value,
            iter::repeat([108, 50, 0]).take(NORTH_LEDS + WEST_LEDS),
        )),
        (Mode::Test, Strip::SouthEast) => ModeIter::TestSe(seemingly_random_vibration_se(
            clock_value,
            iter::repeat([108, 50, 0]).take(SOUTH_LEDS + EAST_LEDS),
        )),
    }
}

fn west_to_east_gradiant_modifier_nw(
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
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
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
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

fn seemingly_random_vibration_nw(
    clock_value: Duration,
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    let wave1_add = (((clock_value.as_millis() as u32) / 600) & 0xff) as u8;
    let wave2_add = (((clock_value.as_millis() as u32) / 3000) & 0xff) as u8;
    let wave3_add = (((clock_value.as_millis() as u32) / 2100) & 0xff) as u8;
    let wave4_add = (((clock_value.as_millis() as u32) / 2200) & 0xff) as u8;

    iter.enumerate().map(move |(idx, val)| {
        let led_pos = idx as u32;
        let angle1 = u32::from(wave1_add)
            + 5 * 256 * led_pos / (NORTH_LEDS + WEST_LEDS + SOUTH_LEDS + EAST_LEDS) as u32;
        let sin_value1 = i16::from(sin_approx((angle1 & 0xff) as u8));
        let angle2 = 3 * 256 * led_pos / (NORTH_LEDS + WEST_LEDS + SOUTH_LEDS + EAST_LEDS) as u32
            - u32::from(wave2_add);
        let sin_value2 = i16::from(sin_approx((angle2 & 0xff) as u8));
        let angle3 = u32::from(wave3_add)
            + 7 * 256 * led_pos / (NORTH_LEDS + WEST_LEDS + SOUTH_LEDS + EAST_LEDS) as u32;
        let sin_value3 = i16::from(sin_approx((angle3 & 0xff) as u8));
        let angle4 = 11 * 256 * led_pos / (NORTH_LEDS + WEST_LEDS + SOUTH_LEDS + EAST_LEDS) as u32
            - u32::from(wave4_add);
        let sin_value4 = i16::from(sin_approx((angle4 & 0xff) as u8));
        let sin_value = cmp::min(
            64,
            cmp::max(-64, sin_value1 + sin_value2 + sin_value3 + sin_value4),
        );
        let map = move |n| (i16::from(n) * (sin_value + 64) / 128) as u8;
        [map(val[0]), map(val[1]), map(val[2])]
    })
}

fn seemingly_random_vibration_se(
    clock_value: Duration,
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    let wave1_add = (((clock_value.as_millis() as u32) / 600) & 0xff) as u8;
    let wave2_add = (((clock_value.as_millis() as u32) / 3000) & 0xff) as u8;
    let wave3_add = (((clock_value.as_millis() as u32) / 2100) & 0xff) as u8;
    let wave4_add = (((clock_value.as_millis() as u32) / 2200) & 0xff) as u8;

    iter.enumerate().map(move |(idx, val)| {
        let led_pos =
            ((SOUTH_LEDS + EAST_LEDS) as u32 - idx as u32) + (NORTH_LEDS + WEST_LEDS) as u32;
        let angle1 = u32::from(wave1_add)
            + 5 * 256 * led_pos / (NORTH_LEDS + WEST_LEDS + SOUTH_LEDS + EAST_LEDS) as u32;
        let sin_value1 = i16::from(sin_approx((angle1 & 0xff) as u8));
        let angle2 = 3 * 256 * led_pos / (NORTH_LEDS + WEST_LEDS + SOUTH_LEDS + EAST_LEDS) as u32
            - u32::from(wave2_add);
        let sin_value2 = i16::from(sin_approx((angle2 & 0xff) as u8));
        let angle3 = u32::from(wave3_add)
            + 7 * 256 * led_pos / (NORTH_LEDS + WEST_LEDS + SOUTH_LEDS + EAST_LEDS) as u32;
        let sin_value3 = i16::from(sin_approx((angle3 & 0xff) as u8));
        let angle4 = 11 * 256 * led_pos / (NORTH_LEDS + WEST_LEDS + SOUTH_LEDS + EAST_LEDS) as u32
            - u32::from(wave4_add);
        let sin_value4 = i16::from(sin_approx((angle4 & 0xff) as u8));
        let sin_value = cmp::min(
            64,
            cmp::max(-64, sin_value1 + sin_value2 + sin_value3 + sin_value4),
        );
        let map = move |n| (i16::from(n) * (sin_value + 64) / 128) as u8;
        [map(val[0]), map(val[1]), map(val[2])]
    })
}

fn wave_modifier_nw(
    num_periods: u16,
    angle_add: u8,
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    iter.enumerate().map(move |(idx, val)| {
        let led_pos = idx as u32;
        let angle = u32::from(angle_add)
            + u32::from(num_periods) * 256 * led_pos
                / (NORTH_LEDS + WEST_LEDS + SOUTH_LEDS + EAST_LEDS) as u32;
        let sin_value = i16::from(sin_approx((angle & 0xff) as u8));
        let map = move |n| (i16::from(n) * (sin_value + 64) / 128) as u8;
        [map(val[0]), map(val[1]), map(val[2])]
    })
}

fn wave_modifier_se(
    num_periods: u16,
    angle_add: u8,
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    iter.enumerate().map(move |(idx, val)| {
        let led_pos =
            ((SOUTH_LEDS + EAST_LEDS) as u32 - idx as u32) + (NORTH_LEDS + WEST_LEDS) as u32;
        let angle = u32::from(angle_add)
            + u32::from(num_periods) * 256 * led_pos
                / (NORTH_LEDS + WEST_LEDS + SOUTH_LEDS + EAST_LEDS) as u32;
        let sin_value = i16::from(sin_approx((angle & 0xff) as u8));
        let map = move |n| (i16::from(n) * (sin_value + 64) / 128) as u8;
        [map(val[0]), map(val[1]), map(val[2])]
    })
}

fn cursor_add_nw(
    clock_value: Duration,
    cursor_color: [u8; 3],
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    let cursor_pos = (((clock_value.as_millis() / 500) as u32)
        % u32::try_from(WEST_LEDS + NORTH_LEDS + EAST_LEDS + SOUTH_LEDS).unwrap())
        as u16;
    iter.enumerate().map(move |(pos, value)| {
        if pos == usize::from(cursor_pos) {
            cursor_color
        } else {
            value
        }
    })
}

fn cursor_add_se(
    clock_value: Duration,
    cursor_color: [u8; 3],
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    let cursor_pos = (((clock_value.as_millis() / 500) as u32)
        % u32::try_from(WEST_LEDS + NORTH_LEDS + EAST_LEDS + SOUTH_LEDS).unwrap())
        as u16;
    let cursor_pos_adj = cursor_pos
        .checked_sub((WEST_LEDS + NORTH_LEDS) as u16)
        .map(|n| (SOUTH_LEDS + EAST_LEDS) as u16 - n)
        .unwrap_or(u16::max_value());
    iter.enumerate().map(move |(pos, value)| {
        if pos == usize::from(cursor_pos_adj) {
            cursor_color
        } else {
            value
        }
    })
}

/// Returns the approximation of `sin(angle)`.
///
/// A value of 256 for the angle represents `2pi`.
///
/// This function returns a value between -64 (represents -1) and 64 (represents 1).
fn sin_approx(angle: u8) -> i8 {
    // Baskara I's approximation.
    let (angle, invert) = if angle < 128 {
        (i32::from(angle), false)
    } else {
        (i32::from(angle) - 128, true)
    };
    let angle_times_pi_minus_angle = angle * (128i32 - angle);
    let nominator = 4 * angle_times_pi_minus_angle;
    let denominator = ((5 * 128 * 128) / 4) - angle_times_pi_minus_angle;
    debug_assert!(nominator <= denominator);
    let result = (64 * nominator / denominator);
    (if invert { -result } else { result }) as i8
}
