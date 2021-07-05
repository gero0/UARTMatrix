use core::char::MAX;

use super::text_animations::TextAnimation;
use heapless::String;

#[derive(Debug)]
pub struct TextDisplay<const MAX_ROW_LENGTH: usize> {
    rows: [String<MAX_ROW_LENGTH>; 3],
    animation: [TextAnimation; 3],
}

impl<const MAX_ROW_LENGTH: usize> TextDisplay<MAX_ROW_LENGTH> {
    pub fn new() -> Self {
        TextDisplay {
            rows: [String::from(""), String::from(""), String::from("")],
            animation: [
                TextAnimation::NoAnimation,
                TextAnimation::NoAnimation,
                TextAnimation::NoAnimation,
            ],
        }
    }
}
