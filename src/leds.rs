use core::{cmp, iter, time::Duration};

pub const TOTAL_LEDS: usize = WEST_LEDS + NORTH_LEDS + SOUTH_LEDS + EAST_LEDS;
const WEST_LEDS: usize = 22;
const NORTH_LEDS: usize = 62;
const SOUTH_LEDS: usize = 64; // Note: it's actually 64.5, as the corner cuts it in half, a bit annoying
const EAST_LEDS: usize = 25;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Off,
    Neutral,
    Fireplace,
    SegmentLights,
    WholeStripAlternatingColor,
    PartyCycle,
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
    updates_wrapping_counter: u8,
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

    let colors1 = led_colors(mode1, clock_value, updates_wrapping_counter, strip);
    let colors2 = led_colors(mode2, clock_value, updates_wrapping_counter, strip);

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
    mut mode: Mode,
    clock_value: Duration,
    updates_wrapping_counter: u8,
    strip: Strip,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    macro_rules! gen {
        ($($n:ident),*) => {
            #[derive(Clone)]
            enum ModeIter<$($n),*> {
                $($n($n),)*
            }

            impl<
                    $($n: Iterator<Item = [u8; 3]>,)*
                > Iterator for ModeIter<$($n),*>
            {
                type Item = [u8; 3];

                fn next(&mut self) -> Option<Self::Item> {
                    match self {
                        $(ModeIter::$n(i) => i.next(),)*
                    }
                }

                fn size_hint(&self) -> (usize, Option<usize>) {
                    match self {
                        $(ModeIter::$n(i) => i.size_hint(),)*
                    }
                }
            }
        };
    }

    gen!(
        OffNw,
        OffSe,
        Fireplace,
        Neutral,
        SegmentLights,
        WholeStripAlternatingColor
    );

    if matches!(mode, Mode::PartyCycle) {
        mode = match ((clock_value.as_millis() as u16) / 4780) % 2 {
            0 => Mode::WholeStripAlternatingColor,
            1 => Mode::SegmentLights,
            _ => unreachable!(),
        }
    }

    match (mode, strip) {
        (Mode::Off, Strip::NorthWest) => {
            ModeIter::OffNw(iter::repeat([0, 0, 0]).take(NORTH_LEDS + WEST_LEDS))
        }
        (Mode::Off, Strip::SouthEast) => {
            ModeIter::OffSe(iter::repeat([0, 0, 0]).take(SOUTH_LEDS + EAST_LEDS))
        }
        (Mode::Neutral, _) => ModeIter::Neutral(
            iter::repeat({
                // Roughly white.
                [140, 50, 20]
            })
            .take(match strip {
                Strip::NorthWest => NORTH_LEDS + WEST_LEDS,
                Strip::SouthEast => SOUTH_LEDS + EAST_LEDS,
            }),
        ),
        (Mode::PartyCycle, _) => unreachable!(), // Handled above.
        (Mode::Fireplace, strip) => ModeIter::Fireplace(seemingly_random_vibration(
            clock_value,
            strip,
            0,
            255,
            iter::repeat([0, 0, 0]).take(match strip {
                Strip::NorthWest => NORTH_LEDS + WEST_LEDS,
                Strip::SouthEast => SOUTH_LEDS + EAST_LEDS,
            }),
            |_, intensity| {
                let color1 = [9, 3, 0];
                let color2 = [40, 5, 0];
                let intensity = 255 - ONE_MINUS_EXP_MINUS_X_TABLE[(255 - intensity) as usize];
                [
                    ((color1[0] as u16 * (255 - intensity) as u16
                        + color2[0] as u16 * intensity as u16)
                        / 255) as u8,
                    ((color1[1] as u16 * (255 - intensity) as u16
                        + color2[1] as u16 * intensity as u16)
                        / 255) as u8,
                    ((color1[2] as u16 * (255 - intensity) as u16
                        + color2[2] as u16 * intensity as u16)
                        / 255) as u8,
                ]
            },
        )),
        (Mode::WholeStripAlternatingColor, strip) => {
            let color = |v| -> [u8; 3] {
                match v % 6 {
                    0 => [128, 0, 0],
                    1 => [80, 0, 32],
                    2 => [0, 0, 128],
                    3 => [20, 128, 50],
                    4 => [0, 128, 0],
                    5 => [64, 64, 0],
                    _ => unreachable!(),
                }
            };

            let color1 = color(clock_value.as_secs() as u8 / 2);
            let color2 = color((clock_value.as_secs() as u8 / 2).wrapping_add(1));
            let lerp = ((clock_value.as_millis() as u32 * 256) / 2000) as u8;

            // TODO: dry
            fn avg(a: u8, b: u8, b_weight: u8) -> u8 {
                let res =
                    u16::from(a) * u16::from(255 - b_weight) + u16::from(b) * u16::from(b_weight);
                u8::try_from(res / 255).unwrap()
            }

            let final_color = [
                avg(color1[0], color2[0], lerp),
                avg(color1[1], color2[1], lerp),
                avg(color1[2], color2[2], lerp),
            ];

            ModeIter::WholeStripAlternatingColor(iter::repeat(final_color).take(match strip {
                Strip::NorthWest => NORTH_LEDS + WEST_LEDS,
                Strip::SouthEast => SOUTH_LEDS + EAST_LEDS,
            }))
        }
        (Mode::SegmentLights, strip) => {
            let segment_offset = {
                let base = (clock_value.as_millis() as u32) / 35;
                let base = base % 256;
                if base > 128 {
                    256 - base
                } else {
                    base
                }
            };

            let iter = (0u32..).map(move |idx| {
                let led_pos = if matches!(strip, Strip::NorthWest) {
                    idx
                } else {
                    ((SOUTH_LEDS + EAST_LEDS) as u32 - idx) + (NORTH_LEDS + WEST_LEDS) as u32
                };

                let segment_num = (led_pos + segment_offset) / 6;
                match segment_num % 7 {
                    0 => [128, 0, 0],
                    1 => [100, 50, 50],
                    2 => [0, 64, 64],
                    3 => [64, 64, 0],
                    4 => [0, 0, 128],
                    5 => [0, 128, 0],
                    6 => [64, 0, 64],
                    _ => unreachable!(),
                }
            });

            ModeIter::SegmentLights(iter.take(match strip {
                Strip::NorthWest => NORTH_LEDS + WEST_LEDS,
                Strip::SouthEast => SOUTH_LEDS + EAST_LEDS,
            }))
        }
    }
}

fn west_to_east_gradiant_modifier(
    strip: Strip,
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    let mut n = 0;
    iter.map(move |item| {
        const MIN_INTENSITY: u16 = 4;

        let intensity = if matches!(strip, Strip::NorthWest) {
            if n < WEST_LEDS {
                MIN_INTENSITY
            } else {
                MIN_INTENSITY
                    + (256 - MIN_INTENSITY) * u16::try_from(n - WEST_LEDS).unwrap()
                        / u16::try_from(NORTH_LEDS).unwrap()
            }
        } else {
            if n < SOUTH_LEDS {
                MIN_INTENSITY
                    + (256 - MIN_INTENSITY) * u16::try_from(n).unwrap()
                        / u16::try_from(SOUTH_LEDS).unwrap()
            } else {
                256
            }
        };

        n += 1;

        [
            (intensity * u16::from(item[0]) / 256) as u8,
            (intensity * u16::from(item[1]) / 256) as u8,
            (intensity * u16::from(item[2]) / 256) as u8,
        ]
    })
}

fn flashing(
    updates_wrapping_counter: u8,
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    let flash = updates_wrapping_counter % 2 == 0;
    iter.map(move |color| {
        if flash {
            [
                cmp::max(cmp::min(u16::from(color[0]) * 9 / 4, 255) as u8, 1),
                cmp::max(cmp::min(u16::from(color[1]) * 9 / 4, 255) as u8, 1),
                cmp::max(cmp::min(u16::from(color[2]) * 9 / 4, 255) as u8, 1),
            ]
        } else {
            color
        }
    })
}

// TODO: has a weird API now
fn seemingly_random_vibration(
    clock_value: Duration,
    strip: Strip,
    wave_min_intensity: u8,
    wave_max_intensity: u8,
    iter: impl Iterator<Item = [u8; 3]> + Clone,
    mut map: impl FnMut([u8; 3], u8) -> [u8; 3] + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    let wave1_add = ((clock_value.as_millis() as u32) / 6) as u32;
    let wave2_add = ((clock_value.as_millis() as u32) / 30) as u32;
    let wave3_add = ((clock_value.as_millis() as u32) / 21) as u32;
    let wave4_add = ((clock_value.as_millis() as u32) / 22) as u32;

    iter.enumerate().map(move |(idx, val)| {
        let led_pos = if matches!(strip, Strip::NorthWest) {
            idx as u32
        } else {
            ((SOUTH_LEDS + EAST_LEDS) as u32 - idx as u32) + (NORTH_LEDS + WEST_LEDS) as u32
        };

        let angle1 = u32::from(wave1_add) + 5 * 256 * led_pos / TOTAL_LEDS as u32;
        let sin_value1 = i16::from(SIN_TABLE[(angle1 & 0xff) as usize]);
        let angle2 = 3 * 256 * led_pos / TOTAL_LEDS as u32 - u32::from(wave2_add);
        let sin_value2 = i16::from(SIN_TABLE[(angle2 & 0xff) as usize]);
        let angle3 = u32::from(wave3_add) + 7 * 256 * led_pos / TOTAL_LEDS as u32;
        let sin_value3 = i16::from(SIN_TABLE[(angle3 & 0xff) as usize]);
        let angle4 = 11 * 256 * led_pos / TOTAL_LEDS as u32 - u32::from(wave4_add);
        let sin_value4 = i16::from(SIN_TABLE[(angle4 & 0xff) as usize]);

        let sin_value = cmp::min(
            127,
            cmp::max(
                -128,
                (sin_value1 + sin_value2 + sin_value3 + sin_value4) / 2,
            ),
        );

        let intensity = ((sin_value + 128) as u16
            * (wave_max_intensity as u16 - wave_min_intensity as u16)
            / 255
            + wave_min_intensity as u16) as u8;

        map(val, intensity)
    })
}

fn slowly_changing_color(clock_value: Duration) -> [u8; 3] {
    const COLOR_DURATION: u32 = 60000;

    let colors = [
        [255, 0, 0],
        [0, 128, 128],
        [0, 0, 255],
        [128, 128, 0],
        [0, 255, 0],
        [128, 0, 128],
    ];

    let step = (clock_value.as_millis() as u32) % (COLOR_DURATION * colors.len() as u32);

    let color_from_idx = (step / COLOR_DURATION) as usize;
    let color_from = colors[color_from_idx];
    let color_to_idx = (color_from_idx + 1) % colors.len();
    let color_to = colors[color_to_idx];

    assert!(COLOR_DURATION < u16::max_value() as u32); // Overflow check.
    let progress = (((step % COLOR_DURATION) << 16) / COLOR_DURATION) as u32;
    let one_minus_progress = u16::max_value() as u32 - progress;

    [
        ((color_from[0] as u32 * one_minus_progress + color_to[0] as u32 * progress) >> 16) as u8,
        ((color_from[1] as u32 * one_minus_progress + color_to[1] as u32 * progress) >> 16) as u8,
        ((color_from[2] as u32 * one_minus_progress + color_to[2] as u32 * progress) >> 16) as u8,
    ]
}

fn colors_rotation_by_side(
    clock_value: Duration,
    strip: Strip,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    let side_add = (((clock_value.as_secs() as u32) / 2) & 0xff) as u16;

    (0..).map_while(move |counter| {
        let side_num = match (counter, strip) {
            (n, Strip::NorthWest) if n >= NORTH_LEDS + WEST_LEDS => return None,
            (n, Strip::NorthWest) if n >= WEST_LEDS => 1,
            (_, Strip::NorthWest) => 0,
            (n, Strip::SouthEast) if n >= SOUTH_LEDS + EAST_LEDS => return None,
            (n, Strip::SouthEast) if n >= SOUTH_LEDS => 2,
            (_, Strip::SouthEast) => 3,
        };

        let side_num_adjusted = (side_num + side_add) % 4;

        Some(match side_num_adjusted {
            0 => [255, 0, 0],
            1 => [128, 0, 128],
            2 => [0, 0, 255],
            3 => [0, 255, 0],
            _ => unreachable!(),
        })
    })
}

fn wave_modifier_nw(
    num_periods: u16,
    angle_add: u8,
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    iter.enumerate().map(move |(idx, val)| {
        let led_pos = idx as u32;
        let angle =
            u32::from(angle_add) + u32::from(num_periods) * 256 * led_pos / TOTAL_LEDS as u32;
        let sin_value = i16::from(SIN_TABLE[(angle & 0xff) as usize]);
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
        let angle =
            u32::from(angle_add) + u32::from(num_periods) * 256 * led_pos / TOTAL_LEDS as u32;
        let sin_value = i16::from(SIN_TABLE[(angle & 0xff) as usize]);
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

fn onoff_periodic(
    clock_value: Duration,
    iter: impl Iterator<Item = [u8; 3]> + Clone,
) -> impl Iterator<Item = [u8; 3]> + Clone {
    let is_on = (clock_value.as_secs() % 6) < 3;
    iter.map(move |c| if is_on { c } else { [0, 0, 0] })
}

include!(concat!(env!("OUT_DIR"), "/exp_table.rs"));

/// Returns the approximation of `sin(angle)`.
///
/// A value of 256 for the angle represents `2pi`.
///
/// This function returns a value between -64 (represents -1) and 64 (represents 1).
const fn sin_approx(angle: u8) -> i8 {
    // Baskara I's approximation.
    let (angle, invert) = if angle < 128 {
        (angle as i32, false)
    } else {
        (angle as i32 - 128, true)
    };
    let angle_times_pi_minus_angle = angle * (128i32 - angle);
    let nominator = 4 * angle_times_pi_minus_angle;
    let denominator = ((5 * 128 * 128) / 4) - angle_times_pi_minus_angle;
    debug_assert!(nominator <= denominator);
    let result = 64 * nominator / denominator;
    (if invert { -result } else { result }) as i8
}

macro_rules! gen_sin_table {
    ($($n:expr),*) => {
        const SIN_TABLE: [i8; 256] = [
            $(sin_approx($n)),*
        ];
    };
}

// TODO: I'm sure there's a better way than to enumerate the numbers?
gen_sin_table!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135,
    136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154,
    155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173,
    174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192,
    193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211,
    212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230,
    231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249,
    250, 251, 252, 253, 254, 255
);
