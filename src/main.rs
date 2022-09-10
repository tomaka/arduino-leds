#![feature(asm_experimental_arch, asm_const, maybe_uninit_uninit_array, maybe_uninit_array_assume_init)]
#![no_std]
#![no_main]

use ruduino::cores::atmega328p::port;
use ruduino::Pin as _;

#[no_mangle]
pub extern "C" fn main() {
    port::B0::set_output();

    // Note: we do shenanigans with MaybeUninit in order to avoid a linking error with memset
    let data: [core::mem::MaybeUninit<u8>; 5 * 3] = core::mem::MaybeUninit::uninit_array();
    let mut data = unsafe { core::mem::MaybeUninit::array_assume_init(data) };

    data[0] = 255;
    data[1] = 0;
    data[2] = 0;
    data[3] = 0;
    data[4] = 0;
    data[5] = 255;

    loop {
        upload_data::<0>(&data);
    }
}

fn upload_data<const PIN: usize>(input_data: &[u8]) {
    // TODO: don't wait 50µs always
    ruduino::delay::delay_us(280);

    ruduino::interrupt::without_interrupts(|| {
        unsafe {
            // See <http://ww1.microchip.com/downloads/en/devicedoc/atmel-0856-avr-instruction-set-manual.pdf>

            // To write a 1, we set the bit high for 10 cycles (625ns) then low for 4 cycles (250ns).
            // To write a 0, we set the bit high for 4 cycles (250ns) then low for 10 cycles (625ns).
            // Note that these timings don't count the time it takes to actually set or clear the
            // bit (125ns).
            core::arch::asm!(r#"
                ld {val}, X+

            0:
                sbi {addr}, {pin}       // T= 0, set pin output to 1
                nop
                nop
                nop

                sbrs {val}, 7           // T= 5, skip next instruction if bit 7 of val is set
                cbi {addr}, {pin}       // set pin output to 0

                dec {nbits}             // T= 7 or 8 (depending on whether bit 7 of val was set)
                breq 1f                 // T= 8 or 9

                nop
                nop
                nop
                sbrc {val}, 7           // T= 12 or 13, skip next instruction if bit 7 of val is clear
                cbi {addr}, {pin}       // set pin output to 0

                rol {val}               // T= 15, rotate the value to write so that bit 7 becomes bit 6
                nop
                rjmp 0b                 // T= 17

            1:
                ldi {nbits}, 8          // T= 10 or 11, reset nbits to 8
                ld {tmp}, X+ // T= 11 or 12, load the next byte to write

                sbrc {val}, 7           // T= 12 or 13, skip next instruction if bit 7 of val is clear
                cbi {addr}, {pin}       // set pin output to 0

                mov {val}, {tmp}        // T= 15
                dec {nbytes}            // T= 16, if nbytes is 0 then the byte we just read is out of bounds
                brne 0b                 // T= 17
            "#,
                addr = const 0x5, pin = const PIN,

                nbytes = in(reg) u8::try_from(input_data.len()).unwrap(),

                // Temporary registers.
                nbits = inout(reg) 8u8 => _,
                tmp = out(reg) _,
                val = out(reg) _,

                in("X") input_data.as_ptr(),
                lateout("X") _,

                options(nostack)
            );
        }
    })
}
