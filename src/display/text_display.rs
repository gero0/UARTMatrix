use crate::SlideAnimation;
use heapless::String;

use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{Rgb888, RgbColor},
    prelude::Point,
    text::Text,
    Drawable,
};

use embedded_graphics::mono_font::ascii::FONT_6X9;
use ibm437::IBM437_8X8_NORMAL;
use profont::PROFONT_7_POINT;

use super::{
    font::Font,
    text_animations::TextAnimation,
    DisplayError,
};

const ROWS: usize = 3;
const OFFSET: i32 = 8;
const LETTER_WIDTH: usize = 9;
const ROW_PX_WIDTH: usize = 64;

pub fn utf8_slice(s: &str, start: usize, end: usize) -> Option<&str> {
    let mut iter = s
        .char_indices()
        .map(|(pos, _)| pos)
        .chain(Some(s.len()))
        .skip(start)
        .peekable();
    let start_pos = *iter.peek()?;
    for _ in start..end {
        iter.next();
    }
    Some(&s[start_pos..*iter.peek()?])
}
#[derive(Debug)]
pub struct TextDisplay<'a, const TEXT_ROW_LENGTH: usize> {
    rows: [String<TEXT_ROW_LENGTH>; ROWS],
    animation: [TextAnimation; ROWS],
    style: [MonoTextStyle<'a, Rgb888>; ROWS],
}

impl<'a, const TEXT_ROW_LENGTH: usize> TextDisplay<'a, TEXT_ROW_LENGTH> {
    pub fn new() -> Self {
        let style = MonoTextStyleBuilder::new()
            .text_color(Rgb888::new(255, 255, 255))
            .background_color(Rgb888::BLACK)
            .font(&FONT_6X9)
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

    pub fn write(&mut self, row: usize, text: String<TEXT_ROW_LENGTH>) -> Result<(), DisplayError> {
        if row >= ROWS {
            return Err(DisplayError::OutOfBounds);
        }

        if let TextAnimation::SlideAnimation(ref mut anim) = &mut self.animation[row] {
            update_slide_length(anim, text.len())
        }

        self.rows[row] = text;

        Ok(())
    }

    pub fn set_color(&mut self, row: usize, rgb_color: (u8, u8, u8)) -> Result<(), DisplayError> {
        if row >= ROWS {
            return Err(DisplayError::OutOfBounds);
        }

        let current_style = &mut self.style[row];

        let (r, g, b) = rgb_color;

        current_style.text_color = Some(Rgb888::new(r, g, b));

        Ok(())
    }

    pub fn set_font(&mut self, row: usize, font: Font) -> Result<(), DisplayError> {
        if row >= ROWS {
            return Err(DisplayError::OutOfBounds);
        }

        let style = &mut self.style[row];

        match font {
            Font::Default => style.font = &FONT_6X9,
            Font::Ibm => style.font = &IBM437_8X8_NORMAL,
            Font::ProFont => style.font = &PROFONT_7_POINT,
        };

        Ok(())
    }

    pub fn set_animation(
        &mut self,
        row: usize,
        mut animation: TextAnimation,
    ) -> Result<(), DisplayError> {
        if row >= ROWS {
            return Err(DisplayError::OutOfBounds);
        }

        if let TextAnimation::SlideAnimation(ref mut anim) = animation {
            update_slide_length(anim, self.rows[row].len())
        }

        self.animation[row] = animation;

        Ok(())
    }

    pub fn update<T: DrawTarget<Color = Rgb888>>(&mut self, target: &mut T) {
        for i in 0..ROWS {
            let anim_state = self.animation[i].get();

            if anim_state.visible {
                match self.animation[i] {
                    TextAnimation::SlideAnimation(_anim) => {
                        let mut string = String::<512>::from(" ");

                        let anim_offset = anim_state.x_offset;
                        let glyph_width = self.style[i].font.character_size.width;

                        if anim_offset > 0 {
                            let pixels_still_on_screen = ROW_PX_WIDTH as i32 - anim_offset;
                            if pixels_still_on_screen > 0 {
                                let mut characters_still_on_screen =
                                    (pixels_still_on_screen as usize / glyph_width as usize) + 1;

                                let char_length = self.rows[i].chars().count();

                                if characters_still_on_screen >= char_length {
                                    characters_still_on_screen = char_length - 1;
                                }

                                string
                                    .push_str(
                                        utf8_slice(
                                            &self.rows[i],
                                            0,
                                            characters_still_on_screen + 1,
                                        )
                                        .unwrap(),
                                    )
                                    .ok();
                            }

                            string.push_str(" ").ok();

                            Text::new(
                                string.as_str(),
                                Point::new(
                                    anim_offset,
                                    OFFSET + (i as i32 * 9) + anim_state.y_offset,
                                ),
                                self.style[i].clone(),
                            )
                            .draw(target)
                            .ok();
                        } else {
                            let characters_skipped =
                                anim_offset.abs() as usize / glyph_width as usize;

                            let x_offset = anim_offset % glyph_width as i32;

                            if characters_skipped < self.rows[i].len() {
                                let characters_that_will_fit =
                                    (ROW_PX_WIDTH / glyph_width as usize) + 1;

                                let mut last_index = characters_skipped + characters_that_will_fit;

                                let char_length = self.rows[i].chars().count();

                                if last_index >= char_length {
                                    last_index = char_length - 1;
                                }

                                if last_index >= characters_skipped {
                                    string
                                        .push_str(
                                            utf8_slice(
                                                &self.rows[i],
                                                characters_skipped,
                                                last_index + 1,
                                            )
                                            .unwrap_or_else(|| {
                                                panic!(
                                                    "b: {}, e:{}",
                                                    characters_skipped, last_index
                                                )
                                            }),
                                        )
                                        .ok();
                                }
                            }

                            string.push_str(" ").ok();

                            Text::new(
                                string.as_str(),
                                Point::new(x_offset, OFFSET + (i as i32 * 9) + anim_state.y_offset),
                                self.style[i].clone(),
                            )
                            .draw(target)
                            .ok();
                        }
                    }
                    _ => {
                        Text::new(
                            self.rows[i].as_str(),
                            Point::new(
                                0 + anim_state.x_offset,
                                OFFSET + (i as i32 * 9) + anim_state.y_offset,
                            ),
                            self.style[i].clone(),
                        )
                        .draw(target)
                        .ok();
                    }
                }
            } else {
                //Because clearing the display would take too much time ;)
                Text::new(
                    "                                  ",
                    Point::new(
                        0 + anim_state.x_offset,
                        OFFSET + (i as i32 * 9) + anim_state.y_offset,
                    ),
                    self.style[i].clone(),
                )
                .draw(target)
                .ok();
            }
        }
    }

    pub fn anim_tick(&mut self) {
        for i in 0..ROWS {
            self.animation[i].tick();
        }
    }
}

fn update_slide_length(anim: &mut SlideAnimation, text_len: usize) {
    anim.set_length((text_len + 2) * LETTER_WIDTH);
}
