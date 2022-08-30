avrdude -patmega328p -carduino -P/dev/ttyACM0 -b115200 -D -Uflash:w:target/avr-unknown-gnu-atmega328p/release/leds-arduino.elf:e
