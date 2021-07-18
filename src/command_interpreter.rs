use crate::display::font::Font;
use crate::DisplayMode;
use heapless::String;

pub fn interpret_command<const MAX_ROW_LENGTH: usize, const ROW_LENGTH: usize>(
    buffer: &[u8],
) -> Command<MAX_ROW_LENGTH, ROW_LENGTH> {
    let command_id = buffer[0];

    match command_id {
        //0 => param_request
        //1 => switch_mode
        //2 => write
        //3 => set_font
        //4 => set_color
        //5 => set_animation
        //5 => draw_pixel
        //6 => draw_row
        //7 => disable_output
        //8 => enable_output
        //9 => ping
        _ => Command::InvalidCommand,
    }
}

pub enum Command<const MAX_ROW_LENGTH: usize, const ROW_LENGTH: usize> {
    Ping,
    SwitchMode(SwitchMode<MAX_ROW_LENGTH>),
    Write(Write<MAX_ROW_LENGTH>),
    SetFont(SetFont),
    SetColor(SetColor),
    SetAnimation(SetAnimation),
    DrawPixel(DrawPixel),
    DrawRow(DrawRow<ROW_LENGTH>),
    InvalidCommand,
}

pub struct SwitchMode<const MAX_ROW_LENGTH: usize> {
    mode: DisplayMode<MAX_ROW_LENGTH>,
}

// impl SwitchMode{
//     pub fn new(buffer: &[u8]) -> Self{

//         let mode = match buffer[1] {
//             0 => TextDisplay(SomeSettings),
//             1 => DirectDisplay(SomeSettings),
//             _ => return error
//         };

//         SwitchMode{
//             mode
//         }
//     }
// }

pub struct Write<const MAX_ROW_LENGTH: usize> {
    text: String<MAX_ROW_LENGTH>,
    row: usize,
}

impl<const MAX_ROW_LENGTH: usize> Write<MAX_ROW_LENGTH> {
    pub fn new(buffer: &[u8]) -> Self {
        let row = buffer[1] as usize;
        let string = core::str::from_utf8(&buffer[2..]).unwrap_or("Parsing text failed");

        Write {
            text: String::from(string),
            row,
        }
    }
}

pub struct SetFont {
    font: Font,
    row: usize,
}

impl SetFont {}

pub struct SetColor {
    rgb_color: (u8, u8, u8),
    row: usize,
}

pub struct SetAnimation {
    font: u8,
    row: usize,
}

pub struct DrawPixel {
    rgb_color: (u8, u8, u8),
    coords: (usize, usize),
}

pub struct DrawRow<const ROW_LENGTH: usize> {
    rgb_color: [(u8, u8, u8); ROW_LENGTH],
    row: usize,
}


