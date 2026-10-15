use smithay::backend::input::{
    Device, DeviceCapability, Event, GestureBeginEvent, GestureEndEvent, GestureSwipeBeginEvent,
    GestureSwipeEndEvent, GestureSwipeUpdateEvent, InputBackend, InputTime, KeyState,
    KeyboardKeyEvent, Keycode, UnusedEvent,
};
use smithay::output::Output;

use crate::input::backend_ext::NiriInputDevice;

pub struct TestInputBackend;

#[derive(PartialEq, Eq, Hash)]
pub struct TestInputDevice;

#[derive(Default)]
pub struct TestSwipeEvent {
    pub fingers: u32,
    pub dx: f64,
    pub dy: f64,
    pub cancelled: bool,
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
    type PointerAxisEvent = UnusedEvent;
    type PointerButtonEvent = UnusedEvent;
    type PointerMotionEvent = UnusedEvent;
    type PointerMotionAbsoluteEvent = UnusedEvent;

    type GestureSwipeBeginEvent = TestSwipeEvent;
    type GestureSwipeUpdateEvent = TestSwipeEvent;
    type GestureSwipeEndEvent = TestSwipeEvent;
    type GesturePinchBeginEvent = UnusedEvent;
    type GesturePinchUpdateEvent = UnusedEvent;
    type GesturePinchEndEvent = UnusedEvent;
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
