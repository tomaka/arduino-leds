#[cfg(not(target_arch = "avr"))]
compile_error!("Can only work on AVR");

pub fn enable_bport_out<const PIN: usize>() {
    unsafe {
        core::arch::asm!("sbi {addr}, {pin}", addr = const 0x4, pin = const PIN);
    }
}

/// Sends the given data to the given PIN of port B.
pub fn upload_bport_data<const PIN: usize>(input_data: &[u8]) {
    // The ASM code below doesn't like it when the length is 0.
    if input_data.is_empty() {
        return;
    }

    debug_assert!(input_data.len() <= 256 * 255);

    // TODO: unclear why, but we add +1 to the number of bytes to write to fix a bug
    let num_bytes = input_data.len() + 1;

    unsafe {
        // See <http://ww1.microchip.com/downloads/en/devicedoc/atmel-0856-avr-instruction-set-manual.pdf>

        // To write a 1, we set the bit high for 10 cycles (1125ns) then low for 4 cycles (250ns).
        // To write a 0, we set the bit high for 4 cycles (250ns) then low for 10 cycles (1125ns).
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
                nop
                nop
                nop
                nop
                nop
                nop
                sbrc {val}, 7           // T= 18 or 19, skip next instruction if bit 7 of val is clear
                cbi {addr}, {pin}       // set pin output to 0

                rol {val}               // T= 21, rotate the value to write so that bit 7 becomes bit 6
                nop
                rjmp 0b                 // T= 23

            1:
                ldi {nbits}, 8          // T= 10 or 11, reset nbits to 8
                dec {nbytes_low}        // T= 11 or 12, we update nbytes_low for the byte we've just sent
                breq 2f                 // T= 12 or 13
                ld {tmp}, X+            // T= 13 or 14, load the next byte to write

                nop
                nop
                nop
                nop
                sbrc {val}, 7           // T= 18 or 19, skip next instruction if bit 7 of val is clear
                cbi {addr}, {pin}       // set pin output to 0

                mov {val}, {tmp}        // T= 21
                nop
                brne 0b                 // T= 23

            2:
                dec {nbytes_high}       // T= 14 or 15
                breq 3f                 // T= 15 or 16, jump to the end of no more data
                ldi {nbytes_low}, 255   // T= 16 or 17, reset nbytes_low
                ld {tmp}, X+            // T= 17 or 18, load the next byte to write

                sbrc {val}, 7           // T= 18 or 19, skip next instruction if bit 7 of val is clear
                cbi {addr}, {pin}       // set pin output to 0

                mov {val}, {tmp}        // T= 21
                nop
                rjmp 0b                 // T= 23

            3:
                nop
                nop
                cbi {addr}, {pin}       // set pin output to 0
                nop
                nop
                nop

                pop {tmp}
                sts 0x5f, {tmp}     // SREG
            "#,
            addr = const 0x5, pin = const PIN,

            nbytes_low = in(reg) u8::try_from(num_bytes & 0xff).unwrap(),
            nbytes_high = in(reg) u8::try_from((num_bytes >> 8) & 0xff).unwrap() + 1,

            // Temporary registers.
            nbits = inout(reg) 8u8 => _,
            tmp = out(reg) _,
            val = out(reg) _,

            inout("X") input_data.as_ptr() => _,

            // TODO: restore options(preserves_flags)
        );
    }
}
