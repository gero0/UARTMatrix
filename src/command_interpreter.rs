use crate::display::{font::Font, text_animations::TextAnimation, DisplayError, TextDisplay};
use crate::DisplayMode;
use embedded_graphics::draw_target::DrawTarget;
use heapless::String;

pub fn interpret_command<const MAX_ROW_LENGTH: usize, const ROW_LENGTH: usize>(
    buffer: &[u8],
) -> Result<Command<MAX_ROW_LENGTH, ROW_LENGTH>, DisplayError> {
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
        8 => Ok(Command::EnableOutput),
        9 => Ok(Command::DisableOutput),
        10 => Ok(Command::Ping),
        _ => Err(DisplayError::InvalidCommand),
    }
}

pub fn execute_command<T: DrawTarget, const MAX_ROW_LENGTH: usize>(
    mode: &mut DisplayMode<MAX_ROW_LENGTH>,
    target: &mut T,
) {
}

pub enum Command<'a, const MAX_ROW_LENGTH: usize, const ROW_LENGTH: usize> {
    Ping,
    ParamRequest,
    DisableOutput,
    EnableOutput,
    SwitchMode(SwitchMode<'a, MAX_ROW_LENGTH>),
    Write(Write<MAX_ROW_LENGTH>),
    SetFont(SetFont),
    SetColor(SetColor),
    SetAnimation(SetAnimation),
    DrawPixel(DrawPixel),
    DrawRow(DrawRow<ROW_LENGTH>),
}

pub struct SwitchMode<'a, const MAX_ROW_LENGTH: usize> {
    mode: DisplayMode<'a, MAX_ROW_LENGTH>,
}

impl<'a, const MAX_ROW_LENGTH: usize> SwitchMode<'a, MAX_ROW_LENGTH> {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let mode = match buffer[1] {
            0 => DisplayMode::TextMode(TextDisplay::new()),
            1 => DisplayMode::DirectMode,
            _ => return Err(DisplayError::InvalidSetting),
        };

        Ok(SwitchMode { mode })
    }
}

pub struct Write<const MAX_ROW_LENGTH: usize> {
    text: String<MAX_ROW_LENGTH>,
    row: usize,
}

impl<const MAX_ROW_LENGTH: usize> Write<MAX_ROW_LENGTH> {
    pub fn new(buffer: &[u8]) -> Result<Self, DisplayError> {
        let row = buffer[1] as usize;
        let string =
            core::str::from_utf8(&buffer[2..]).map_err(|_| DisplayError::InvalidSetting)?;

        Ok(Write {
            text: String::from(string),
            row,
        })
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
}
