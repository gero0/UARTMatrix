use super::{font::Font, text_animations::TextAnimation};
use embedded_graphics::drawable::Drawable;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::fonts::Text;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::Point;
use embedded_graphics::style::TextStyle;
use embedded_graphics::style::TextStyleBuilder;
use embedded_graphics::DrawTarget;
use heapless::String;

use ibm437::Ibm437Font8x8Normal;
use profont::ProFont7Point;

const ROWS: usize = 3;
const OFFSET: i32 = 3;

#[derive(Debug)]
enum Style {
    Default(TextStyle<Rgb888, Font6x8>),
    ProFont(TextStyle<Rgb888, ProFont7Point>),
    Ibm(TextStyle<Rgb888, Ibm437Font8x8Normal>),
}

#[derive(Debug)]
pub struct TextDisplay<const MAX_ROW_LENGTH: usize> {
    rows: [String<MAX_ROW_LENGTH>; ROWS],
    animation: [TextAnimation; ROWS],
    style: [Style; ROWS],
}

impl<const MAX_ROW_LENGTH: usize> TextDisplay<MAX_ROW_LENGTH> {
    pub fn new() -> Self {
        let style = TextStyleBuilder::new(Font6x8)
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
            style: [
                Style::Default(style.clone()),
                Style::Default(style.clone()),
                Style::Default(style.clone()),
            ],
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

        match current_style {
            Style::Default(style) => style.text_color = Some(Rgb888::new(r, g, b)),
            Style::ProFont(style) => style.text_color = Some(Rgb888::new(r, g, b)),
            Style::Ibm(style) => style.text_color = Some(Rgb888::new(r, g, b)),
        };
    }

    pub fn set_font(&mut self, row: usize, font: Font) {
        if row >= ROWS {
            return;
        }

        let current_style = &mut self.style[row];

        let mut current_color = Rgb888::new(255, 255, 255);

        match current_style {
            Style::Default(style) => {
                if style.text_color.is_some() {
                    current_color = style.text_color.unwrap();
                }
            }
            Style::ProFont(style) => {
                if style.text_color.is_some() {
                    current_color = style.text_color.unwrap();
                }
            }
            Style::Ibm(style) => {
                if style.text_color.is_some() {
                    current_color = style.text_color.unwrap();
                }
            }
        };

        match font {
            Font::Default => {
                self.style[row] = Style::Default(
                    TextStyleBuilder::new(Font6x8)
                        .text_color(current_color)
                        .background_color(Rgb888::BLACK)
                        .build(),
                );
            }
            Font::Ibm => {
                self.style[row] = Style::Ibm(
                    TextStyleBuilder::new(Ibm437Font8x8Normal)
                        .text_color(current_color)
                        .background_color(Rgb888::BLACK)
                        .build(),
                );
            }
            Font::ProFont => {
                self.style[row] = Style::ProFont(
                    TextStyleBuilder::new(ProFont7Point {})
                        .text_color(current_color)
                        .background_color(Rgb888::BLACK)
                        .build(),
                );
            }
        };
    }

    pub fn set_animation(&mut self, row: usize, animation: TextAnimation) {
        if row >= ROWS {
            return;
        }

        self.animation[row] = animation;
    }

    pub fn update<T: DrawTarget<Rgb888>>(&mut self, target: &mut T) {
        for i in 0..ROWS {
            let current_style = &mut self.style[i];

            match current_style {
                Style::Default(style) => {
                    Text::new(self.rows[i].as_str(), Point::new(0, OFFSET + (i as i32 * 9)))
                        .into_styled(style.clone())
                        .draw(target).ok();
                }
                Style::ProFont(style) => {
                    Text::new(self.rows[i].as_str(), Point::new(0, OFFSET + (i as i32 * 9)))
                        .into_styled(style.clone())
                        .draw(target).ok();
                }
                Style::Ibm(style) => {
                    Text::new(self.rows[i].as_str(), Point::new(0, OFFSET + (i as i32 * 9)))
                        .into_styled(style.clone())
                        .draw(target).ok();
                }
            }
        }
    }
}
