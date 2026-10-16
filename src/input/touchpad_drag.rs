use smithay::backend::input::{
    ButtonState, Event, InputBackend, InputTime, PointerButtonEvent, PointerMotionEvent,
};

pub struct TouchpadDrag {
    pub device_id: String,
    pub pressed: bool,
    pub source: DragSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragSource {
    Swipe,
    Pinch,
}

/// Feed gesture motion through the normal pointer handlers so constraints,
/// client-initiated grabs, Mod+drag and DnD behave exactly like mouse input.
pub struct GesturePointerEvent<'a, E> {
    pub source: &'a E,
    pub delta: (f64, f64),
    pub delta_unaccelerated: (f64, f64),
    pub state: ButtonState,
}

impl<I: InputBackend, E: Event<I>> Event<I> for GesturePointerEvent<'_, E> {
    fn time(&self) -> InputTime {
        self.source.time()
    }
    fn device(&self) -> I::Device {
        self.source.device()
    }
}

impl<I: InputBackend, E: Event<I>> PointerButtonEvent<I> for GesturePointerEvent<'_, E> {
    fn button_code(&self) -> u32 {
        0x110
    } // BTN_LEFT
    fn state(&self) -> ButtonState {
        self.state
    }
}

impl<I: InputBackend, E: Event<I>> PointerMotionEvent<I> for GesturePointerEvent<'_, E> {
    fn delta_x(&self) -> f64 {
        self.delta.0
    }
    fn delta_y(&self) -> f64 {
        self.delta.1
    }
    fn delta_x_unaccel(&self) -> f64 {
        self.delta_unaccelerated.0
    }
    fn delta_y_unaccel(&self) -> f64 {
        self.delta_unaccelerated.1
    }
}
