#[derive(Debug, Clone)]
pub enum TextAnimation {
    NoAnimation,
    SlideAnimation,
    BlinkingAnimation,
}

#[derive(Clone, Copy)]
pub struct SlideAnimation {}

#[derive(Clone, Copy)]
pub struct BlinkingAnimation {}

#[derive(Clone, Copy)]
pub struct NoAnimation {}
