#![feature(asm_experimental_arch, asm_const, abi_avr_interrupt)]
#![no_std]
#![no_main]

mod hal;

static mut NUM_TIMER0_OVERFLOWS: u8 = 0;

#[no_mangle]
pub extern "C" fn main() {
    // Set ports B0 and B1 as output ports.
    // On the Arduino Uno, they are the ones marked "8" (B0) and "9" (B1) on DIGITAL side.
    hal::enable_bport_out::<0>();
    hal::enable_bport_out::<1>();

    // Enable the timer0 with a prescaler of 64.
    unsafe {
        core::arch::asm!(
            r#"
        lds {tmp}, 0x5f  // SREG
        push {tmp}
        cli

        sts 0x44, {tccrOa}
        sts 0x45, {tccr0b}
        sts 0x46, {tcnt0}
        sts 0x6e, {timsk0}

        pop {tmp}
        sts 0x5f, {tmp}     // SREG
    "#,  tccrOa = in(reg) 0u8, tccr0b = in(reg) 0b11u8,
        tcnt0 = in(reg) 0u8, timsk0 = in(reg) 0b1u8,
        tmp = out(reg) _
        );
    }

    unsafe {
        core::arch::asm!("sei");
    }

    let mut data = [0u8; 50 * 3];

    data[0] = 255;

    loop {
        let timer = unsafe {
            let val: u8;
            core::arch::asm!(r#"lds {out}, 0x46"#, out = out(reg) val);
            val
        };

        data[0] = unsafe { NUM_TIMER0_OVERFLOWS };
        data[3] = data[3].wrapping_add(1);
        hal::upload_bport_data::<0>(&data);
    }
}

#[no_mangle]
pub unsafe extern "avr-interrupt" fn __vector_16() {
    NUM_TIMER0_OVERFLOWS += 1;
}
