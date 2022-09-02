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
            upload_data(&[255, 0, 0, 255, 0, 0]);

            /*for _ in 0..50 {
                // Num LEDs
                send_byte(255);
                send_byte(0);
                send_byte(0);
            }*/
        });

        port::B3::set_low();
        ruduino::delay::delay_us(280);
    }
}

#[inline(always)]
fn send_byte(byte: u8) {
    for n in 0..8 {
        if (byte & (1 << n)) == 1 {
            send1(
                <<port::B3 as ruduino::Pin>::PORT as ruduino::Register>::ADDRESS,
                3,
            );
        } else {
            send0(
                <<port::B3 as ruduino::Pin>::PORT as ruduino::Register>::ADDRESS,
                3,
            );
        }
    }
}

fn upload_data(input_data: &[u8]) {
    ruduino::interrupt::without_interrupts(|| {
        unsafe {
            // See <http://ww1.microchip.com/downloads/en/devicedoc/atmel-0856-avr-instruction-set-manual.pdf>
            core::arch::asm!(r#"
        0:
            ldi {nbits}, 8
            ld {val}, {input_data}+
            cbi {addr}, {mask}
            rjmp .+0
            nop
            dec {nbytes}
            breq 2f

        1:
            sbi {addr}, {mask}
            sbrc {val}, 7
            mov {tmp}, {high}
            dec {nbits}
            nop
            st {port}, {tmp}
            mov {tmp}, {low}
            breq 0b
            rol {val}
            rjmp .+0
            cbi {addr}, {mask}
            rjmp .+0
            nop
            rjmp 1b
        
        2:
            nop
    "#,
        addr = const 0x25, mask = const 3,
        port = in(reg_ptr) 0x25 as *mut u8,

        input_data = in(reg_ptr) input_data.as_ptr(),
        nbytes = in(reg) u8::try_from(input_data.len()).unwrap(),

        high = in(reg) 3u8,
        low = in(reg) (!3) as u8,

        // Temporary registers.
        nbits = out(reg) _,
        tmp = out(reg) _,
        val = out(reg) _,

        options(nostack));
            /*
             //  Instruction        Clock   Description                           Phase
             "nextbit:\n\t"              // -    label                                     (T =  0)
              "sbi  %0, %1\n\t"     // 2    signal HIGH                         (T =  2)
              "sbrc %4, 7\n\t"       // 1-2  if MSB set                           (T =  ?)
               "mov  %6, %3\n\t"  // 0-1   tmp'll set signal high          (T =  4)
              "dec  %5\n\t"           // 1    decrease bitcount                (T =  5)
              "nop\n\t"                  // 1    nop (idle 1 clock cycle)        (T =  6)
              "st   %a2, %6\n\t"    // 2    set PORT to tmp                 (T =  8)
              "mov  %6, %7\n\t"   // 1    reset tmp to low (default)     (T =  9)
              "breq nextbyte\n\t"  // 1-2  if bitcount ==0 -> nextbyte  (T =  ?)
              "rol  %4\n\t"             // 1    shift MSB leftwards              (T = 11)
              "rjmp .+0\n\t"           // 2    nop nop                                (T = 13)
              "cbi   %0, %1\n\t"    // 2    signal LOW                          (T = 15)
              "rjmp .+0\n\t"           // 2    nop nop                                (T = 17)
              "nop\n\t"                  // 1    nop                                        (T = 18)
              "rjmp nextbit\n\t"     // 2    bitcount !=0 -> nextbit           (T = 20)
             "nextbyte:\n\t"          // -    label                                       -
              "ldi  %5, 8\n\t"         // 1    reset bitcount                       (T = 11)
              "ld   %4, %a8+\n\t" // 2    val = *p++                             (T = 13)
              "cbi   %0, %1\n\t"   // 2    signal LOW                           (T = 15)
              "rjmp .+0\n\t"          // 2    nop nop                                 (T = 17)
              "nop\n\t"                 // 1    nop                                        (T = 18)
              "dec %9\n\t"           // 1    decrease bytecount             (T = 19)
              "brne nextbit\n\t"    // 2    if bytecount !=0 -> nextbit     (T = 20)
              ::
              "I" (PORT_PIN),           // %1
              "e" (&PORT),              // %a2
              "r" (high),               // %3
              "r" (val),                // %4
              "r" (nbits),              // %5
              "r" (tmp),                // %6
              "r" (low),                // %7
              "e" (p),                  // %a8
              "w" (nbytes)              // %9
            );)*/
        }
    })
}

#[inline(always)]
fn send0(addr: *mut u8, pin: u8) {
    unsafe {
        core::arch::asm!(r#"
            sbi 0x25, {mask}
            nop
            nop
            nop
            nop
            cbi 0x25, {mask}
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
        "#, mask = const 3, options(nostack, preserves_flags));
    }
}

#[inline(always)]
fn send1(addr: *mut u8, pin: u8) {
    unsafe {
        core::arch::asm!(r#"
            sbi 0x25, {mask}
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
            cbi 0x25, {mask}
            nop
            nop
            nop
            nop
        "#, mask = const 3, options(nostack, preserves_flags));
    }
}
