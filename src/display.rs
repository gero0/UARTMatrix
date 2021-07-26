pub mod direct_display;
pub mod text_display;
pub mod text_animations;
pub mod font;

pub use text_display::TextDisplay;

pub enum DisplayError{
    OutOfBounds,
    IncorrectMode,
    InvalidSetting,
    InvalidCommand,
    DrawError,
}

impl DisplayError{
    pub fn message(&self) -> &'static str{
        match self {
            DisplayError::OutOfBounds => "Index out of Bounds",
            DisplayError::IncorrectMode => "Incorrect Mode",
            DisplayError::InvalidSetting => "Invalid Setting",
            DisplayError::InvalidCommand => "Invalid Command",
            DisplayError::DrawError => "Drawing Error",
        }
    }
}

pub enum DisplayMode<'a, const MAX_ROW_LENGTH: usize>{
    TextMode(TextDisplay<'a, MAX_ROW_LENGTH>),
    DirectMode
}