#![no_std]
#![no_main]
#![feature(const_generics)]

mod command_interpreter;
mod display;
mod uart;

use crate::command_interpreter::interpret_command;
use crate::display::text_animations::BlinkingAnimation;
use crate::display::text_animations::SlideAnimation;
use crate::display::text_animations::SlideDirection;
use crate::display::text_animations::TextAnimation;
use crate::uart::UartController;
use cortex_m_semihosting::hprintln;
use display::text_display::TextDisplay;
use display::DisplayMode;
use heapless::String;
use stm32f1xx_hal::gpio::gpioa::PA10;
use stm32f1xx_hal::gpio::gpioa::PA9;
use stm32f1xx_hal::gpio::Alternate;
use stm32f1xx_hal::gpio::Floating;
use stm32f1xx_hal::gpio::Input;
use stm32f1xx_hal::gpio::PushPull;
use stm32f1xx_hal::pac::TIM3;

use cortex_m::asm::delay;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use stm32f1xx_hal::pac::TIM2;
use stm32f1xx_hal::pac::USART1;
use stm32f1xx_hal::serial::Config;
use stm32f1xx_hal::serial::Serial;
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

use display::font::Font;

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

static mut SERIAL: Option<Serial<USART1, (PA9<Alternate<PushPull>>, PA10<Input<Floating>>)>> = None;
static mut UARTCONTROLLER: Option<UartController<512>> = None;

static mut DISPLAY: Option<Hub75<PIN_POS, DOUBLE_SCREEN_WIDTH>> = None;
static mut DISPLAY_MODE: DisplayMode<256> = DisplayMode::DirectMode;
static mut DELAY: Option<Delay> = None;
static mut DRAW_TIMER: Option<CountDownTimer<TIM2>> = None;
static mut ANIM_TIMER: Option<CountDownTimer<TIM3>> = None;
static mut OUTPUT_ENABLED: bool = true;

static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut USB_SERIAL: Option<usbd_serial::SerialPort<UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

#[entry]
fn main() -> ! {
    let mut p = cortex_m::Peripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

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

    // USART1
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    let mut serial = Serial::usart1(
        dp.USART1,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(9600.bps()),
        clocks,
        &mut rcc.apb2,
    );

    serial.listen(stm32f1xx_hal::serial::Event::Rxne);

    unsafe {
        UARTCONTROLLER = Some(UartController::new());
        SERIAL = Some(serial);
    }

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

        p.NVIC.set_priority(Interrupt::USART1, 16);
        p.NVIC.set_priority(Interrupt::TIM2, 32);
        p.NVIC.set_priority(Interrupt::TIM3, 32);
        p.NVIC.set_priority(Interrupt::USB_HP_CAN_TX, 64);
        p.NVIC.set_priority(Interrupt::USB_LP_CAN_RX0, 64);

        NVIC::unmask(Interrupt::USART1);
        NVIC::unmask(Interrupt::USB_HP_CAN_TX);
        NVIC::unmask(Interrupt::USB_LP_CAN_RX0);
        NVIC::unmask(Interrupt::TIM2);
        NVIC::unmask(Interrupt::TIM3);

        DELAY = Some(Delay::new(p.SYST, clocks));
    }

    unsafe {
        DISPLAY_MODE = DisplayMode::TextMode(TextDisplay::<256>::new());

        match &mut DISPLAY_MODE {
            DisplayMode::TextMode(tm) => {
                tm.write(0, String::from("Rust is cool")).ok();
                tm.write(1, String::from("AAAAAAAAAAA")).ok();
                tm.write(2, String::from("Man i love crabs (\\/) (°,,°) (\\/)"))
                    .ok();
                tm.set_animation(
                    0,
                    TextAnimation::SlideAnimation(SlideAnimation::new(4, SlideDirection::Left)),
                )
                .ok();
                tm.set_animation(
                    1,
                    TextAnimation::BlinkingAnimation(BlinkingAnimation::new(64)),
                )
                .ok();
                tm.set_animation(
                    2,
                    TextAnimation::SlideAnimation(SlideAnimation::new(4, SlideDirection::Left)),
                )
                .ok();
                tm.set_color(0, (128, 128, 128)).ok();
                tm.set_color(1, (255, 0, 0)).ok();
                tm.set_color(2, (240, 120, 0)).ok();
                tm.set_font(1, Font::Ibm).ok();
                tm.set_font(2, Font::ProFont).ok();
            }
            _ => {}
        }
    }
    let mut draw_timer = Timer::tim2(dp.TIM2, &clocks, &mut rcc.apb1).start_count_down(100.hz());
    let mut anim_timer = Timer::tim3(dp.TIM3, &clocks, &mut rcc.apb1).start_count_down(60.hz());

    draw_timer.listen(Event::Update);
    anim_timer.listen(Event::Update);

    unsafe {
        DRAW_TIMER = Some(draw_timer);
        ANIM_TIMER = Some(anim_timer);
    }
    loop {
        unsafe {
            match &mut DISPLAY_MODE {
                DisplayMode::TextMode(tm) => tm.update(DISPLAY.as_mut().unwrap()),
                _ => {}
            };
        }
    }
}

#[interrupt]
unsafe fn USART1() {
    let serial = SERIAL.as_mut().unwrap();
    let result = serial.read();
    if let Ok(byte) = result {
        UARTCONTROLLER.as_mut().unwrap().read_byte(byte);
    }

    let command = UARTCONTROLLER.as_mut().unwrap().get_command();

    if let Some(c) = command {
        let response = parse_command(&c);
    }

    serial.listen(stm32f1xx_hal::serial::Event::Rxne);
}

#[interrupt]
unsafe fn TIM2() {
    if OUTPUT_ENABLED {
        DISPLAY
            .as_mut()
            .unwrap()
            .output_bcm(DELAY.as_mut().unwrap(), 1, 100);
    }
    DRAW_TIMER.as_mut().unwrap().clear_update_interrupt_flag();
}

#[interrupt]
unsafe fn TIM3() {
    match &mut DISPLAY_MODE {
        DisplayMode::TextMode(tm) => tm.anim_tick(),
        _ => {}
    };
    ANIM_TIMER.as_mut().unwrap().clear_update_interrupt_flag();
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

    let mut buf = [0_u8; 512];

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            let response = parse_command(&buf[4..]);
            serial.write(response).ok();
        }
        _ => {}
    }
}

fn parse_command(buffer: &[u8]) -> &[u8] {
    let command = interpret_command::<256, 64>(&buffer);
    match command {
        Ok(command) => unsafe {
            let result = command.execute(
                &mut DISPLAY_MODE,
                DISPLAY.as_mut().unwrap(),
                &mut OUTPUT_ENABLED,
            );

            match result {
                Ok(_) => {
                    return "OK".as_bytes();
                }
                Err(e) => {
                    return e.message().as_bytes();
                }
            };
        },
        Err(e) => {
            return e.message().as_bytes();
        }
    }
}
