#![no_std]
#![no_main]
#![feature(const_generics)]

mod display;
mod command_interpreter;

use crate::command_interpreter::interpret_command;
use display::DisplayMode;
use display::text_display::TextDisplay;

use cortex_m::asm::delay;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use stm32f1xx_hal::pac::TIM2;
use stm32f1xx_hal::timer::CountDownTimer;

use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::delay::Delay;
use stm32f1xx_hal::pac::Peripherals;
use stm32f1xx_hal::pac::{interrupt, Interrupt};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::timer::{Event, Timer};
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};

use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use embedded_graphics::drawable::Drawable;
use embedded_graphics::{image::Image, pixelcolor::Rgb888, prelude::Point, DrawTarget};
use tinytga::Tga;

use hub75::{Hub75, Pins};

extern crate panic_semihosting;

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
static mut DELAY: Option<Delay> = None;
static mut INT_TIMER: Option<CountDownTimer<TIM2>> = None;

static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut USB_SERIAL: Option<usbd_serial::SerialPort<UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

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
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    //Matrix pins
    let mut _r1 = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
    let mut _g1 = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
    let mut _b1 = gpiob.pb5.into_push_pull_output(&mut gpiob.crl);
    let mut _r2 = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);
    let mut _g2 = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);
    let mut _b2 = gpiob.pb8.into_push_pull_output(&mut gpiob.crh);
    let mut _a = gpiob.pb9.into_push_pull_output(&mut gpiob.crh);
    let mut _b = gpiob.pb10.into_push_pull_output(&mut gpiob.crh);
    let mut _c = gpiob.pb11.into_push_pull_output(&mut gpiob.crh);
    let mut _clock = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    let mut _latch = gpiob.pb13.into_push_pull_output(&mut gpiob.crh);
    let mut _oe = gpiob.pb14.into_push_pull_output(&mut gpiob.crh);

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

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
    let bus = UsbBus::new(usb);

    let img = include_bytes!("../ferris.tga");
    let tga = Tga::from_slice(img).unwrap();
    let image: Image<Tga, Rgb888> = Image::new(&tga, Point::zero());

    // Unsafe to allow access to static variables
    unsafe {
        USB_BUS = Some(bus);
        USB_SERIAL = Some(SerialPort::new(USB_BUS.as_ref().unwrap()));
        let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build();

        USB_DEVICE = Some(usb_dev);
        DISPLAY = Some(Hub75::new(4, &mut *(0x40010C0C as *mut u16)));

        NVIC::unmask(Interrupt::USB_HP_CAN_TX);
        NVIC::unmask(Interrupt::USB_LP_CAN_RX0);
        NVIC::unmask(Interrupt::TIM2);

        DELAY = Some(Delay::new(p.SYST, clocks));

        image.draw(DISPLAY.as_mut().unwrap()).unwrap();
    }

    let display_mode = DisplayMode::TextMode(TextDisplay::<256>::new());

    let mut timer = Timer::tim2(dp.TIM2, &clocks, &mut rcc.apb1).start_count_down(100.hz());
    timer.listen(Event::Update);

    unsafe {
        INT_TIMER = Some(timer);
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

#[interrupt]
fn TIM2() {
    unsafe {
        DISPLAY
            .as_mut()
            .unwrap()
            .output_bcm(DELAY.as_mut().unwrap(), 1, 100);
        INT_TIMER.as_mut().unwrap().clear_update_interrupt_flag();
    }
}

fn usb_interrupt() {
    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0_u8; 256];

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            let command = interpret_command::<256, 64>(&buf);

            serial.write(&buf).ok();
            //hprintln!("{:?}", buf);
        }
        _ => {}
    }
}
