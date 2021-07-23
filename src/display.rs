pub mod direct_display;
pub mod text_display;
pub mod text_animations;
pub mod font;

pub use direct_display::DirectDisplay;
pub use text_display::TextDisplay;

pub enum DisplayMode<'a, const MAX_ROW_LENGTH: usize>{
    TextMode(TextDisplay<'a, MAX_ROW_LENGTH>),
    DirectMode(DirectDisplay)
}