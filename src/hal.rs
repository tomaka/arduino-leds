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
    // The ASM code below doesn't like it when the length is 0.
    if input_data.is_empty() {
        return;
    }

    debug_assert!(input_data.len() <= 256 * 255);

    unsafe {
        // See <http://ww1.microchip.com/downloads/en/devicedoc/atmel-0856-avr-instruction-set-manual.pdf>
        // and <https://github.com/rust-lang/rust/blob/263d8682d6e01bb02727b15b1c72ffabc0e7396b/compiler/rustc_target/src/asm/avr.rs>
        // For reminder, 1 cycle = 62.5ns

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
                // T= 0, set pin output to 1
                sbi {addr}, {pin}       // 2 cycles
                nop                     // 1 cycle
                nop
                nop

                // T= 5 cycles
                // Set pin output to 0 if bit 7 of `val` isn't set
                sbrs {val}, 7           // 1 cycle if condition is false (no skip), 2 cycle if true and 1-word instruction is skipped
                cbi {addr}, {pin}       // 2 cycles
                // If bit 7 of `val` isn't set, the pin stayed at 1 for 6 cycles (375ns)

                // T= 7 (if bit 7 of `val` was set) or 8 cycles (if bit 7 of `val` was clear)
                dec {nbits}             // 1 cycle
                // Jump to label `1` if `nbits` is now 0
                breq 1f                 // 1 cycle if condition is false, 2 if true

                // T= 9 or 10 (depending on bit 7 of `val`)
                nop
                nop
                nop
                nop
                nop
                nop
                nop
                nop
                nop

                // T= 18 or 19
                // Set pin output to 0 if bit 7 of `val` is set
                // Either this `cbi` or the one above is executed, but never both
                sbrc {val}, 7
                cbi {addr}, {pin}
                // If bit 7 of `val` is set, the pin stayed at 1 for 19 cycles (1187.5ns)

                // T= 21
                rol {val}               // 1 cycle
                nop
                nop
                nop
                nop
                nop
                rjmp 0b                 // 2 cycles
                // We jump back to label `0` at T= 29

            1:
                // We arrive here at T= 10 or 11
                // Reset nbits to 8
                ldi {nbits}, 8          // 1 cycle
                dec {nbytes_low}
                // T= 12 or 13
                breq 2f                 // 1 cycle if condition is false, 2 if true
                ld {tmp}, X+            // 1 cycle, although it's kind of complicated

                // T= 14 or 15
                nop
                nop
                nop
                nop
                sbrc {val}, 7
                cbi {addr}, {pin}
                // If bit 7 of `val` is set, the pin stayed at 1 for 19 cycles (1187.5ns)

                // T= 21
                mov {val}, {tmp}        // 1 cycle
                nop
                nop
                nop
                nop
                nop
                rjmp 0b                 // 2 cycles
                // We jump back to label `0` at T= 29

            2:
                // We arrive here at T= 14 or 15
                dec {nbytes_high}
                // Jump to the end if no more data
                breq 3f
                ldi {nbytes_low}, 0     // 1 cycle
                ld {tmp}, X+            // 1 cycle, although it's kind of complicated

                sbrc {val}, 7           // T= 18 or 19, skip next instruction if bit 7 of val is clear
                cbi {addr}, {pin}       // set pin output to 0
                // If bit 7 of `val` is set, the pin stayed at 1 for 19 cycles (1187.5ns)

                // T= 21
                mov {val}, {tmp}
                nop
                nop
                nop
                nop
                nop
                rjmp 0b
                // We jump back to label `0` at T= 29

            3:
                nop
                nop
                cbi {addr}, {pin}       // set pin output to 0

                // We add some nops just to make sure that the pin remains at 0 long enough,
                // which is important for example if the user calls this function twice in a row
                // with the same port.
                nop
                nop

                // Trailer to restore the SREG value.
                pop {tmp}
                sts 0x5f, {tmp}     // SREG
            "#,
            addr = const 0x5, pin = const PIN,

            nbytes_low = inout(reg_upper) u8::try_from(input_data.len() & 0xff).unwrap() => _,
            nbytes_high = inout(reg_upper) u8::try_from((input_data.len() >> 8) & 0xff).unwrap() + 1 => _,

            // Temporary registers.
            nbits = inout(reg_upper) 8u8 => _,
            tmp = out(reg_upper) _,
            val = out(reg_upper) _,

            inout("X") input_data.as_ptr() => _,

            options(preserves_flags)
        );
    }
}
