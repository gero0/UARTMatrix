#[derive(Debug, Clone)]
pub enum TextAnimation {
    NoAnimation,
    SlideAnimation(SlideAnimation),
    BlinkingAnimation(BlinkingAnimation),
}

impl TextAnimation {
    pub fn tick(&mut self) {
        match self {
            TextAnimation::SlideAnimation(anim) => anim.tick(),
            TextAnimation::BlinkingAnimation(anim) => anim.tick(),
            _ => {}
        }
    }

    pub fn get(&mut self) -> AnimationState {
        match self {
            TextAnimation::SlideAnimation(anim) => anim.get(),
            TextAnimation::BlinkingAnimation(anim) => anim.get(),
            _ => AnimationState {
                x_offset: 0,
                y_offset: 0,
                visible: true,
            },
        }
    }
}

pub struct AnimationState {
    pub x_offset: i32,
    pub y_offset: i32,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SlideDirection {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct SlideAnimation {
    x_offset: i32,
    //ticks between moving one pixel
    tempo: i32,
    counter: i32,
    //TODO: Better name?
    slide_length: usize,
    direction: SlideDirection,
}

impl SlideAnimation {
    pub fn new(tempo: i32, slide_length: usize, direction: SlideDirection) -> Self {
        SlideAnimation {
            x_offset: 0,
            counter: 0,
            tempo,
            slide_length,
            direction,
        }
    }

    pub fn tick(&mut self) {
        self.counter += 1;
        if self.counter >= self.tempo {
            self.counter = 0;

            match self.direction {
                SlideDirection::Right => {
                    self.x_offset += 1;
                    if self.x_offset > self.slide_length as i32 {
                        self.x_offset = -(2* self.slide_length as i32)
                    }
                }
                SlideDirection::Left => {
                    self.x_offset -= 1;
                    if self.x_offset.abs() > self.slide_length as i32 {
                        self.x_offset = self.slide_length as i32
                    }
                }
            }
        }
    }

    pub fn get(&mut self) -> AnimationState {
        AnimationState {
            x_offset: self.x_offset,
            y_offset: 0,
            visible: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlinkingAnimation {
    visible: bool,
    //ticks between changing state
    tempo: i32,
    counter: i32,
}

impl BlinkingAnimation {
    pub fn new(tempo: i32) -> Self {
        BlinkingAnimation {
            counter: 0,
            tempo,
            visible: true,
        }
    }

    pub fn tick(&mut self) {
        self.counter += 1;
        if self.counter >= self.tempo {
            self.counter = 0;
            self.visible = !self.visible;
        }
    }

    pub fn get(&mut self) -> AnimationState {
        AnimationState {
            visible: self.visible,
            x_offset: 0,
            y_offset: 0,
        }
    }
}
