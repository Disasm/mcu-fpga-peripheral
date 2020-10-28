#![no_main]
#![no_std]

use panic_semihosting as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::stm32;
use stm32f4xx_hal::spi::Spi;
use stm32f4xx_hal::hal::spi::MODE_0;
use stm32f4xx_hal::delay::Delay;
use litex_pac::register::MemoryInterface;
use litex_pac::{ctrl, leds};
use litex_pac::{read_reg, write_reg};

struct SpiMemoryInterface<SPI, CS, DELAY> {
    spi: SPI,
    cs: CS,
    delay: DELAY,
}

impl<SPI, CS, DELAY> MemoryInterface for SpiMemoryInterface<SPI, CS, DELAY>
where
    SPI: stm32f4xx_hal::hal::blocking::spi::Transfer<u8>,
    CS: stm32f4xx_hal::hal::digital::v2::OutputPin,
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
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(25.mhz()).sysclk(100.mhz()).freeze();

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();

    let button = gpioa.pa0.into_pull_up_input();
    let mut led = gpioc.pc13.into_open_drain_output();

    let mut cs = gpioa.pa4.into_push_pull_output();
    cs.set_high().ok();
    let sck = gpioa.pa5.into_alternate_af5();
    let miso = gpioa.pa6.into_alternate_af5();
    let mosi = gpioa.pa7.into_alternate_af5();
    let spi = Spi::spi1(dp.SPI1, (sck, miso, mosi), MODE_0, 8_000_000.hz(), clocks);

    let delay = Delay::new(cp.SYST, clocks);

    let mut mem_interface = SpiMemoryInterface {
        spi,
        cs,
        delay,
    };
    unsafe {
        let ptr = &mut mem_interface as *mut _;
        litex_pac::register::set_memory_interface(&mut *ptr);
    }

    let ctrl = ctrl::CTRL::take().unwrap();
    let leds = leds::LEDS::take().unwrap();

    let mut counter = 0u32;
    loop {
        if button.is_low().unwrap() {
            counter = counter.wrapping_add(1);
            led.set_low().ok();

            write_reg!(leds, leds, OUT, hledr1: counter & 1);

            /*let value = 0xff00ff01;

            write_reg!(ctrl, ctrl, SCRATCH, value);
            let value2 = read_reg!(ctrl, ctrl, SCRATCH);

            if value != value2 {
                panic!("Values mismatch: {:#x} => {:#x}", value, value2);
            }*/

        } else {
            led.set_high().ok();
        }
    }
}

