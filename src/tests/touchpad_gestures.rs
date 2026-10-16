use niri_config::{Action, Config};
use smithay::backend::input::InputEvent;

use super::fixture::Fixture;
use super::test_input_backend::{TestInputBackend, TestInputDevice, TestSwipeEvent};

fn fixture() -> Fixture {
    let mut config = Config::parse_mem(
        r#"gestures {
        swipe fingers=4 direction="up" { open-overview; }
    }"#,
    )
    .unwrap();
    config
        .gestures
        .swipes
        .iter_mut()
        .find(|b| b.direction == niri_config::gestures::SwipeDirection::Up)
        .unwrap()
        .action = Action::TestAction;
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1920, 1080));
    f
}

fn begin(f: &mut Fixture) {
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::GestureSwipeBegin {
            event: TestSwipeEvent {
                fingers: 4,
                ..Default::default()
            },
        });
}

fn update(f: &mut Fixture, dy: f64) {
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::GestureSwipeUpdate {
            event: TestSwipeEvent {
                dy,
                ..Default::default()
            },
        });
}

fn end(f: &mut Fixture, cancelled: bool) {
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::GestureSwipeEnd {
            event: TestSwipeEvent {
                cancelled,
                ..Default::default()
            },
        });
}

fn drag_begin(f: &mut Fixture) {
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::GestureSwipeBegin {
            event: TestSwipeEvent {
                fingers: 3,
                ..Default::default()
            },
        });
}

fn drag_update(f: &mut Fixture, dx: f64, dy: f64) {
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::GestureSwipeUpdate {
            event: TestSwipeEvent {
                dx,
                dy,
                ..Default::default()
            },
        });
}

#[test]
fn three_finger_drag_starts_in_every_direction_and_releases() {
    for (dx, dy) in [
        (4., 0.),
        (-4., 0.),
        (0., 4.),
        (0., -4.),
        (3., 3.),
        (-3., -3.),
    ] {
        let mut f = fixture();
        f.niri()
            .config
            .borrow_mut()
            .input
            .touchpad
            .three_finger_drag = true;
        drag_begin(&mut f);
        assert!(!f.niri().touchpad_drag.as_ref().unwrap().pressed);
        drag_update(&mut f, dx, dy);
        assert!(f.niri().touchpad_drag.as_ref().unwrap().pressed);
        assert!(f.niri().touchpad_swipe.is_none());
        end(&mut f, false);
        assert!(f.niri().touchpad_drag.is_none());
    }
}

#[test]
fn three_finger_drag_moves_pointer_on_both_axes() {
    let mut f = fixture();
    f.niri()
        .config
        .borrow_mut()
        .input
        .touchpad
        .three_finger_drag = true;
    drag_begin(&mut f);
    let pointer = f.niri().seat.get_pointer().unwrap();
    let before = pointer.current_location();
    drag_update(&mut f, 12., 7.);
    let after = pointer.current_location();
    assert_eq!(after - before, (12., 7.).into());
    end(&mut f, true);
    assert!(f.niri().touchpad_drag.is_none());
}

#[test]
fn three_finger_drag_cancels_on_removal_and_finger_count_change() {
    let mut f = fixture();
    f.niri()
        .config
        .borrow_mut()
        .input
        .touchpad
        .three_finger_drag = true;
    drag_begin(&mut f);
    drag_update(&mut f, 5., 0.);
    begin(&mut f);
    assert!(f.niri().touchpad_drag.is_none());
    assert!(f.niri().touchpad_swipe.is_some());
    end(&mut f, true);
    drag_begin(&mut f);
    drag_update(&mut f, 0., 5.);
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::DeviceRemoved {
            device: TestInputDevice,
        });
    assert!(f.niri().touchpad_drag.is_none());
}

#[test]
fn native_three_finger_drag_does_not_synthesize_pointer_input() {
    let mut f = fixture();
    {
        let mut config = f.niri().config.borrow_mut();
        config.input.touchpad.three_finger_drag = true;
        config.input.touchpad.three_finger_drag_mode =
            niri_config::input::ThreeFingerDragMode::Native;
    }
    drag_begin(&mut f);
    drag_update(&mut f, 20., 0.);
    assert!(f.niri().touchpad_drag.is_none());
}

#[test]
fn three_finger_drag_does_not_release_physical_mouse_button() {
    use super::test_input_backend::TestButtonEvent;
    use smithay::backend::input::ButtonState;
    let mut f = fixture();
    f.niri()
        .config
        .borrow_mut()
        .input
        .touchpad
        .three_finger_drag = true;
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::PointerButton {
            event: TestButtonEvent(ButtonState::Pressed),
        });
    drag_begin(&mut f);
    drag_update(&mut f, 20., 0.);
    assert!(f.niri().touchpad_drag.is_none());
    end(&mut f, false);
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::PointerButton {
            event: TestButtonEvent(ButtonState::Released),
        });
    drag_begin(&mut f);
    drag_update(&mut f, 20., 0.);
    assert!(f.niri().touchpad_drag.as_ref().unwrap().pressed);
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::PointerButton {
            event: TestButtonEvent(ButtonState::Pressed),
        });
    assert!(f.niri().touchpad_drag.is_none());
}

#[test]
fn three_finger_drag_modes_parse() {
    use niri_config::input::ThreeFingerDragMode;
    for (name, mode) in [
        ("gesture", ThreeFingerDragMode::Gesture),
        ("native", ThreeFingerDragMode::Native),
    ] {
        let config = Config::parse_mem(&format!(r#"input {{ touchpad {{ three-finger-drag true; three-finger-drag-mode "{name}"; }}; }}"#)).unwrap();
        assert!(config.input.touchpad.three_finger_drag);
        assert_eq!(config.input.touchpad.three_finger_drag_mode, mode);
    }
    assert!(Config::parse_mem(include_str!("../../resources/mac-touchpad.kdl")).is_ok());
}

#[test]
fn swipe_binding_fires_once_on_release() {
    let mut f = fixture();
    begin(&mut f);
    update(&mut f, -20.);
    update(&mut f, -40.);
    assert_eq!(f.niri().test_action_count, 0);
    end(&mut f, false);
    assert_eq!(f.niri().test_action_count, 1);
    end(&mut f, false);
    assert_eq!(f.niri().test_action_count, 1);
}

#[test]
fn cancelled_workspace_swipe_and_pinch_takeover_do_not_switch_workspace() {
    for takeover in [false, true] {
        let config = Config::parse_mem("workspace \"one\"\nworkspace \"two\"").unwrap();
        let mut f = Fixture::with_config(config);
        f.add_output(1, (1920, 1080));
        let original = f.niri().layout.active_workspace().unwrap().id();
        begin(&mut f);
        update(&mut f, 300.);
        if takeover {
            pinch_begin(&mut f, 4);
            pinch_update(&mut f, 1.1);
        } else {
            end(&mut f, true);
        }
        assert_eq!(f.niri().layout.active_workspace().unwrap().id(), original);
    }
}

#[test]
fn cancelled_short_and_orphan_swipes_do_not_fire() {
    let mut f = fixture();
    update(&mut f, -100.);
    end(&mut f, false);
    begin(&mut f);
    update(&mut f, -2.);
    end(&mut f, false);
    begin(&mut f);
    update(&mut f, -100.);
    end(&mut f, true);
    assert_eq!(f.niri().test_action_count, 0);
}

#[test]
fn device_removal_cancels_pending_swipe() {
    let mut f = fixture();
    begin(&mut f);
    update(&mut f, -100.);
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::DeviceRemoved {
            device: TestInputDevice,
        });
    end(&mut f, false);
    assert!(f.niri().touchpad_swipe.is_none());
    assert_eq!(f.niri().test_action_count, 0);
}

#[test]
fn swipe_keeps_mapping_when_configuration_changes() {
    let mut f = fixture();
    begin(&mut f);
    f.niri().config.borrow_mut().gestures.swipes.clear();
    update(&mut f, -100.);
    end(&mut f, false);
    assert_eq!(f.niri().test_action_count, 1);
}

fn pinch_begin(f: &mut Fixture, fingers: u32) {
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::GesturePinchBegin {
            event: TestSwipeEvent {
                fingers,
                ..Default::default()
            },
        });
}

fn pinch_update(f: &mut Fixture, scale: f64) {
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::GesturePinchUpdate {
            event: TestSwipeEvent {
                scale,
                ..Default::default()
            },
        });
}

#[test]
fn three_finger_pinch_translation_drags_in_all_directions() {
    for (dx, dy) in [(8., 0.), (-8., 0.), (0., 8.), (0., -8.), (5., 5.)] {
        let mut f = fixture();
        f.niri()
            .config
            .borrow_mut()
            .input
            .touchpad
            .three_finger_drag = true;
        pinch_begin(&mut f, 3);
        let overview_before = f.niri().layout.is_overview_open();
        pinch_update(&mut f, 1.2);
        assert!(!f.niri().touchpad_drag.as_ref().unwrap().pressed);
        assert_eq!(f.niri().layout.is_overview_open(), overview_before);
        f.niri_state()
            .process_input_event::<TestInputBackend>(InputEvent::GesturePinchUpdate {
                event: TestSwipeEvent {
                    dx,
                    dy,
                    scale: 1.2,
                    ..Default::default()
                },
            });
        assert!(f.niri().touchpad_drag.as_ref().unwrap().pressed);
        assert!(f.niri().touchpad_pinch.is_none());
        // A stray end from the other event family must not release the drag.
        end(&mut f, true);
        assert!(f.niri().touchpad_drag.as_ref().unwrap().pressed);
        pinch_end(&mut f, true);
        assert!(f.niri().touchpad_drag.is_none());
    }
}

#[test]
fn three_finger_pinch_drag_moves_pointer_and_cancels_on_four_fingers() {
    let mut f = fixture();
    f.niri()
        .config
        .borrow_mut()
        .input
        .touchpad
        .three_finger_drag = true;
    pinch_begin(&mut f, 3);
    let pointer = f.niri().seat.get_pointer().unwrap();
    let before = pointer.current_location();
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::GesturePinchUpdate {
            event: TestSwipeEvent {
                dx: 12.,
                dy: 7.,
                scale: 0.9,
                ..Default::default()
            },
        });
    assert_eq!(pointer.current_location() - before, (12., 7.).into());
    pinch_begin(&mut f, 4);
    assert!(f.niri().touchpad_drag.is_none());
    pinch_update(&mut f, 1.1);
    assert!(f.niri().layout.is_overview_open());
}

#[test]
fn native_mode_does_not_convert_three_finger_pinch() {
    let mut f = fixture();
    {
        let mut config = f.niri().config.borrow_mut();
        config.input.touchpad.three_finger_drag = true;
        config.input.touchpad.three_finger_drag_mode =
            niri_config::input::ThreeFingerDragMode::Native;
    }
    pinch_begin(&mut f, 3);
    assert!(f.niri().touchpad_drag.is_none());
    assert!(f.niri().touchpad_pinch.is_some());
}

fn pinch_end(f: &mut Fixture, cancelled: bool) {
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::GesturePinchEnd {
            event: TestSwipeEvent {
                cancelled,
                ..Default::default()
            },
        });
}

#[test]
fn pinch_overview_follows_fingers_before_release_and_cancels() {
    let mut f = fixture();
    pinch_begin(&mut f, 4);
    pinch_update(&mut f, 1.1);
    assert!(f.niri().layout.is_overview_open());
    assert!(matches!(
        f.niri().touchpad_pinch.as_ref().unwrap().action,
        crate::input::touchpad_swipe::SwipeAction::View {
            grid: false,
            opening: true
        }
    ));
    pinch_end(&mut f, true);
    assert!(!f.niri().layout.is_overview_open());
    assert!(f.niri().touchpad_pinch.is_none());
}

#[test]
fn pinch_actions_cancel_and_keep_mapping_on_reload() {
    let mut f = fixture();
    f.niri().config.borrow_mut().gestures.pinches[0].action = Action::TestAction;
    pinch_begin(&mut f, 4);
    pinch_update(&mut f, 0.7);
    pinch_end(&mut f, true);
    assert_eq!(f.niri().test_action_count, 0);
    pinch_begin(&mut f, 4);
    f.niri().config.borrow_mut().gestures.pinches.clear();
    pinch_update(&mut f, 0.7);
    assert_eq!(f.niri().test_action_count, 0);
    pinch_end(&mut f, false);
    assert_eq!(f.niri().test_action_count, 1);
    pinch_end(&mut f, false);
    assert_eq!(f.niri().test_action_count, 1);
}

#[test]
fn opposite_pinch_closes_view_instead_of_opening_other_view() {
    for (grid, enter, leave) in [(true, 0.55, 1.8), (false, 1.8, 0.55)] {
        let mut f = fixture();
        pinch_begin(&mut f, 4);
        pinch_update(&mut f, enter);
        pinch_end(&mut f, false);
        assert_eq!(f.niri().layout.is_grid_overview_open(), grid);
        assert_eq!(f.niri().layout.is_overview_open(), !grid);
        pinch_begin(&mut f, 4);
        pinch_update(&mut f, leave);
        assert_eq!(
            f.niri().touchpad_pinch.as_ref().unwrap().action,
            crate::input::touchpad_swipe::SwipeAction::View {
                grid,
                opening: false
            }
        );
        pinch_end(&mut f, false);
        assert!(!f.niri().layout.is_grid_overview_open());
        assert!(!f.niri().layout.is_overview_open());
    }
}

#[test]
fn modest_pinch_can_open_and_close_overview() {
    let mut f = fixture();
    pinch_begin(&mut f, 4);
    pinch_update(&mut f, 1.12);
    pinch_end(&mut f, false);
    assert!(f.niri().layout.is_overview_open());
    pinch_begin(&mut f, 4);
    pinch_update(&mut f, 0.88);
    pinch_end(&mut f, false);
    assert!(!f.niri().layout.is_overview_open());
}

#[test]
fn cancelling_opposite_pinch_keeps_original_view_open() {
    for (grid, enter, leave) in [(true, 0.55, 1.8), (false, 1.8, 0.55)] {
        let mut f = fixture();
        pinch_begin(&mut f, 4);
        pinch_update(&mut f, enter);
        pinch_end(&mut f, false);
        pinch_begin(&mut f, 4);
        pinch_update(&mut f, leave);
        pinch_end(&mut f, true);
        assert_eq!(f.niri().layout.is_grid_overview_open(), grid);
        assert_eq!(f.niri().layout.is_overview_open(), !grid);
    }
}

#[test]
fn opposite_pinch_keeps_custom_action_binding() {
    let mut f = fixture();
    pinch_begin(&mut f, 4);
    pinch_update(&mut f, 1.8);
    pinch_end(&mut f, false);
    f.niri().config.borrow_mut().gestures.pinches[0].action = Action::TestAction;
    pinch_begin(&mut f, 4);
    pinch_update(&mut f, 0.55);
    pinch_end(&mut f, false);
    assert_eq!(f.niri().test_action_count, 1);
    assert!(f.niri().layout.is_overview_open());
}

#[test]
fn pinch_short_or_removed_does_not_dispatch() {
    let mut f = fixture();
    f.niri().config.borrow_mut().gestures.pinches[0].action = Action::TestAction;
    pinch_begin(&mut f, 4);
    pinch_update(&mut f, 0.995);
    pinch_end(&mut f, false);
    pinch_begin(&mut f, 4);
    pinch_update(&mut f, 0.7);
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::DeviceRemoved {
            device: TestInputDevice,
        });
    pinch_end(&mut f, false);
    assert_eq!(f.niri().test_action_count, 0);
    assert!(f.niri().touchpad_pinch.is_none());
}

#[test]
fn bound_swipe_overview_starts_before_release() {
    let mut f = Fixture::with_config(
        Config::parse_mem(
            r#"gestures {
        swipe fingers=4 direction="up" { open-overview; }
    }"#,
        )
        .unwrap(),
    );
    f.add_output(1, (1920, 1080));
    begin(&mut f);
    update(&mut f, -60.);
    assert!(f.niri().layout.is_overview_open());
    end(&mut f, true);
    assert!(!f.niri().layout.is_overview_open());
}

#[test]
fn mod_two_finger_gesture_does_not_fall_back_to_wheel_binding() {
    use super::test_input_backend::{TestAxisEvent, TestKeyboardKeyEvent};
    use smithay::backend::input::{AxisSource, InputTime, KeyState, Keycode};
    let mut config =
        Config::parse_mem("binds { Mod+WheelScrollDown { focus-workspace-down; }; }").unwrap();
    config.binds.0[0].action = Action::TestAction;
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1920, 1080));
    let state = f.niri_state();
    let mod_key = state.backend.mod_key(&state.niri.config.borrow());
    let code = Keycode::new(match mod_key {
        niri_config::ModKey::Super => 133,
        niri_config::ModKey::Alt => 64,
        _ => unreachable!(),
    });
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::Keyboard {
            event: TestKeyboardKeyEvent {
                time: InputTime::now(),
                code,
                state: KeyState::Pressed,
                count: 1,
            },
        });
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::PointerAxis {
            event: TestAxisEvent {
                source: AxisSource::Finger,
                horizontal: None,
                vertical: Some(120.),
                inverted: false,
            },
        });
    assert_eq!(f.niri().test_action_count, 0);
    f.niri_state()
        .process_input_event::<TestInputBackend>(InputEvent::PointerAxis {
            event: TestAxisEvent {
                source: AxisSource::Wheel,
                horizontal: None,
                vertical: Some(120.),
                inverted: false,
            },
        });
    assert_eq!(f.niri().test_action_count, 1);
}
