#![feature(asm_experimental_arch, asm_const)]
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
        ruduino::delay::delay_us(280);
    }
}

#[inline(always)]
fn send_byte(byte: u8) {
    for n in 0..8 {
        if (byte & (1 << n)) == 1 {
            send1(<<port::B3 as ruduino::Pin>::PORT as ruduino::Register>::ADDRESS, 3);
        } else {
            send0(<<port::B3 as ruduino::Pin>::PORT as ruduino::Register>::ADDRESS, 3);
        }
    }
}

#[inline(always)]
fn send0(addr: *mut u8, pin: u8) {
    unsafe {
        core::arch::asm!(r#"
            sbi 0x05, {mask}
            nop
            nop
            nop
            nop
            cbi 0x05, {mask}
            nop
            nop
            nop
            nop
            nop
            nop
            nop
            nop
            nop
            nop
        "#, mask = const 3);
    }
}

#[inline(always)]
fn send1(addr: *mut u8, pin: u8) {
    unsafe {
        core::arch::asm!(r#"
            sbi 0x05, {mask}
            nop
            nop
            nop
            nop
            nop
            nop
            nop
            nop
            nop
            nop
            cbi 0x05, {mask}
            nop
            nop
            nop
            nop
        "#, mask = const 3);
    }
}
