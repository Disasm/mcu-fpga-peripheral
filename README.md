# `mcu-fpga-peripheral`

> A demonstration project to show possibility of making your own MCU
> peripherals that work on FPGA

The original idea was to use FMC/FSMC for communication, so that additional
peripherals will be transparently mapped into MCU address space, but this
PoC uses a regular SPI bus for communication.

## Build and run

The following procedure assumes you have WeAct STM32F4x1 MiniF4 v3.0 and
iCEBreaker boards connected to PC. iCEBreaker board should be modified before connecting.

* CS pin (pin 1) of the flash chip should be disconnected from the board
* J15 and J16 should be switched into FPGA mode

```console
cd icebreaker-soc
./soc.py --flash

cd ../example-f411
cargo embed --release
```

Now connect two boards according to the connection table provided at the top of
the `example-f411/src/main.rs` file.
