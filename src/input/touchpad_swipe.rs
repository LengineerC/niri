use niri_config::gestures::{Gestures, SwipeAxis, SwipeDirection};
use niri_config::Action;

/// Snapshot the mapping at begin so a config reload cannot change a gesture halfway through.
pub struct TouchpadSwipe {
    pub device_id: String,
    config: Gestures,
    fingers: u32,
    cumulative: (f64, f64),
    pub action: SwipeAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwipeAction {
    Pending,
    ViewScroll,
    WorkspaceSwitch(SwipeAxis),
    Overview,
    Binding(Action),
    Ignored,
    Forwarded,
}

impl TouchpadSwipe {
    pub fn new(device_id: String, fingers: u32, config: Gestures, blocked: bool) -> Self {
        let has_bindings = config
            .swipes
            .iter()
            .any(|b| u32::from(b.fingers) == fingers);
        let reserved = has_bindings
            || fingers == u32::from(config.view_scroll_fingers)
            || fingers == u32::from(config.workspace_switch_fingers)
            || fingers == u32::from(config.overview_fingers);
        let action = if blocked {
            SwipeAction::Ignored
        } else if !reserved {
            SwipeAction::Forwarded
        } else if !has_bindings
            && fingers == u32::from(config.overview_fingers)
            && fingers != u32::from(config.view_scroll_fingers)
            && fingers != u32::from(config.workspace_switch_fingers)
        {
            // Preserve the immediate, finger-following default overview gesture.
            SwipeAction::Overview
        } else {
            SwipeAction::Pending
        };
        Self {
            device_id,
            config,
            fingers,
            cumulative: (0., 0.),
            action,
        }
    }

    /// Directions follow physical finger motion, independently of natural scrolling.
    pub fn recognize(&mut self, dx: f64, dy: f64) -> bool {
        if self.action != SwipeAction::Pending {
            return false;
        }
        self.cumulative.0 += dx;
        self.cumulative.1 += dy;
        let (x, y) = self.cumulative;
        if x.hypot(y) < self.config.swipe_threshold {
            return false;
        }
        let direction = if x.abs() > y.abs() {
            if x < 0. {
                SwipeDirection::Left
            } else {
                SwipeDirection::Right
            }
        } else if y < 0. {
            SwipeDirection::Up
        } else {
            SwipeDirection::Down
        };
        self.action = if let Some(binding) = self
            .config
            .swipes
            .iter()
            .find(|b| u32::from(b.fingers) == self.fingers && b.direction == direction)
        {
            SwipeAction::Binding(binding.action.clone())
        } else if self.fingers == u32::from(self.config.workspace_switch_fingers)
            && direction.axis() == self.config.workspace_switch_axis
        {
            SwipeAction::WorkspaceSwitch(direction.axis())
        } else if self.fingers == u32::from(self.config.view_scroll_fingers)
            && direction.axis() == SwipeAxis::Horizontal
        {
            SwipeAction::ViewScroll
        } else if self.fingers == u32::from(self.config.overview_fingers)
            && direction.axis() == SwipeAxis::Vertical
        {
            SwipeAction::Overview
        } else {
            SwipeAction::Ignored
        };
        true
    }
}

pub fn configure_three_finger_drag(device: &input::Device, enabled: bool) {
    #[cfg(have_libinput_3fg_drag)]
    {
        use input::AsRaw;
        // The input crate gates its safe wrapper behind a Cargo feature requiring newer
        // libinput everywhere. Probe at build time instead, as for the plugin system.
        unsafe extern "C" {
            fn libinput_device_config_3fg_drag_set_enabled(
                device: *mut input::ffi::libinput_device,
                state: std::ffi::c_uint,
            ) -> std::ffi::c_uint;
        }
        // LIBINPUT_CONFIG_3FG_DRAG_DISABLED = 0, ENABLED_3FG = 1.
        // SAFETY: device owns a live libinput_device; the symbol is available since 1.28.
        let status = unsafe {
            libinput_device_config_3fg_drag_set_enabled(
                device.as_raw() as *mut _,
                u32::from(enabled),
            )
        };
        if enabled && status != 0 {
            tracing::warn!(status, "touchpad does not support three-finger drag");
        }
    }
    #[cfg(not(have_libinput_3fg_drag))]
    {
        let _ = device;
        if enabled {
            tracing::warn!("three-finger drag requires a build with libinput 1.28 or newer");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_preserve_three_finger_navigation_and_four_finger_overview() {
        let mut horizontal = TouchpadSwipe::new("a".into(), 3, Gestures::default(), false);
        assert!(!horizontal.recognize(4., 0.));
        assert_eq!(horizontal.action, SwipeAction::Pending);
        assert!(horizontal.recognize(12., 0.));
        assert_eq!(horizontal.action, SwipeAction::ViewScroll);
        assert!(!horizontal.recognize(0., 100.));
        assert_eq!(horizontal.action, SwipeAction::ViewScroll);
        let mut vertical = TouchpadSwipe::new("a".into(), 3, Gestures::default(), false);
        assert!(vertical.recognize(0., -16.));
        assert_eq!(
            vertical.action,
            SwipeAction::WorkspaceSwitch(SwipeAxis::Vertical)
        );
        assert_eq!(
            TouchpadSwipe::new("a".into(), 4, Gestures::default(), false).action,
            SwipeAction::Overview
        );
    }

    #[test]
    fn four_finger_horizontal_workspaces_and_vertical_bindings_coexist() {
        let config = niri_config::Config::parse_mem(
            r#"gestures {
            workspace-switch-fingers 4
            workspace-switch-axis "horizontal"
            view-scroll-fingers 0
            swipe fingers=4 direction="up" { open-grid-overview; }
        }"#,
        )
        .unwrap();
        for (dx, dy, expected) in [
            (
                -20.,
                0.,
                SwipeAction::WorkspaceSwitch(SwipeAxis::Horizontal),
            ),
            (20., 0., SwipeAction::WorkspaceSwitch(SwipeAxis::Horizontal)),
            (0., -20., SwipeAction::Binding(Action::OpenGridOverview)),
            (0., 20., SwipeAction::Overview),
        ] {
            let mut swipe = TouchpadSwipe::new("a".into(), 4, config.gestures.clone(), false);
            assert!(swipe.recognize(dx, dy));
            assert_eq!(swipe.action, expected);
        }
    }

    #[test]
    fn disabled_and_blocked_gestures() {
        let config = Gestures {
            workspace_switch_fingers: 0,
            view_scroll_fingers: 0,
            overview_fingers: 0,
            ..Default::default()
        };
        assert_eq!(
            TouchpadSwipe::new("a".into(), 4, config, false).action,
            SwipeAction::Forwarded
        );
        let mut blocked = TouchpadSwipe::new("a".into(), 3, Gestures::default(), true);
        assert!(!blocked.recognize(100., 0.));
        assert_eq!(blocked.action, SwipeAction::Ignored);
    }
}
