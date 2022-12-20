#[cfg(not(target_arch = "avr"))]
compile_error!("Can only work on AVR");

pub fn enable_bport_out<const PIN: usize>() {
    unsafe {
        core::arch::asm!(
            "sbi {addr}, {pin}",
            addr = const 0x4, pin = const PIN,
            options(preserves_flags, nostack)
        );
    }
}

/// Sends the given data to the given PIN of port B.
pub fn upload_bport_data<const PIN: usize>(input_data: &[u8]) {
    unsafe {
        // See <http://ww1.microchip.com/downloads/en/devicedoc/atmel-0856-avr-instruction-set-manual.pdf>
        // and <https://github.com/rust-lang/rust/blob/263d8682d6e01bb02727b15b1c72ffabc0e7396b/compiler/rustc_target/src/asm/avr.rs>
        // and <https://wp.josh.com/2014/05/13/ws2812-neopixels-are-not-so-finicky-once-you-get-to-know-them/>
        // For reminder, 1 cycle = 62.5ns
        core::arch::asm!(r#"
                lds {sreg}, 0x5f  // SREG
                cli

                rjmp 1f


            0:
                // T= 0 cycles, set pin output to 1
                sbi {addr}, {pin}       // 2 cycles
                nop                     // 1 cycle
                nop

                // T= 4
                // Set pin output to 0 if bit 7 of `val` is clear
                sbrs {val}, 7           // 1 cycle if condition is false (no skip), 2 cycle if true and 1-word instruction is skipped
                cbi {addr}, {pin}       // 2 cycles
                // If bit 7 of `val` is clear, the pin stayed at 1 for 5 cycles (312.5ns)

                // T= 6 (if bit 7 of `val` is set) or 7 (if bit 7 of `val` is clear)
                nop
                nop

                // T= 8 or 9
                // Set pin output to 0 if bit 7 of `val` is set
                // Either this `cbi` is executed or the one above, but never both
                sbrc {val}, 7           // 1 cycle if condition is false (no skip), 2 cycle if true and 1-word instruction is skipped
                cbi {addr}, {pin}       // 2 cycles
                // If bit 7 of `val` is set, the pin stayed at 1 for 11 cycles (687.5ns)

                // T= 11
                dec {nbits}             // 1 cycle
                breq 1f                 // 1 cycle if condition is false, 2 cycle if condition is true

                // T= 13
                rol {val}               // 1 cycle
                nop
                nop
                nop
                nop
                nop
                nop
                nop
                rjmp 0b                 // 2 cycles
                // We jump back to 0 at T= 23 (1437.5ns)

            1:
                // T= 14
                subi {nbytes_low}, 1    // 1 cycle
                brcs 2f

                // T= 16
                nop
                rjmp 3f

            2:
                // T= 17
                subi {nbytes_high}, 1
                brcs 4f

            3:
                // T= 19
                ldi {nbits}, 8          // 1 cycle
                ld {val}, X+            // 1 cycle

                rjmp 0b
                // We jump back to 0 at T= 23 (1437.5ns)


            4:
                // We add some nops just to make sure that the pin remains at 0 long enough,
                // which is important for example if the user calls this function twice in a row
                // with the same port.
                nop
                nop
                nop
                nop
                nop
                nop
                nop

                // Trailer to restore the SREG value.
                sts 0x5f, {sreg}     // SREG

            "#,
            addr = const 0x5, pin = const PIN,

            nbytes_low = inout(reg_upper) u8::try_from(input_data.len() & 0xff).unwrap() => _,
            nbytes_high = inout(reg_upper) u8::try_from((input_data.len() >> 8) & 0xff).unwrap() => _,

            // Temporary registers.
            nbits = out(reg_upper) _,
            sreg = out(reg_upper) _,
            val = out(reg_upper) _,

            inout("X") input_data.as_ptr() => _,

            options(preserves_flags, nostack)
        );
    }
}
