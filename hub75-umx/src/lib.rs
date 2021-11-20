//Based on https://github.com/david-sawatzke/hub75-rs by David Savatzke

// Licensed under either of

//     Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
//     MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)


// Significant changes include:
// - Adjusting gamma correction values
// - Optimization of PWM loop
// - Optimization of I/O for stm32 platform
// - Mapping pixels for 1/8 driving mode matrices
// - Upgrade to Embedded-graphics-07
// - Using const generics to enable variable length of matrix


#![no_std]
#![feature(const_generics)]

use core::usize;

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;

#[cfg(feature = "size-64x64")]
const NUM_ROWS: usize = 32;
#[cfg(not(feature = "size-64x64"))]
const NUM_ROWS: usize = 16;

// gamma 2.8
// const GAMMA8: [u8; 256] = [
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5,
//     5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14,
//     14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25, 25, 26, 27,
//     27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46,
//     47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72,
//     73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104,
//     105, 107, 109, 110, 112, 114, 115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137,
//     138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175,
//     177, 180, 182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
//     223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
// ];

//no corr
// const GAMMA8: [u8; 256] = [
//     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
//     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
//     50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
//     74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97,
//     98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
//     117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135,
//     136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154,
//     155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173,
//     174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192,
//     193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211,
//     212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230,
//     231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249,
//     250, 251, 252, 253, 254, 255,
// ];


//gamma 2.0
const GAMMA8: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4,
    4, 4, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 11, 11, 11, 12, 12, 13, 13, 14,
    14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 23, 23, 24, 24, 25, 26, 26, 27,
    28, 28, 29, 30, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 38, 39, 40, 41, 42, 42, 43, 44, 45,
    46, 47, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67,
    68, 69, 70, 71, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 84, 85, 86, 87, 88, 89, 91, 92, 93, 94,
    95, 97, 98, 99, 100, 102, 103, 104, 105, 107, 108, 109, 111, 112, 113, 115, 116, 117, 119, 120,
    121, 123, 124, 126, 127, 128, 130, 131, 133, 134, 136, 137, 139, 140, 142, 143, 145, 146, 148,
    149, 151, 152, 154, 155, 157, 158, 160, 162, 163, 165, 166, 168, 170, 171, 173, 175, 176, 178,
    180, 181, 183, 185, 186, 188, 190, 192, 193, 195, 197, 199, 200, 202, 204, 206, 207, 209, 211,
    213, 215, 217, 218, 220, 222, 224, 226, 228, 230, 232, 233, 235, 237, 239, 241, 243, 245, 247,
    249, 251, 253, 255,
];

#[derive(PartialEq, Eq)]
pub struct Pins {
    pub r1: u16,
    pub g1: u16,
    pub b1: u16,
    pub r2: u16,
    pub g2: u16,
    pub b2: u16,
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub clock: u16,
    pub latch: u16,
    pub oe: u16,
}
pub struct Hub75<const PIN_POS: Pins, const ROW_LENGTH: usize> {
    //r1, g1, b1, r2, g2, b2, column, row
    #[cfg(not(feature = "stripe-multiplexing"))]
    data: [[(u8, u8, u8, u8, u8, u8); ROW_LENGTH]; NUM_ROWS],

    #[cfg(feature = "stripe-multiplexing")]
    data: [[(u8, u8, u8, u8, u8, u8); ROW_LENGTH]; NUM_ROWS / 2],

    output_port: *mut u16,
}

impl<const PIN_POS: Pins, const ROW_LENGTH: usize> Hub75<PIN_POS, ROW_LENGTH> {
    const PINS: Pins = Pins {
        r1: 1 << PIN_POS.r1,
        g1: 1 << PIN_POS.g1,
        b1: 1 << PIN_POS.b1,
        r2: 1 << PIN_POS.r2,
        g2: 1 << PIN_POS.g2,
        b2: 1 << PIN_POS.b2,
        a: 1 << PIN_POS.a,
        b: 1 << PIN_POS.b,
        c: 1 << PIN_POS.c,
        clock: 1 << PIN_POS.clock,
        latch: 1 << PIN_POS.latch,
        oe: 1 << PIN_POS.oe,
    };

    /// TODO: Write better documentation
    /// color_pins are numbers of pins r1, g1, b1, r2, g2, b2, A, B, C, clock, latch, OE
    pub fn new(output_port: &mut u16) -> Self {

        #[cfg(not(feature = "stripe-multiplexing"))]
        let data = [[(0, 0, 0, 0, 0, 0); ROW_LENGTH]; NUM_ROWS];
        #[cfg(feature = "stripe-multiplexing")]
        let data = [[(0, 0, 0, 0, 0, 0); ROW_LENGTH]; NUM_ROWS / 2];

        Self {
            data,
            output_port,
        }
    }

    /// Output the buffer to the display
    ///
    /// Takes some time and should be called quite often, otherwise the output
    /// will flicker
    pub fn output<DELAY: DelayUs<u8>>(&mut self, delay: &mut DELAY) {
        // PWM cycle
        let brightness = [
            16, 32, 48, 64, 80, 96, 112, 128, 144, 160, 176, 192, 208, 224, 240, 255,
        ];
        for b in brightness.iter() {
            //brightness = (brightness + 1).saturating_mul(self.brightness_step);
            self.output_single(delay, *b);
        }
    }

    pub fn output_single<DELAY: DelayUs<u8>>(&mut self, delay: &mut DELAY, brightness: u8) {
        //hacky, but it's the most efficient way. We need to make sure oe is HIGH when pushing color bits, but only during first iteration.
        //By assigning it here we don't have to check a condition every iteration of inner loop;
        let mut address = Self::PINS.oe;
        let mut output_buffer = 0;

        for (count, row) in self.data.iter().enumerate() {
            for element in row.iter() {
                output_buffer = address;

                //Assuming data pins are connected to consecutive pins of a single port starting ftom P0
                //in this order: r1,g1,b1,r2,g2,b2
                if element.0 >= brightness {
                    output_buffer += Self::PINS.r1;
                }
                if element.1 >= brightness {
                    output_buffer += Self::PINS.g1;
                }
                if element.2 >= brightness {
                    output_buffer += Self::PINS.b1;
                }
                if element.3 >= brightness {
                    output_buffer += Self::PINS.r2;
                }
                if element.4 >= brightness {
                    output_buffer += Self::PINS.g2;
                }
                if element.5 >= brightness {
                    output_buffer += Self::PINS.b2;
                }

                output_buffer += Self::PINS.clock;

                unsafe {
                    *self.output_port = output_buffer;
                    output_buffer -= Self::PINS.clock;
                    *self.output_port = output_buffer;
                }
            }

            output_buffer |= Self::PINS.oe;
            output_buffer &= !Self::PINS.latch;

            unsafe {
                *self.output_port = output_buffer;
            }

            output_buffer |= Self::PINS.latch;

            address = 0;

            if count & 1 != 0 {
                address += Self::PINS.a;
            }
            if count & 2 != 0 {
                address += Self::PINS.b;
            }
            if count & 4 != 0 {
                address += Self::PINS.c;
            }

            output_buffer &= !(Self::PINS.a + Self::PINS.b + Self::PINS.c);
            output_buffer += address;
            output_buffer &= !Self::PINS.oe;

            unsafe {
                *self.output_port = output_buffer;
            }
        }

        //prevents last row from being brighter
        delay.delay_us(60);

        output_buffer |= Self::PINS.oe;
        unsafe {
            *self.output_port = output_buffer;
        }
    }

    /// Clear the output
    ///
    /// It's a bit faster than using the embedded_graphics interface
    /// to do the same
    pub fn clear_display(&mut self) {
        for row in self.data.iter_mut() {
            for e in row.iter_mut() {
                e.0 = 0;
                e.1 = 0;
                e.2 = 0;
                e.3 = 0;
                e.4 = 0;
                e.5 = 0;
            }
        }
    }

    #[cfg(not(feature = "stripe-multiplexing"))]
    fn draw_pixel(
        &mut self,
        item: Pixel<Rgb888>,
    ) -> Result<(), <Hub75<PIN_POS, ROW_LENGTH> as DrawTarget>::Error> {
        let Pixel(coord, color) = item;

        let column = coord[0];
        let row = coord[1];

        if column < 0 || column >= ROW_LENGTH as i32 || row < 0 || row >= (NUM_ROWS * 2) as i32 {
            return Ok(());
        }

        let mut pixel_tuple = &mut self.data[row as usize % NUM_ROWS][column as usize];

        if row > 15 {
            pixel_tuple.3 = GAMMA8[color.r() as usize];
            pixel_tuple.4 = GAMMA8[color.g() as usize];
            pixel_tuple.5 = GAMMA8[color.b() as usize];
        } else {
            pixel_tuple.0 = GAMMA8[color.r() as usize];
            pixel_tuple.1 = GAMMA8[color.g() as usize];
            pixel_tuple.2 = GAMMA8[color.b() as usize];
        }

        Ok(())
    }

    #[cfg(feature = "stripe-multiplexing")]
    fn draw_pixel(
        &mut self,
        item: Pixel<Rgb888>,
    ) -> Result<(), <Hub75<PIN_POS, ROW_LENGTH> as DrawTarget>::Error> {
        let Pixel(coord, color) = item;

        let mut x = coord[0] as usize;
        let mut y = coord[1] as usize;

        if (x < 0 || x >= ROW_LENGTH / 2 || y < 0 || y >= NUM_ROWS * 2) {
            return Ok(());
        }

        let is_top_stripe = (y % NUM_ROWS) < NUM_ROWS / 2;

        let screen_offset = x / 32;

        x = x + (screen_offset * 32);

        if is_top_stripe {
            x = x + 32;
        }

        let column = x;
        let row = y % (NUM_ROWS / 2);

        let mut pixel_tuple = &mut self.data[row as usize][column as usize];

        if y > 15 {
            pixel_tuple.3 = GAMMA8[color.r() as usize];
            pixel_tuple.4 = GAMMA8[color.g() as usize];
            pixel_tuple.5 = GAMMA8[color.b() as usize];
        } else {
            pixel_tuple.0 = GAMMA8[color.r() as usize];
            pixel_tuple.1 = GAMMA8[color.g() as usize];
            pixel_tuple.2 = GAMMA8[color.b() as usize];
        }

        Ok(())
    }
}

use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb888, RgbColor},
    prelude::{OriginDimensions, Size},
    Pixel,
};

impl<const PIN_POS: Pins, const ROW_LENGTH: usize> DrawTarget for Hub75<PIN_POS, ROW_LENGTH> {
    type Error = core::convert::Infallible;
    type Color = Rgb888;

    fn draw_iter<T>(&mut self, item: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = Pixel<Rgb888>>,
    {
        let pixels = item.into_iter();

        for pixel in pixels {
            self.draw_pixel(pixel).unwrap();
        }

        Ok(())
    }

    fn clear(&mut self, color: Rgb888) -> Result<(), Self::Error> {
        #[cfg(not(feature = "stripe-multiplexing"))]
        let rows = NUM_ROWS;
        #[cfg(feature = "stripe-multiplexing")]
        let rows = NUM_ROWS / 2;

        for row in 0..rows {
            for column in 0..ROW_LENGTH {
                let pixel_tuple = &mut self.data[row][column];
                pixel_tuple.0 = GAMMA8[color.r() as usize];
                pixel_tuple.1 = GAMMA8[color.g() as usize];
                pixel_tuple.2 = GAMMA8[color.b() as usize];
                pixel_tuple.3 = GAMMA8[color.r() as usize];
                pixel_tuple.4 = GAMMA8[color.g() as usize];
                pixel_tuple.5 = GAMMA8[color.b() as usize];
            }
        }

        Ok(())
    }
}

impl<const PIN_POS: Pins, const ROW_LENGTH: usize> OriginDimensions for Hub75<PIN_POS, ROW_LENGTH> {
    fn size(&self) -> Size {
        Size {
            #[cfg(not(feature = "stripe-multiplexing"))]
            width: ROW_LENGTH as u32,
            #[cfg(feature = "stripe-multiplexing")]
            width: (ROW_LENGTH as u32) / 2,
            height: (NUM_ROWS * 2) as u32,
        }
    }
}
