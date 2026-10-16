use niri_config::gestures::{Gestures, PinchBinding, PinchDirection, SwipeAxis, SwipeDirection};
use niri_config::Action;

/// Snapshot the mapping at begin so a config reload cannot change a gesture halfway through.
pub struct TouchpadSwipe {
    pub device_id: String,
    config: Gestures,
    fingers: u32,
    pub cumulative: (f64, f64),
    pub direction: Option<SwipeDirection>,
    pub action: SwipeAction,
    path_length: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwipeAction {
    Pending,
    ViewScroll,
    WorkspaceSwitch(SwipeAxis),
    Overview,
    Binding(Action),
    View { grid: bool, opening: bool },
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
            direction: None,
            action,
            path_length: 0.,
        }
    }

    /// Directions follow physical finger motion, independently of natural scrolling.
    pub fn recognize(&mut self, dx: f64, dy: f64) -> bool {
        if !dx.is_finite() || !dy.is_finite() {
            return false;
        }
        self.cumulative.0 += dx;
        self.cumulative.1 += dy;
        self.path_length += dx.hypot(dy);
        if self.action != SwipeAction::Pending {
            return false;
        }
        let (x, y) = self.cumulative;
        // Require distance evidence, not a number of packets: report rate should
        // not determine latency. A diagonal start remains undecided until one
        // axis has a meaningful lead, even on a long, slightly diagonal swipe.
        let major = x.abs().max(y.abs());
        let minor = x.abs().min(y.abs());
        let threshold = self.config.swipe_threshold;
        if major < threshold || major - minor < threshold * 0.5 {
            return false;
        }
        // Short back-and-forth placement jitter is not directional intent.
        if major < threshold * 2. && x.hypot(y) < self.path_length * 0.65 {
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
        self.direction = Some(direction);
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

pub struct TouchpadPinch {
    pub device_id: String,
    pub bindings: Vec<PinchBinding>,
    pub action: SwipeAction,
    pub direction: Option<PinchDirection>,
    threshold: f64,
    pub sensitivity: f64,
    pub translation: (f64, f64),
    candidate_scale: Option<f64>,
}

impl TouchpadPinch {
    /// An opposite view-opening binding closes the currently open view first.
    /// Explicit non-opening bindings (including close/toggle and user actions)
    /// retain their configured meaning.
    pub fn resolve_reverse_view(&mut self, grid_open: bool, overview_open: bool) {
        if !matches!(
            self.action,
            SwipeAction::Binding(Action::OpenOverview | Action::OpenGridOverview)
        ) {
            return;
        }
        let opposite = match self.direction {
            Some(PinchDirection::In) => PinchDirection::Out,
            Some(PinchDirection::Out) => PinchDirection::In,
            None => return,
        };
        let Some(binding) = self.bindings.iter().find(|b| b.direction == opposite) else {
            return;
        };
        match binding.action {
            Action::OpenOverview if overview_open => {
                self.action = SwipeAction::Binding(Action::CloseOverview)
            }
            Action::OpenGridOverview if grid_open => {
                self.action = SwipeAction::Binding(Action::CloseGridOverview)
            }
            _ => (),
        }
    }

    pub fn new(device_id: String, fingers: u32, config: &Gestures, blocked: bool) -> Self {
        let bindings: Vec<_> = config
            .pinches
            .iter()
            .filter(|b| u32::from(b.fingers) == fingers)
            .cloned()
            .collect();
        let action = if blocked {
            SwipeAction::Ignored
        } else if bindings.is_empty() {
            SwipeAction::Forwarded
        } else {
            SwipeAction::Pending
        };
        Self {
            device_id,
            bindings,
            action,
            direction: None,
            threshold: config.pinch_threshold / 100.,
            sensitivity: config.pinch_sensitivity,
            translation: (0., 0.),
            candidate_scale: None,
        }
    }

    pub fn recognize(&mut self, scale: f64) -> bool {
        if self.action != SwipeAction::Pending || !scale.is_finite() || scale <= 0. {
            return false;
        }
        let log_scale = scale.ln();
        // Raise the evidence requirement smoothly when the centroid translates.
        // No discontinuous 8px/6% gate, which penalized legitimate off-center pinches.
        let threshold = self
            .threshold
            .max((self.translation.0.hypot(self.translation.1) * 0.0015).min(0.03));
        if log_scale.abs() < threshold {
            self.candidate_scale = None;
            return false;
        }
        let direction = if scale < 1. {
            PinchDirection::In
        } else {
            PinchDirection::Out
        };
        if log_scale.abs() < threshold * 2. {
            match self.candidate_scale {
                Some(previous) if previous.signum() == log_scale.signum() => {
                    // Repeated stationary packets must not confirm a pinch.
                    if log_scale.abs() - previous.abs() < threshold * 0.25 {
                        return false;
                    }
                }
                _ => {
                    self.candidate_scale = Some(log_scale);
                    return false;
                }
            }
        }
        self.direction = Some(direction);
        self.action = self
            .bindings
            .iter()
            .find(|b| b.direction == direction)
            .map_or(SwipeAction::Ignored, |b| {
                SwipeAction::Binding(b.action.clone())
            });
        true
    }
}

pub struct TouchpadResize {
    pub device_id: String,
    pub window: smithay::desktop::Window,
    pub delta: smithay::utils::Point<f64, smithay::utils::Logical>,
    pub active_axes: [bool; 2],
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
    fn swipe_direction_is_symmetric_and_independent_of_packet_count() {
        for (dx, dy) in [(12., 0.), (-12., 0.), (0., 12.), (0., -12.), (20., 15.)] {
            let mut single = TouchpadSwipe::new("a".into(), 4, Gestures::default(), false);
            let mut split = TouchpadSwipe::new("a".into(), 4, Gestures::default(), false);
            assert!(single.recognize(dx, dy));
            for _ in 0..10 {
                split.recognize(dx / 10., dy / 10.);
            }
            assert_eq!(single.direction, split.direction);
            assert_eq!(single.action, split.action);
        }
    }

    #[test]
    fn startup_jitter_waits_for_clear_motion() {
        let mut swipe = TouchpadSwipe::new("a".into(), 4, Gestures::default(), false);
        assert!(!swipe.recognize(6., 0.));
        assert!(!swipe.recognize(-6., 0.));
        assert!(!swipe.recognize(8., 0.));
        assert!(swipe.recognize(8., 0.));
        assert_eq!(swipe.direction, Some(SwipeDirection::Right));
    }

    #[test]
    fn diagonal_start_does_not_lock_wrong_swipe_axis() {
        let mut swipe = TouchpadSwipe::new("a".into(), 4, Gestures::default(), false);
        assert!(!swipe.recognize(7., 6.));
        assert!(swipe.recognize(1., 6.));
        assert_eq!(
            swipe.action,
            SwipeAction::WorkspaceSwitch(SwipeAxis::Vertical)
        );
    }

    #[test]
    fn translating_fingers_do_not_trigger_weak_pinch() {
        let mut pinch = TouchpadPinch::new("a".into(), 4, &Gestures::default(), false);
        pinch.translation = (20., 5.);
        assert!(!pinch.recognize(1.02));
        assert!(!pinch.recognize(1.03));
        assert!(pinch.recognize(1.1));
        assert_eq!(pinch.action, SwipeAction::Binding(Action::OpenOverview));
    }

    #[test]
    fn pinch_sensitivity_and_threshold_are_snapshotted() {
        let mut config = Gestures::default();
        let mut pinch = TouchpadPinch::new("a".into(), 4, &config, false);
        config.pinch_sensitivity = 1.;
        assert_eq!(pinch.sensitivity, 2.5);
        assert!(!pinch.recognize(1.005));
        assert!(!pinch.recognize(1.02));
        assert!(!pinch.recognize(1.02));
        assert!(pinch.recognize(1.03));
        assert_eq!(pinch.action, SwipeAction::Binding(Action::OpenOverview));
        let mut pinch = TouchpadPinch::new("a".into(), 4, &config, false);
        assert!(!pinch.recognize(1. / 1.02));
        assert!(!pinch.recognize(1. / 1.02));
        assert!(pinch.recognize(1. / 1.03));
        assert_eq!(pinch.action, SwipeAction::Binding(Action::OpenGridOverview));
        assert_eq!(pinch.sensitivity, 1.);
    }

    #[test]
    fn defaults_use_four_finger_workspaces_and_window_switching() {
        let mut horizontal = TouchpadSwipe::new("a".into(), 4, Gestures::default(), false);
        assert!(!horizontal.recognize(4., 0.));
        assert_eq!(horizontal.action, SwipeAction::Pending);
        assert!(horizontal.recognize(4., 0.));
        assert_eq!(
            horizontal.action,
            SwipeAction::Binding(Action::FocusColumnLeft)
        );
        assert!(!horizontal.recognize(0., 100.));
        assert_eq!(
            horizontal.action,
            SwipeAction::Binding(Action::FocusColumnLeft)
        );
        let mut vertical = TouchpadSwipe::new("a".into(), 4, Gestures::default(), false);
        assert!(vertical.recognize(0., -16.));
        assert_eq!(
            vertical.action,
            SwipeAction::WorkspaceSwitch(SwipeAxis::Vertical)
        );
        assert_eq!(
            TouchpadSwipe::new("a".into(), 3, Gestures::default(), false).action,
            SwipeAction::Forwarded
        );
        let mut left = TouchpadSwipe::new("a".into(), 4, Gestures::default(), false);
        assert!(left.recognize(-8., 0.));
        assert_eq!(left.action, SwipeAction::Binding(Action::FocusColumnRight));
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
        let mut config = config;
        config
            .gestures
            .swipes
            .retain(|b| b.direction == SwipeDirection::Up);
        config.gestures.overview_fingers = 4;
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
            swipes: vec![],
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
