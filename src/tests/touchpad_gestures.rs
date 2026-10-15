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
    config.gestures.swipes[0].action = Action::TestAction;
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
