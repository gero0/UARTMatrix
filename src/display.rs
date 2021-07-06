pub mod direct_display;
pub mod text_display;
pub mod text_animations;
pub mod font;

pub use direct_display::DirectDisplay;
pub use text_display::TextDisplay;

enum DisplayModes<const MAX_ROW_LENGTH: usize>{
    TextMode(TextDisplay<MAX_ROW_LENGTH>),
    DirectMode(DirectDisplay)
}