#![no_main]
#![no_std]

/*
  Connections:
    B12 - CS_N   - FLASH[nCS]
    B13 - SCK    - FLASH[SCK]
    B14 - MISO   - FLASH[IO1]
    B15 - MOSI   - FLASH[IO0]
    B2  - CRESET - CRESET - add 4.7k pull-down resistor
    A8  - CLK16  - PMOD2[9]
*/

use panic_rtt_target as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::stm32;
use stm32f4xx_hal::spi::Spi;
use stm32f4xx_hal::hal::spi::MODE_0;
use stm32f4xx_hal::delay::Delay;
use rtt_target::{rtt_init_print, rprintln};
use litex_pac::register::MemoryInterface;
use litex_pac::{ctrl, leds};
use litex_pac::{read_reg, write_reg};
use stm32f4xx_hal::gpio::Speed;

const BITSTREAM: &[u8] = include_bytes!("../../icebreaker-soc/build/icebreaker/gateware/icebreaker.bin");

struct SpiMemoryInterface<SPI, CS, RESET, DELAY> {
    spi: SPI,
    cs: CS,
    creset: RESET,
    delay: DELAY,
}

impl<SPI, CS, RESET, DELAY> SpiMemoryInterface<SPI, CS, RESET, DELAY>
where
    SPI: stm32f4xx_hal::hal::blocking::spi::Write<u8>,
    CS: stm32f4xx_hal::hal::digital::v2::OutputPin,
    RESET: stm32f4xx_hal::hal::digital::v2::OutputPin,
    DELAY: stm32f4xx_hal::hal::blocking::delay::DelayUs<u32>,
    SPI::Error: core::fmt::Debug,
{
    pub fn upload_bitstream(&mut self, bitstream: &[u8]) {
        self.creset.set_low().ok();
        self.cs.set_low().ok();
        self.delay.delay_us(10); // >=200ns
        self.creset.set_high().ok();

        self.delay.delay_us(1500); // >=1200us

        self.cs.set_high().ok();
        self.spi.write(&[0]).unwrap();
        self.cs.set_low().ok();

        self.spi.write(bitstream).unwrap();
        self.spi.write(&[0; 6]).unwrap();

        self.cs.set_high().ok();
    }
}

impl<SPI, CS, RESET, DELAY> MemoryInterface for SpiMemoryInterface<SPI, CS, RESET, DELAY>
where
    SPI: stm32f4xx_hal::hal::blocking::spi::Transfer<u8>,
    CS: stm32f4xx_hal::hal::digital::v2::OutputPin,
    RESET: stm32f4xx_hal::hal::digital::v2::OutputPin,
    DELAY: stm32f4xx_hal::hal::blocking::delay::DelayUs<u32>,
    SPI::Error: core::fmt::Debug,
{
    fn read32(&mut self, address: u32) -> u32 {
        let address = (address >> 2).to_le_bytes();

        self.cs.set_low().ok();
        self.delay.delay_us(1);

        let mut buffer = [0x03, address[0], address[1], 0x00, 0xcc, 0xcc, 0xcc, 0xcc];
        self.spi.transfer(&mut buffer).unwrap();

        self.delay.delay_us(1);
        self.cs.set_high().ok();
        self.delay.delay_us(1);

        u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]])
    }

    fn write32(&mut self, address: u32, value: u32) {
        let address = (address >> 2).to_le_bytes();
        let value = value.to_le_bytes();

        self.cs.set_low().ok();
        self.delay.delay_us(1);

        let mut buffer = [0x02, address[0], address[1], value[0], value[1], value[2], value[3]];
        self.spi.transfer(&mut buffer).unwrap();

        self.delay.delay_us(1);
        self.cs.set_high().ok();

        self.delay.delay_us(1);
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(25.mhz()).sysclk(100.mhz()).freeze();

    // Setup MCO
    unsafe {
        // 16MHz output
        let rcc = &*stm32f4xx_hal::stm32::RCC::ptr();
        rcc.cfgr.modify(|_, w| {
            w.mco1().hsi();
            w.mco1pre().div1()
        });
    }

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    let button = gpioa.pa0.into_pull_up_input();
    let mut led = gpioc.pc13.into_open_drain_output();

    let _mco = gpioa.pa8.into_alternate_af0();

    let mut cs = gpiob.pb12.into_push_pull_output();
    cs.set_high().ok();
    let mut creset = gpiob.pb2.into_push_pull_output();
    creset.set_low().ok();
    let sck = gpiob.pb13.into_alternate_af5().set_speed(Speed::VeryHigh);
    let miso = gpiob.pb14.into_alternate_af5().set_speed(Speed::VeryHigh);
    let mosi = gpiob.pb15.into_alternate_af5().set_speed(Speed::VeryHigh);
    let spi = Spi::spi2(dp.SPI2, (sck, miso, mosi), MODE_0, 8_000_000.hz(), clocks);

    let delay = Delay::new(cp.SYST, clocks);

    let mut mem_interface = SpiMemoryInterface {
        spi,
        cs,
        creset,
        delay,
    };
    unsafe {
        let ptr = &mut mem_interface as *mut _;
        litex_pac::register::set_memory_interface(&mut *ptr);
    }

    mem_interface.upload_bitstream(BITSTREAM);

    let ctrl = ctrl::CTRL::take().unwrap();
    let leds = leds::LEDS::take().unwrap();

    rprintln!("SCRATCH: {:08x}", read_reg!(ctrl, ctrl, SCRATCH));
    write_reg!(ctrl, ctrl, SCRATCH, 0xdeadbeef);
    rprintln!("SCRATCH2: {:08x}", read_reg!(ctrl, ctrl, SCRATCH));

    let mut counter = 0u32;
    loop {
        if button.is_low().unwrap() {
            counter = counter.wrapping_add(1);
            led.set_low().ok();

            write_reg!(leds, leds, OUT, hledr1: counter & 1);

            let b = [
                0x11u8.wrapping_add(counter as u8),
                0x22u8.wrapping_add(counter as u8),
                0x33u8.wrapping_add(counter as u8),
                0x44u8.wrapping_add(counter as u8),
            ];
            let value = u32::from_le_bytes(b);

            write_reg!(ctrl, ctrl, SCRATCH, value);
            let value2 = read_reg!(ctrl, ctrl, SCRATCH);

            if value != value2 {
                panic!("Values mismatch: {:#x} => {:#x}", value, value2);
            }

        } else {
            led.set_high().ok();
        }
    }
}

