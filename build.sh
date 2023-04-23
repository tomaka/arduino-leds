AVR_CPU_FREQUENCY_HZ=16000000 cargo +nightly-2023-03-25 build -Z build-std-features=compiler-builtins-mem -Z build-std=core,alloc --target avr-unknown-gnu-atmega328p.json --release
