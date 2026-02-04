# LPC176x5x Hardware Abstraction Layer

Work in progress

## Blinky

```bash
cargo build --release --example blinky
arm-none-eabi-objcopy -O ihex target/thumbv7m-none-eabi/release/examples/blinky rust_blinky.hex
lpc21isp -wipe -hex -control rust_blinky.hex /dev/ttyUSB0 38400 4000
```
