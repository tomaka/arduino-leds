#![feature(asm_experimental_arch, asm_const)]
#![no_std]
#![no_main]

use ruduino::cores::atmega328p::port;
use ruduino::Pin as _;

#[no_mangle]
pub extern "C" fn main() {
    port::B3::set_output();

    loop {
        ruduino::interrupt::without_interrupts(|| {
            upload_data(&[255, 0, 0, 0, 255, 0, 0, 0]);

            /*for _ in 0..50 {
                // Num LEDs
                send_byte(255);
                send_byte(0);
                send_byte(0);
            }*/
        });

        ruduino::delay::delay_us(50);
    }
}

fn upload_data(input_data: &[u8]) {
    ruduino::interrupt::without_interrupts(|| {
        unsafe {
            // See <http://ww1.microchip.com/downloads/en/devicedoc/atmel-0856-avr-instruction-set-manual.pdf>
            core::arch::asm!(r#"
                ldi {nbits}, 8
                cbi {addr}, {mask}      // T=14, set pin output to 0
                ld {low}, {port}
                mov {high}, {low}
                ori {high}, {mask}
                ld {val}, {input_data}+
                mov {tmp}, {low}

            0:
                sbi {addr}, {mask}      // T= 0, set pin output to 1
                sbrc {val}, 7           // T= 2, skip next instruction if bit 7 of val is clear
                mov {tmp}, {high}       // T= ?, 
                dec {nbits}             // T= 4, 
                nop
                st {port}, {tmp}        // T= 6, set pin output to tmp, so either "high" or "low" depending on bit 7 of "val"
                mov {tmp}, {low}        // T= 8, reset tmp
                breq 1f                 // T= 9, jump if nbits == 0
                rol {val}               // T=10, rotate the value to write so that bit 7 becomes bit 6
                rjmp .+0                // T=11, nop 2 cycles
                cbi {addr}, {mask}      // T=13, set pin output to 0
                rjmp .+0                // T=15, nop
                nop
                rjmp 0b                 // T=18, taking 2 cycles

            1:
                ldi {nbits}, 8          // T=11, reset nbits to 8
                ld {val}, {input_data}+ // T=12, load the next byte to write
                cbi {addr}, {mask}      // T=14, set pin output to 0
                rjmp .+0                // T=16, nop
                nop
                dec {nbytes}            // T=19, if nbytes is 0 then the byte we just read is out of bounds
                brne 0b                 // T=20, taking 2 cycles
            "#,
                addr = const 0x25, mask = const 3,
                port = in(reg_ptr) 0x25 as *mut u8,

                input_data = in(reg_ptr) input_data.as_ptr(),
                nbytes = in(reg) u8::try_from(input_data.len()).unwrap(),

                high = out(reg) _,
                low = out(reg) _,

                // Temporary registers.
                nbits = out(reg) _,
                tmp = out(reg) _,
                val = out(reg) _,

                options(nostack)
            );
        }
    })
}
