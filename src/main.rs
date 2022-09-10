#![feature(asm_experimental_arch, asm_const)]
#![no_std]
#![no_main]

mod hal;

#[no_mangle]
pub extern "C" fn main() {
    // Set ports B0 and B1 as output ports.
    // On the Arduino Uno, they are the ones marked "8" (B0) and "9" (B1) on DIGITAL side.
    hal::enable_bport_out::<0>();
    hal::enable_bport_out::<1>();

    let mut data = [0u8; 50 * 3];

    loop {
        hal::upload_bport_data::<0>(&data);
    }
}
