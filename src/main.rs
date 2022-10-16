#![feature(
    asm_experimental_arch,
    asm_const,
    abi_avr_interrupt,
    default_alloc_error_handler
)]
#![no_std]
#![no_main]

use core::time::Duration;

mod hal;
mod leds;

static mut NUM_TIMER0_OVERFLOWS: u32 = 0;

#[no_mangle]
pub extern "C" fn main() {
    // Enable interrupts.
    // They have to be enabled at some point for things to work, and there's no reason to not do
    // it right at the beginning.
    unsafe {
        core::arch::asm!("sei");
    }

    // Set ports B0 and B1 as output ports.
    // On the Arduino Uno, they are the ones marked "8" (B0) and "9" (B1) on DIGITAL side.
    hal::enable_bport_out::<0>();
    hal::enable_bport_out::<1>();

    // Enable the timer0 with a prescaler of 64.
    // This means that every 64 cycles the clock timer increases by 1. After 16384 cycles
    // (64 * 256), which is 1024µs, the timer overflows and an interrupt is generated. The
    // interrupt handler increases `NUM_TIMER0_OVERFLOWS` by 1.
    unsafe {
        core::arch::asm!(
            r#"
        lds {tmp}, 0x5f  // SREG
        push {tmp}
        cli

        sts 0x44, {tccrOa}
        sts 0x45, {tccr0b}
        sts 0x46, {tcnt0}
        sts 0x6e, {timsk0}

        pop {tmp}
        sts 0x5f, {tmp}     // SREG
    "#,  tccrOa = in(reg) 0u8, tccr0b = in(reg) 0b11u8,
        tcnt0 = in(reg) 0u8, timsk0 = in(reg) 0b1u8,
        tmp = out(reg) _
        );
    }

    // Buffer to collect the LED data in. Must be large enough to fit all the data of all the LED
    // strips at once, otherwise the sending timing will not work.
    let mut data_buffer = [0; leds::TOTAL_LEDS * 3];

    loop {
        let clock_value = unsafe {
            // In order to grab the clock value without running the risk of a race condition, we
            // need to interrupts. For this reason, this is done directly in assembly code.

            let sreg: u8;
            let subtimer: u8;
            let num_timer0_overflows_byte0: u8;
            let num_timer0_overflows_byte1: u8;
            let num_timer0_overflows_byte2: u8;
            let num_timer0_overflows_byte3: u8;

            core::arch::asm!(r#"
                lds {sreg}, 0x5f  // SREG
                cli
                lds {subtimer}, 0x46
                ld {byte0}, X+
                ld {byte1}, X+
                ld {byte2}, X+
                ld {byte3}, X+
                sts 0x5f, {sreg}
                "#,
                sreg = out(reg) _,
                subtimer = out(reg) subtimer,
                byte0 = out(reg) num_timer0_overflows_byte0,
                byte1 = out(reg) num_timer0_overflows_byte1,
                byte2 = out(reg) num_timer0_overflows_byte2,
                byte3 = out(reg) num_timer0_overflows_byte3,
                inout("X") (&NUM_TIMER0_OVERFLOWS) as *const u32 as usize => _,
                options(preserves_flags, readonly, nostack)
            );

            let num_timer0_overflows = u32::from_ne_bytes([
                num_timer0_overflows_byte0,
                num_timer0_overflows_byte1,
                num_timer0_overflows_byte2,
                num_timer0_overflows_byte3,
            ]);

            Duration::from_micros(
                u64::from(num_timer0_overflows) * 1024
                    + u64::from(subtimer) * 64 * 6250 / 100 / 1000,
            )
        };

        let mut data_size = 0usize;
        let mut northwest_data_end = 0;

        for strip in [leds::Strip::NorthWest, leds::Strip::SouthEast] {
            northwest_data_end = data_size;

            let mut iter = leds::led_colors_lerp(
                leds::Mode::Off,
                leds::Mode::Test,
                Duration::from_secs(50), // TODO:
                clock_value,
                strip,
            )
            .flat_map(|c| {
                // For some reason, the LED strip shows green as blue and vice versa, so we swap bytes.
                [c[0], c[2], c[1]].into_iter()
            })
            .fuse();

            let mut data_buffer_iter = data_buffer[data_size..].iter_mut();
            loop {
                match (data_buffer_iter.next(), iter.next()) {
                    (Some(o), Some(i)) => {
                        *o = i;
                        data_size += 1;
                    }
                    (None, Some(_)) => panic!(), // Note: the buffer must be large enough to hold all the data.
                    _ => break,
                }
            }
        }

        hal::upload_bport_data::<1>(&data_buffer[..northwest_data_end]);
        hal::upload_bport_data::<0>(&data_buffer[northwest_data_end..data_size]);

        // TODO: don't wait the full duration
        ruduino::delay::delay_us(300);
    }
}

#[no_mangle]
pub unsafe extern "avr-interrupt" fn __vector_16() {
    NUM_TIMER0_OVERFLOWS += 1;
}

#[no_mangle]
pub unsafe extern "C" fn abort() {
    loop {}
}
