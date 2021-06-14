#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32f1xx_hal::{prelude::*, stm32};
use stm32f1xx_hal::pac::Peripherals;

use cortex_m::asm::{delay, wfi};
use embedded_hal::digital::v2::OutputPin;

extern crate panic_semihosting;

#[entry]
fn main() -> ! {
    let p = cortex_m::Peripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
   
    loop {
        led.set_high().unwrap();
        delay(clocks.sysclk().0 / 10);
        led.set_low().unwrap();
        delay(clocks.sysclk().0 / 10);
    }
}
