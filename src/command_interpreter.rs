use crate::{
    display::{font::Font, text_animations::TextAnimation, DisplayError, TextDisplay},
    BlinkingAnimation, DisplayMode, SlideAnimation, SlideDirection,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb888,
    prelude::{Point, Primitive},
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Triangle},
    Drawable, Pixel,
};
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
        8 => Ok(Command::DrawLine(DrawLine::new(&buffer)?)),
        9 => Ok(Command::DrawRectangle(DrawRectangle::new(&buffer)?)),
        10 => Ok(Command::DrawTriangle(DrawTriangle::new(&buffer)?)),
        11 => Ok(Command::DrawCircle(DrawCircle::new(&buffer)?)),
        12 => Ok(Command::Clear),
        13 => Ok(Command::EnableOutput),
        14 => Ok(Command::DisableOutput),
        15 => Ok(Command::Ping),
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
    DrawLine(DrawLine),
    DrawRectangle(DrawRectangle),
    DrawTriangle(DrawTriangle),
    DrawCircle(DrawCircle),
}

impl<const TEXT_ROW_LENGTH: usize, const ROW_LENGTH: usize> Command<TEXT_ROW_LENGTH, ROW_LENGTH> {
    pub fn execute<T: DrawTarget<Color = Rgb888>>(
        self,
        mode: &mut DisplayMode<TEXT_ROW_LENGTH>,
        target: &mut T,
        oe: &mut bool,
    ) -> Result<&'static str, DisplayError> {
        match mode {
            DisplayMode::TextMode(text_display) => {
                match self {
                    Command::Write(write) => write.execute(text_display)?,
                    Command::SetFont(set_font) => set_font.execute(text_display)?,
                    Command::SetColor(set_color) => set_color.execute(text_display)?,
                    Command::SetAnimation(set_animation) => set_animation.execute(text_display)?,
                    Command::Ping => return Ok("Pong"),
                    Command::ParamRequest => return Ok("Mode:Text"),
                    Command::DisableOutput => {
                        *oe = false;
                    }
                    Command::EnableOutput => {
                        *oe = true;
                    }
                    Command::SwitchMode(switch_mode) => switch_mode.execute(mode, target)?,
                    _ => return Err(DisplayError::IncorrectMode),
                }
                target.clear(Rgb888::new(0, 0, 0)).ok();
            }
            DisplayMode::DirectMode => match self {
                Command::DrawPixel(draw_pixel) => draw_pixel.execute(target)?,
                Command::DrawRow(draw_row) => draw_row.execute(target)?,
                Command::DrawLine(draw_line) => draw_line.execute(target)?,
                Command::DrawRectangle(draw_rectangle) => draw_rectangle.execute(target)?,
                Command::DrawTriangle(draw_triangle) => draw_triangle.execute(target)?,
                Command::DrawCircle(draw_circle) => draw_circle.execute(target)?,
                Command::Clear => {
                    target.clear(Rgb888::new(0, 0, 0)).ok();
                    return Ok("cleared");
                }
                Command::Ping => return Ok("Pong"),
                Command::ParamRequest => {
                    return Ok("Mode:Direct");
                }
                Command::DisableOutput => {
                    *oe = false;
                }
                Command::EnableOutput => {
                    *oe = true;
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
        let terminator = buffer[2..].iter().position(|e| e.clone() == 0).unwrap() + 2;

        let string = core::str::from_utf8(&buffer[2..terminator])
            .map_err(|_| DisplayError::InvalidSetting)?;

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
            1 => {
                let anim = BlinkingAnimation::new(buffer[3] as i32);
                TextAnimation::BlinkingAnimation(anim)
            }
            2 => {
                let dir = match buffer[4] {
                    1 => SlideDirection::Right,
                    _ => SlideDirection::Left,
                };

                let anim = SlideAnimation::new(buffer[3] as i32, dir);
                TextAnimation::SlideAnimation(anim)
            }
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

pub struct DrawLine {
    point_a: (u8, u8),
    point_b: (u8, u8),
    thickness: u8,
    color: Rgb888,
}

impl DrawLine {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        Ok(DrawLine {
            point_a: (buffer[1], buffer[2]),
            point_b: (buffer[3], buffer[4]),
            thickness: buffer[5],
            color: Rgb888::new(buffer[6], buffer[7], buffer[8]),
        })
    }

    pub fn execute<T: DrawTarget<Color = Rgb888>>(
        self,
        target: &mut T,
    ) -> Result<(), DisplayError> {
        let (x1, y1) = self.point_a;
        let (x2, y2) = self.point_b;
        Line::new(
            Point::new(x1 as i32, y1 as i32),
            Point::new(x2 as i32, y2 as i32),
        )
        .into_styled(PrimitiveStyle::with_stroke(
            self.color,
            self.thickness as u32,
        ))
        .draw(target)
        .map_err(|_| DisplayError::DrawError)?;

        Ok(())
    }
}

pub struct DrawRectangle {
    point_a: (u8, u8),
    point_b: (u8, u8),
    thickness: u8,
    color: Rgb888,
    filled: bool,
}

impl DrawRectangle {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let filled = match buffer[9] {
            0 => false,
            _ => true,
        };

        Ok(DrawRectangle {
            point_a: (buffer[1], buffer[2]),
            point_b: (buffer[3], buffer[4]),
            thickness: buffer[5],
            color: Rgb888::new(buffer[6], buffer[7], buffer[8]),
            filled,
        })
    }

    pub fn execute<T: DrawTarget<Color = Rgb888>>(
        self,
        target: &mut T,
    ) -> Result<(), DisplayError> {
        let (x1, y1) = self.point_a;
        let (x2, y2) = self.point_b;

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(self.color)
            .stroke_width(self.thickness as u32);

        if self.filled {
            style.fill_color(self.color);
        }

        let style = style.build();

        Rectangle::with_corners(
            Point::new(x1 as i32, y1 as i32),
            Point::new(x2 as i32, y2 as i32),
        )
        .into_styled(style)
        .draw(target)
        .map_err(|_| DisplayError::DrawError)?;

        Ok(())
    }
}

pub struct DrawTriangle {
    point_a: (u8, u8),
    point_b: (u8, u8),
    point_c: (u8, u8),
    thickness: u8,
    color: Rgb888,
    filled: bool,
}

impl DrawTriangle {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let filled = match buffer[11] {
            0 => false,
            _ => true,
        };

        Ok(DrawTriangle {
            point_a: (buffer[1], buffer[2]),
            point_b: (buffer[3], buffer[4]),
            point_c: (buffer[5], buffer[6]),
            thickness: buffer[7],
            color: Rgb888::new(buffer[8], buffer[9], buffer[10]),
            filled,
        })
    }

    pub fn execute<T: DrawTarget<Color = Rgb888>>(
        self,
        target: &mut T,
    ) -> Result<(), DisplayError> {
        let (x1, y1) = self.point_a;
        let (x2, y2) = self.point_b;
        let (x3, y3) = self.point_c;

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(self.color)
            .stroke_width(self.thickness as u32);

        if self.filled {
            style.fill_color(self.color);
        }

        let style = style.build();

        Triangle::new(
            Point::new(x1 as i32, y1 as i32),
            Point::new(x2 as i32, y2 as i32),
            Point::new(x3 as i32, y3 as i32),
        )
        .into_styled(style)
        .draw(target)
        .map_err(|_| DisplayError::DrawError)?;

        Ok(())
    }
}

pub struct DrawCircle {
    center: (u8, u8),
    radius: u8,
    thickness: u8,
    color: Rgb888,
    filled: bool,
}

impl DrawCircle {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let filled = match buffer[7] {
            0 => false,
            _ => true,
        };

        Ok(DrawCircle {
            center: (buffer[1], buffer[2]),
            radius: buffer[3],
            thickness: buffer[4],
            color: Rgb888::new(buffer[5], buffer[6], buffer[7]),
            filled,
        })
    }

    pub fn execute<T: DrawTarget<Color = Rgb888>>(
        self,
        target: &mut T,
    ) -> Result<(), DisplayError> {
        let (x1, y1) = self.center;

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(self.color)
            .stroke_width(self.thickness as u32);

        if self.filled {
            style.fill_color(self.color);
        }

        let style = style.build();

        Circle::new(Point::new(x1 as i32, y1 as i32), self.radius as u32)
            .into_styled(style)
            .draw(target)
            .map_err(|_| DisplayError::DrawError)?;

        Ok(())
    }
}
