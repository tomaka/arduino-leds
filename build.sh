AVR_CPU_FREQUENCY_HZ=16000000 cargo +nightly build -Z build-std-features=compiler-builtins-mem -Z build-std=core --target avr-unknown-gnu-atmega328p.json --release
