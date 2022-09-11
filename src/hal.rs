#[cfg(not(target_arch = "avr"))]
compile_error!("Can only work on AVR");

pub fn enable_bport_out<const PIN: usize>() {
    unsafe {
        core::arch::asm!("sbi {addr}, {pin}", addr = const 0x4, pin = const PIN);
    }
}

/// Sends the given data to the given PIN of port B.
///
/// This takes around 1125ns per byte.
// TODO: document that multiples of 255 are preferred
pub fn upload_bport_data<const PIN: usize>(input_data: &[u8]) {
    for chunk in input_data.chunks(255) {
        upload_bport_data_inner::<PIN>(input_data.chunks(255).next().unwrap());
    }
}

fn upload_bport_data_inner<const PIN: usize>(input_data: &[u8]) {
    // The ASM code below doesn't like it when the length is 0.
    if input_data.is_empty() {
        return;
    }

    debug_assert!(input_data.len() <= 255);

    unsafe {
        // See <http://ww1.microchip.com/downloads/en/devicedoc/atmel-0856-avr-instruction-set-manual.pdf>

        // To write a 1, we set the bit high for 10 cycles (625ns) then low for 4 cycles (250ns).
        // To write a 0, we set the bit high for 4 cycles (250ns) then low for 10 cycles (625ns).
        // Note that these timings don't count the time it takes to actually set or clear the
        // bit (125ns twice).
        core::arch::asm!(r#"
                lds {tmp}, 0x5f  // SREG
                push {tmp}
                cli

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

                pop {tmp}
                sts 0x5f, {tmp}     // SREG
            "#,
            addr = const 0x5, pin = const PIN,

            nbytes = in(reg) u8::try_from(input_data.len()).unwrap(),

            // Temporary registers.
            nbits = inout(reg) 8u8 => _,
            tmp = out(reg) _,
            val = out(reg) _,

            in("X") input_data.as_ptr(),
            lateout("X") _,

            options(preserves_flags)
        );
    }
}
