# `mcu-fpga-peripheral`

> A demonstration project to show possibility of making your own MCU
> peripherals that work on FPGA

The original idea was to use FMC/FSMC for communication, so that additional
peripherals will be transparently mapped into MCU address space, but this
PoC uses a regular SPI bus for communication.

## Build and run

The following procedure assumes you have WeAct STM32F4x1 MiniF4 v3.0 and
iCEBreaker boards connected to PC. Additionally, a break-off PMOD should
be connected to PMOD2 port of the board. 

```console
cd icebreaker-soc
./soc.py --flash

cd ../example-f411
openocd &
cargo run --release
```

Now connect two boards according to the connection table provided at the top of
the `example-f411/src/main.rs` file.
 
When you press the `KEY` button on MiniF4, the central LED on break-off should
light up: it starts blinking at the maximum speed. When you release the button,
the LED should stop blinking.
