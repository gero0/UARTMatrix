#![no_std]
#![no_main]
#![feature(const_generics)]

mod display;

use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use hub75::{Hub75, Pins};
use stm32f1xx_hal::pac::Peripherals;
use stm32f1xx_hal::pac::{interrupt, Interrupt};
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use stm32f1xx_hal::{prelude::*, stm32};

use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use cortex_m::asm::{delay, wfi};
use embedded_hal::digital::v2::OutputPin;

extern crate panic_semihosting;

static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut USB_SERIAL: Option<usbd_serial::SerialPort<UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

const DOUBLE_SCREEN_WIDTH: usize = 128;

const PIN_POS: Pins = Pins {
    r1: 0,
    g1: 1,
    b1: 5,
    r2: 6,
    g2: 7,
    b2: 8,
    a: 9,
    b: 10,
    c: 11,
    clock: 12,
    latch: 13,
    oe: 14,
};

static mut DISPLAY: Option<Hub75<PIN_POS, DOUBLE_SCREEN_WIDTH>> = None;

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

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    
    //Matrix pins
    let mut _r1 = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
    let mut _g1 = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
    let mut _b1 = gpiob.pb5.into_push_pull_output(&mut gpiob.crl);
    let mut _r2 = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);
    let mut _b2 = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);
    let mut _g2 = gpiob.pb8.into_push_pull_output(&mut gpiob.crh);
    let mut _a = gpiob.pb9.into_push_pull_output(&mut gpiob.crh);
    let mut _b = gpiob.pb10.into_push_pull_output(&mut gpiob.crh);
    let mut _c = gpiob.pb11.into_push_pull_output(&mut gpiob.crh);
    let mut _clock = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    let mut _latch = gpiob.pb13.into_push_pull_output(&mut gpiob.crh);
    let mut _oe = gpiob.pb14.into_push_pull_output(&mut gpiob.crh);

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low().unwrap();
    delay(clocks.sysclk().0 / 100);

    let usb_dm = gpioa.pa11;
    let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: usb_dm,
        pin_dp: usb_dp,
    };

    // Unsafe to allow access to static variables
    unsafe {
        let bus = UsbBus::new(usb);

        USB_BUS = Some(bus);

        USB_SERIAL = Some(SerialPort::new(USB_BUS.as_ref().unwrap()));

        let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build();

        USB_DEVICE = Some(usb_dev);
    }

    unsafe {
        NVIC::unmask(Interrupt::USB_HP_CAN_TX);
        NVIC::unmask(Interrupt::USB_LP_CAN_RX0);
    }

    loop {
        led.set_high().unwrap();
        delay(clocks.sysclk().0 / 10);
        led.set_low().unwrap();
        delay(clocks.sysclk().0 / 10);
    }
}

#[interrupt]
fn USB_HP_CAN_TX() {
    usb_interrupt();
}

#[interrupt]
fn USB_LP_CAN_RX0() {
    usb_interrupt();
}

fn usb_interrupt() {
    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0u8; 72];

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            // Echo back in upper case

            serial.write(&buf).ok();
            //hprintln!("{:?}", buf);
        }
        _ => {}
    }
}
