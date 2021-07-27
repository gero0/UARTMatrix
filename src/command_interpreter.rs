use crate::display::{font::Font, text_animations::TextAnimation, DisplayError, TextDisplay};
use crate::DisplayMode;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::Point;
use embedded_graphics::{Drawable, Pixel};
use heapless::String;

pub fn interpret_command<const TEXT_ROW_LENGTH: usize, const ROW_LENGTH: usize>(
    buffer: &[u8],
) -> Result<Command<TEXT_ROW_LENGTH, ROW_LENGTH>, DisplayError> {
    let command_id = buffer[0];

    match command_id {
        0 => Ok(Command::ParamRequest),
        1 => Ok(Command::SwitchMode(SwitchMode::new(&buffer)?)),
        2 => Ok(Command::Write(Write::new(&buffer)?)),
        3 => Ok(Command::SetFont(SetFont::new(&buffer)?)),
        4 => Ok(Command::SetColor(SetColor::new(&buffer)?)),
        5 => Ok(Command::SetAnimation(SetAnimation::new(&buffer)?)),
        6 => Ok(Command::DrawPixel(DrawPixel::new(&buffer)?)),
        7 => Ok(Command::DrawRow(DrawRow::new(&buffer)?)),
        8 => Ok(Command::Clear),
        9 => Ok(Command::EnableOutput),
        10 => Ok(Command::DisableOutput),
        11 => Ok(Command::Ping),
        _ => Err(DisplayError::InvalidCommand),
    }
}

pub enum Command<const TEXT_ROW_LENGTH: usize, const ROW_LENGTH: usize> {
    Ping,
    ParamRequest,
    DisableOutput,
    EnableOutput,
    Clear,
    SwitchMode(SwitchMode<TEXT_ROW_LENGTH>),
    Write(Write<TEXT_ROW_LENGTH>),
    SetFont(SetFont),
    SetColor(SetColor),
    SetAnimation(SetAnimation),
    DrawPixel(DrawPixel),
    DrawRow(DrawRow<ROW_LENGTH>),
}

impl<const TEXT_ROW_LENGTH: usize, const ROW_LENGTH: usize> Command<TEXT_ROW_LENGTH, ROW_LENGTH> {
    pub fn execute<T: DrawTarget<Color = Rgb888>>(
        self,
        mode: &mut DisplayMode<TEXT_ROW_LENGTH>,
        target: &mut T,
    ) -> Result<&'static str, DisplayError> {
        match mode {
            DisplayMode::TextMode(text_display) => match self {
                Command::Write(write) => write.execute(text_display)?,
                Command::SetFont(set_font) => set_font.execute(text_display)?,
                Command::SetColor(set_color) => set_color.execute(text_display)?,
                Command::SetAnimation(set_animation) => set_animation.execute(text_display)?,
                Command::Ping => return Ok("Pong"),
                Command::ParamRequest => {
                    return Ok("Mode:Text");
                }
                Command::DisableOutput => {
                    //TODO: Implement
                }
                Command::EnableOutput => {
                    //TODO: Implement
                }
                Command::SwitchMode(switch_mode) => {
                    switch_mode.execute(mode, target)?;
                }
                _ => return Err(DisplayError::IncorrectMode),
            },
            DisplayMode::DirectMode => match self {
                Command::DrawPixel(draw_pixel) => draw_pixel.execute(target)?,
                Command::DrawRow(draw_row) => draw_row.execute(target)?,
                Command::Clear => {
                    target.clear(Rgb888::new(0, 0, 0)).ok();
                    return Ok("cleared");
                }
                Command::Ping => return Ok("Pong"),
                Command::ParamRequest => {
                    return Ok("Mode:Direct");
                }
                Command::DisableOutput => {
                    //TODO: Implement
                }
                Command::EnableOutput => {
                    //TODO: Implement
                }
                Command::SwitchMode(switch_mode) => {
                    switch_mode.execute(mode, target)?;
                }
                _ => return Err(DisplayError::IncorrectMode),
            },
        }
        Ok("OK")
    }
}

pub struct SwitchMode<const MAX_ROW_LENGTH: usize> {
    mode: u8,
}

impl<const TEXT_ROW_LENGTH: usize> SwitchMode<TEXT_ROW_LENGTH> {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let mode = buffer[1];
        Ok(SwitchMode { mode })
    }

    pub fn execute<T: DrawTarget<Color = Rgb888>>(
        self,
        mode: &mut DisplayMode<TEXT_ROW_LENGTH>,
        target: &mut T,
    ) -> Result<(), DisplayError> {
        match self.mode {
            0 => {
                *mode = DisplayMode::TextMode(TextDisplay::new());
                target.clear(Rgb888::new(0, 0, 0)).ok();
            }
            1 => {
                *mode = DisplayMode::DirectMode;
                target.clear(Rgb888::new(0, 0, 0)).ok();
            }
            _ => return Err(DisplayError::InvalidCommand),
        }
        Ok(())
    }
}

pub struct Write<const TEXT_ROW_LENGTH: usize> {
    text: String<TEXT_ROW_LENGTH>,
    row: usize,
}

impl<const TEXT_ROW_LENGTH: usize> Write<TEXT_ROW_LENGTH> {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let row = buffer[1] as usize;
        let string =
            core::str::from_utf8(&buffer[2..]).map_err(|_| DisplayError::InvalidSetting)?;

        Ok(Write {
            text: String::from(string),
            row,
        })
    }

    pub fn execute(self, target: &mut TextDisplay<TEXT_ROW_LENGTH>) -> Result<(), DisplayError> {
        target.write(self.row, self.text)?;

        Ok(())
    }
}

pub struct SetFont {
    font: Font,
    row: usize,
}

impl SetFont {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let row = buffer[1] as usize;
        let font = match buffer[2] {
            0 => Font::Default,
            1 => Font::ProFont,
            2 => Font::Ibm,
            _ => {
                return Err(DisplayError::InvalidSetting);
            }
        };

        Ok(SetFont { row, font })
    }

    pub fn execute<const TEXT_ROW_LENGTH: usize>(
        self,
        target: &mut TextDisplay<TEXT_ROW_LENGTH>,
    ) -> Result<(), DisplayError> {
        target.set_font(self.row, self.font)?;

        Ok(())
    }
}

pub struct SetColor {
    rgb_color: (u8, u8, u8),
    row: usize,
}

impl SetColor {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let row = buffer[1] as usize;
        let rgb_color = (buffer[2], buffer[3], buffer[4]);

        Ok(SetColor { rgb_color, row })
    }

    pub fn execute<const TEXT_ROW_LENGTH: usize>(
        self,
        target: &mut TextDisplay<TEXT_ROW_LENGTH>,
    ) -> Result<(), DisplayError> {
        target.set_color(self.row, self.rgb_color)?;

        Ok(())
    }
}

pub struct SetAnimation {
    animation: TextAnimation,
    row: usize,
}

impl SetAnimation {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let row = buffer[1] as usize;
        let animation = match buffer[2] {
            0 => TextAnimation::NoAnimation,
            1 => TextAnimation::BlinkingAnimation,
            2 => TextAnimation::SlideAnimation,
            _ => return Err(DisplayError::InvalidSetting),
        };

        Ok(SetAnimation { row, animation })
    }

    pub fn execute<const TEXT_ROW_LENGTH: usize>(
        self,
        target: &mut TextDisplay<TEXT_ROW_LENGTH>,
    ) -> Result<(), DisplayError> {
        target.set_animation(self.row, self.animation)?;

        Ok(())
    }
}

pub struct DrawPixel {
    rgb_color: (u8, u8, u8),
    coords: (usize, usize),
}

impl DrawPixel {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let coords = (buffer[1] as usize, buffer[2] as usize);
        let rgb_color = (buffer[3], buffer[4], buffer[5]);

        Ok(DrawPixel { rgb_color, coords })
    }

    pub fn execute<T: DrawTarget<Color = Rgb888>>(
        self,
        target: &mut T,
    ) -> Result<(), DisplayError> {
        let (x, y) = self.coords;
        let (r, g, b) = self.rgb_color;

        let pixel = Pixel(Point::new(x as i32, y as i32), Rgb888::new(r, g, b));

        pixel.draw(target).map_err(|_| DisplayError::DrawError)?;

        Ok(())
    }
}

pub struct DrawRow<const ROW_LENGTH: usize> {
    rgb_color: [(u8, u8, u8); ROW_LENGTH],
    row: usize,
}

impl<const ROW_LENGTH: usize> DrawRow<ROW_LENGTH> {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let row = buffer[1] as usize;
        let mut rgb_color = [(0, 0, 0); ROW_LENGTH];

        for i in 0..ROW_LENGTH {
            let offset = 2 + (i * 3);
            let color = (buffer[offset], buffer[offset + 1], buffer[offset + 2]);
            rgb_color[i] = color;
        }

        Ok(Self { row, rgb_color })
    }

    pub fn execute<T: DrawTarget<Color = Rgb888>>(
        self,
        target: &mut T,
    ) -> Result<(), DisplayError> {
        let y = self.row;

        let pixels =
            self.rgb_color.iter().enumerate().map(|(x, (r, g, b))| {
                Pixel(Point::new(x as i32, y as i32), Rgb888::new(*r, *g, *b))
            });

        target
            .draw_iter(pixels)
            .map_err(|_| DisplayError::DrawError)?;

        Ok(())
    }
}
