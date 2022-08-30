#![no_std]
#![no_main]

use ruduino::Pin as _;
use ruduino::cores::atmega328p::port;

#[no_mangle]
pub extern "C" fn main() {
    port::B3::set_output();

    loop {
        ruduino::interrupt::without_interrupts(|| {
            for _ in 0..50 {  // Num LEDs
                send_byte(255);
                send_byte(0);
                send_byte(0);
            }
        });

        port::B3::set_low();
        ruduino::delay::delay_us(50);
    }
}

#[inline(always)]
fn send_byte(byte: u8) {
    for n in 0..8 {
        if (byte & (1 << n)) == 1 {
            send1();
        } else {
            send0();
        }
    }
}

#[inline(always)]
fn send0() {
    port::B3::set_high();
    delay_ns(500);
    port::B3::set_low();
    delay_ns(2000);
}

#[inline(always)]
fn send1() {
    port::B3::set_high();
    delay_ns(1200);
    port::B3::set_low();
    delay_ns(1300);
}

#[inline(always)]
fn delay_ns(ns: u32) {
    // Note: the division by 4 is done because each loop passed to `delay` is 4 cycles.
    // See also <https://docs.rs/avr_delay/0.3.1/src/avr_delay/lib.rs.html#27-47>
    let ns_lp = 1000000000 / (ruduino::config::CPU_FREQUENCY_HZ / 4);
    let loops = (ns / ns_lp) as u32;
    ruduino::delay::delay(u64::from(loops));
}
