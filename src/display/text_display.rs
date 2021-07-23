use embedded_graphics::text::Text;
use heapless::String;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::Point;
use embedded_graphics::Drawable;

use embedded_graphics::mono_font::ascii::FONT_6X9;
use ibm437::IBM437_8X8_NORMAL;
use profont::PROFONT_7_POINT;

use super::{font::Font, text_animations::TextAnimation};

const ROWS: usize = 3;
const OFFSET: i32 = 3;
#[derive(Debug)]
pub struct TextDisplay<'a, const MAX_ROW_LENGTH: usize> {
    rows: [String<MAX_ROW_LENGTH>; ROWS],
    animation: [TextAnimation; ROWS],
    style: [MonoTextStyle<'a, Rgb888>; ROWS],
}

impl<'a, const MAX_ROW_LENGTH: usize> TextDisplay<'a, MAX_ROW_LENGTH> {
    pub fn new() -> Self {
        let style = MonoTextStyleBuilder::new()
            .text_color(Rgb888::new(255, 255, 255))
            .background_color(Rgb888::BLACK)
            .build();

        TextDisplay {
            rows: [String::from(""), String::from(""), String::from("")],
            animation: [
                TextAnimation::NoAnimation,
                TextAnimation::NoAnimation,
                TextAnimation::NoAnimation,
            ],
            style: [style; 3],
        }
    }

    pub fn write(&mut self, text: String<MAX_ROW_LENGTH>, row: usize) {
        if row >= ROWS {
            return;
        }

        self.rows[row] = text;
    }

    pub fn set_color(&mut self, row: usize, r: u8, g: u8, b: u8) {
        if row >= ROWS {
            return;
        }

        let current_style = &mut self.style[row];

        current_style.text_color = Some(Rgb888::new(r, g, b));
    }

    pub fn set_font(&mut self, row: usize, font: Font) {
        if row >= ROWS {
            return;
        }

        let style = &mut self.style[row];

        match font {
            Font::Default => style.font = &FONT_6X9,
            Font::Ibm => style.font = &IBM437_8X8_NORMAL,
            Font::ProFont => style.font = &PROFONT_7_POINT,
        };
    }

    pub fn set_animation(&mut self, row: usize, animation: TextAnimation) {
        if row >= ROWS {
            return;
        }

        self.animation[row] = animation;
    }

    pub fn update<T: DrawTarget<Color = Rgb888>>(&mut self, target: &mut T) {
        for i in 0..ROWS {
            let current_style = &mut self.style[i];

            Text::new(
                self.rows[i].as_str(),
                Point::new(0, OFFSET + (i as i32 * 9)),
                current_style.clone(),
            )
            .draw(target).ok();
        }
    }
}
