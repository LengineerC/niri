use smithay::backend::input::{
    Device, DeviceCapability, Event, GestureBeginEvent, GestureEndEvent, GesturePinchBeginEvent,
    GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent, GestureSwipeEndEvent,
    GestureSwipeUpdateEvent, InputBackend, InputTime, KeyState, KeyboardKeyEvent, Keycode,
    UnusedEvent,
};
use smithay::output::Output;

use crate::input::backend_ext::NiriInputDevice;

pub struct TestInputBackend;

#[derive(PartialEq, Eq, Hash)]
pub struct TestInputDevice;

pub struct TestButtonEvent(pub smithay::backend::input::ButtonState);

impl Event<TestInputBackend> for TestButtonEvent {
    fn time(&self) -> InputTime {
        InputTime::now()
    }
    fn device(&self) -> TestInputDevice {
        TestInputDevice
    }
}

impl smithay::backend::input::PointerButtonEvent<TestInputBackend> for TestButtonEvent {
    fn button_code(&self) -> u32 {
        0x110
    }
    fn state(&self) -> smithay::backend::input::ButtonState {
        self.0
    }
}

pub struct TestAxisEvent {
    pub source: smithay::backend::input::AxisSource,
    pub horizontal: Option<f64>,
    pub vertical: Option<f64>,
    pub inverted: bool,
}

impl Event<TestInputBackend> for TestAxisEvent {
    fn time(&self) -> InputTime {
        InputTime::now()
    }
    fn device(&self) -> TestInputDevice {
        TestInputDevice
    }
}

impl smithay::backend::input::PointerAxisEvent<TestInputBackend> for TestAxisEvent {
    fn amount(&self, axis: smithay::backend::input::Axis) -> Option<f64> {
        match axis {
            smithay::backend::input::Axis::Horizontal => self.horizontal,
            smithay::backend::input::Axis::Vertical => self.vertical,
        }
    }
    fn amount_v120(&self, axis: smithay::backend::input::Axis) -> Option<f64> {
        self.amount(axis)
    }
    fn source(&self) -> smithay::backend::input::AxisSource {
        self.source
    }
    fn relative_direction(
        &self,
        _: smithay::backend::input::Axis,
    ) -> smithay::backend::input::AxisRelativeDirection {
        if self.inverted {
            smithay::backend::input::AxisRelativeDirection::Inverted
        } else {
            smithay::backend::input::AxisRelativeDirection::Identical
        }
    }
}

#[derive(Default)]
pub struct TestSwipeEvent {
    pub fingers: u32,
    pub dx: f64,
    pub dy: f64,
    pub cancelled: bool,
    pub scale: f64,
}

impl Event<TestInputBackend> for TestSwipeEvent {
    fn time(&self) -> InputTime {
        InputTime::now()
    }
    fn device(&self) -> TestInputDevice {
        TestInputDevice
    }
}

impl GestureBeginEvent<TestInputBackend> for TestSwipeEvent {
    fn fingers(&self) -> u32 {
        self.fingers
    }
}

impl GestureEndEvent<TestInputBackend> for TestSwipeEvent {
    fn cancelled(&self) -> bool {
        self.cancelled
    }
}

impl GestureSwipeBeginEvent<TestInputBackend> for TestSwipeEvent {}
impl GesturePinchBeginEvent<TestInputBackend> for TestSwipeEvent {}
impl GesturePinchEndEvent<TestInputBackend> for TestSwipeEvent {}
impl GesturePinchUpdateEvent<TestInputBackend> for TestSwipeEvent {
    fn delta_x(&self) -> f64 {
        self.dx
    }
    fn delta_y(&self) -> f64 {
        self.dy
    }
    fn scale(&self) -> f64 {
        self.scale
    }
    fn rotation(&self) -> f64 {
        0.
    }
}
impl GestureSwipeEndEvent<TestInputBackend> for TestSwipeEvent {}
impl GestureSwipeUpdateEvent<TestInputBackend> for TestSwipeEvent {
    fn delta_x(&self) -> f64 {
        self.dx
    }
    fn delta_y(&self) -> f64 {
        self.dy
    }
}

pub struct TestKeyboardKeyEvent {
    pub time: InputTime,
    pub code: Keycode,
    pub state: KeyState,
    pub count: u32,
}

impl InputBackend for TestInputBackend {
    type Device = TestInputDevice;

    type KeyboardKeyEvent = TestKeyboardKeyEvent;
    type PointerAxisEvent = TestAxisEvent;
    type PointerButtonEvent = TestButtonEvent;
    type PointerMotionEvent = UnusedEvent;
    type PointerMotionAbsoluteEvent = UnusedEvent;

    type GestureSwipeBeginEvent = TestSwipeEvent;
    type GestureSwipeUpdateEvent = TestSwipeEvent;
    type GestureSwipeEndEvent = TestSwipeEvent;
    type GesturePinchBeginEvent = TestSwipeEvent;
    type GesturePinchUpdateEvent = TestSwipeEvent;
    type GesturePinchEndEvent = TestSwipeEvent;
    type GestureHoldBeginEvent = UnusedEvent;
    type GestureHoldEndEvent = UnusedEvent;

    type TouchDownEvent = UnusedEvent;
    type TouchUpEvent = UnusedEvent;
    type TouchMotionEvent = UnusedEvent;
    type TouchCancelEvent = UnusedEvent;
    type TouchFrameEvent = UnusedEvent;
    type TabletToolAxisEvent = UnusedEvent;
    type TabletToolProximityEvent = UnusedEvent;
    type TabletToolTipEvent = UnusedEvent;
    type TabletToolButtonEvent = UnusedEvent;

    type SwitchToggleEvent = UnusedEvent;

    type SpecialEvent = UnusedEvent;
}

impl Device for TestInputDevice {
    fn id(&self) -> String {
        String::from("test")
    }

    fn name(&self) -> String {
        String::from("test input device")
    }

    fn has_capability(&self, capability: DeviceCapability) -> bool {
        matches!(capability, DeviceCapability::Keyboard)
    }

    fn usb_id(&self) -> Option<(u32, u32)> {
        None
    }

    fn syspath(&self) -> Option<std::path::PathBuf> {
        None
    }
}

impl NiriInputDevice for TestInputDevice {
    fn output(&self, _state: &crate::niri::State) -> Option<Output> {
        None
    }
}

impl Event<TestInputBackend> for TestKeyboardKeyEvent {
    fn time(&self) -> InputTime {
        self.time
    }

    fn device(&self) -> <TestInputBackend as InputBackend>::Device {
        TestInputDevice
    }
}

impl KeyboardKeyEvent<TestInputBackend> for TestKeyboardKeyEvent {
    fn key_code(&self) -> Keycode {
        self.code
    }

    fn state(&self) -> KeyState {
        self.state
    }

    fn count(&self) -> u32 {
        self.count
    }
}
