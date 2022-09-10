#![feature(asm_experimental_arch, asm_const)]
#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn main() {
    // Set port B0 as an output port. (on the Arduino Uno, this is the one marked "8" on DIGITAL side)
    unsafe {
        core::arch::asm!("sbi {addr}, {pin}", addr = const 0x4, pin = const 0);
    }

    // Set port B1 as an output port. (on the Arduino Uno, this is the one marked "9" on DIGITAL side)
    unsafe {
        core::arch::asm!("sbi {addr}, {pin}", addr = const 0x4, pin = const 1);
    }

    let mut data = [0u8; 50 * 3];

    loop {
        data[0] = data[0].wrapping_add(2);
        data[1] = data[1].wrapping_add(1);
        data[2] = data[2].wrapping_add(3);
        data[3] = 255;
        data[4] = 0;
        data[5] = data[5].wrapping_add(1);

        upload_data::<0>(&data);
    }
}

/// Sends the given data to the given PIN of port B.
///
/// This takes around 1125ns per byte.
fn upload_data<const PIN: usize>(input_data: &[u8]) {
    // TODO: don't wait 50µs always
    ruduino::delay::delay_us(280);

    ruduino::interrupt::without_interrupts(|| {
        unsafe {
            // See <http://ww1.microchip.com/downloads/en/devicedoc/atmel-0856-avr-instruction-set-manual.pdf>

            // To write a 1, we set the bit high for 10 cycles (625ns) then low for 4 cycles (250ns).
            // To write a 0, we set the bit high for 4 cycles (250ns) then low for 10 cycles (625ns).
            // Note that these timings don't count the time it takes to actually set or clear the
            // bit (125ns twice).
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
