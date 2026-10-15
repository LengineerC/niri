use crate::utils::MergeWith;
use crate::FloatOrInt;

#[cfg(test)]
mod swipe_tests {
    use super::*;
    use crate::{Action, Config};

    #[test]
    fn mac_style_gestures_parse() {
        let config = Config::parse_mem(
            r#"
            input { touchpad { three-finger-drag true; }; }
            gestures {
                workspace-switch-fingers 4
                workspace-switch-axis "horizontal"
                view-scroll-fingers 0
                overview-fingers 0
                swipe-threshold 24
                swipe fingers=4 direction="up" { open-overview; }
                swipe fingers=4 direction="down" { open-grid-overview; }
            }
        "#,
        )
        .unwrap();
        assert!(config.input.touchpad.three_finger_drag);
        assert_eq!(config.gestures.workspace_switch_fingers, 4);
        assert_eq!(config.gestures.workspace_switch_axis, SwipeAxis::Horizontal);
        assert!(config
            .gestures
            .swipes
            .iter()
            .any(|b| b.direction == SwipeDirection::Up && b.action == Action::OpenOverview));
        assert!(config
            .gestures
            .swipes
            .iter()
            .any(|b| b.direction == SwipeDirection::Down && b.action == Action::OpenGridOverview));
    }

    #[test]
    fn swipe_bindings_merge_by_fingers_and_direction() {
        let parts: Vec<GesturesPart> = knuffel::parse(
            "gestures.kdl",
            r#"
            gestures { swipe fingers=4 direction="up" { open-overview; }; }
            gestures {
                swipe fingers=4 direction="up" { open-grid-overview; }
                swipe fingers=4 direction="left" { focus-workspace-down; }
            }
        "#,
        )
        .unwrap();
        let mut gestures = Gestures::default();
        for part in parts {
            gestures.merge_with(&part);
        }
        assert_eq!(gestures.swipes.len(), 3);
        assert!(gestures
            .swipes
            .iter()
            .any(|b| b.direction == SwipeDirection::Up && b.action == Action::OpenGridOverview));
        assert_eq!(gestures.view_scroll_fingers, 0);
    }

    #[test]
    fn rejects_invalid_swipe_bindings() {
        for body in [
            "swipe fingers=2 direction=\"up\" { open-overview; }",
            "swipe fingers=6 direction=\"up\" { open-overview; }",
            "swipe fingers=4 direction=\"diagonal\" { open-overview; }",
            "swipe fingers=4 direction=\"up\" {}",
            "swipe fingers=4 direction=\"up\" { open-overview; close-overview; }",
            "swipe-threshold 0",
        ] {
            assert!(
                Config::parse_mem(&format!("gestures {{ {body}; }}")).is_err(),
                "{body}"
            );
        }
    }

    #[test]
    fn pinch_bindings_parse_and_override_defaults() {
        let config = Config::parse_mem(
            r#"gestures {
            pinch fingers=4 direction="in" { close-grid-overview; }
            pinch fingers=5 direction="out" { open-overview; }
        }"#,
        )
        .unwrap();
        assert_eq!(config.gestures.pinches.len(), 3);
        assert!(config.gestures.pinches.iter().any(|b| b.fingers == 4
            && b.direction == PinchDirection::In
            && b.action == Action::CloseGridOverview));
        for body in [
            "pinch fingers=2 direction=\"in\" { open-overview; }",
            "pinch fingers=6 direction=\"out\" { open-overview; }",
            "pinch fingers=4 direction=\"up\" { open-overview; }",
            "pinch fingers=4 direction=\"in\" {}",
            "pinch fingers=4 direction=\"out\" { open-overview; close-overview; }",
        ] {
            assert!(
                Config::parse_mem(&format!("gestures {{ {body}; }}")).is_err(),
                "{body}"
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gestures {
    pub dnd_edge_view_scroll: DndEdgeViewScroll,
    pub dnd_edge_workspace_switch: DndEdgeWorkspaceSwitch,
    pub hot_corners: HotCorners,
    pub workspace_switch_fingers: u8,
    pub workspace_switch_axis: SwipeAxis,
    pub view_scroll_fingers: u8,
    pub overview_fingers: u8,
    pub swipe_threshold: f64,
    pub swipes: Vec<SwipeBinding>,
    pub pinches: Vec<PinchBinding>,
}

impl Default for Gestures {
    fn default() -> Self {
        Self {
            dnd_edge_view_scroll: Default::default(),
            dnd_edge_workspace_switch: Default::default(),
            hot_corners: Default::default(),
            workspace_switch_fingers: 4,
            workspace_switch_axis: SwipeAxis::Vertical,
            view_scroll_fingers: 0,
            overview_fingers: 0,
            swipe_threshold: 16.,
            swipes: vec![
                SwipeBinding {
                    fingers: 4,
                    direction: SwipeDirection::Left,
                    action: crate::Action::FocusColumnLeft,
                },
                SwipeBinding {
                    fingers: 4,
                    direction: SwipeDirection::Right,
                    action: crate::Action::FocusColumnRight,
                },
            ],
            pinches: vec![
                PinchBinding {
                    fingers: 4,
                    direction: PinchDirection::In,
                    action: crate::Action::OpenGridOverview,
                },
                PinchBinding {
                    fingers: 4,
                    direction: PinchDirection::Out,
                    action: crate::Action::OpenOverview,
                },
            ],
        }
    }
}

#[derive(knuffel::DecodeScalar, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SwipeAxis {
    Horizontal,
    #[default]
    Vertical,
}

#[derive(knuffel::DecodeScalar, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(knuffel::DecodeScalar, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinchDirection {
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PinchBinding {
    pub fingers: u8,
    pub direction: PinchDirection,
    pub action: crate::Action,
}

impl<S: knuffel::traits::ErrorSpan> knuffel::Decode<S> for PinchBinding {
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, knuffel::errors::DecodeError<S>> {
        #[derive(knuffel::Decode)]
        struct Raw {
            #[knuffel(property)]
            fingers: u8,
            #[knuffel(property)]
            direction: PinchDirection,
            #[knuffel(children)]
            actions: Vec<crate::Action>,
        }
        let raw = Raw::decode_node(node, ctx)?;
        if !(3..=5).contains(&raw.fingers) || raw.actions.len() != 1 {
            return Err(knuffel::errors::DecodeError::unexpected(
                node,
                "pinch",
                "fingers must be between 3 and 5, and exactly one action is required",
            ));
        }
        Ok(Self {
            fingers: raw.fingers,
            direction: raw.direction,
            action: raw.actions.into_iter().next().unwrap(),
        })
    }
}

impl SwipeDirection {
    pub fn axis(self) -> SwipeAxis {
        match self {
            Self::Left | Self::Right => SwipeAxis::Horizontal,
            Self::Up | Self::Down => SwipeAxis::Vertical,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwipeBinding {
    pub fingers: u8,
    pub direction: SwipeDirection,
    pub action: crate::Action,
}

impl<S: knuffel::traits::ErrorSpan> knuffel::Decode<S> for SwipeBinding {
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, knuffel::errors::DecodeError<S>> {
        #[derive(knuffel::Decode)]
        struct Raw {
            #[knuffel(property)]
            fingers: u8,
            #[knuffel(property)]
            direction: SwipeDirection,
            #[knuffel(children)]
            actions: Vec<crate::Action>,
        }
        let raw = Raw::decode_node(node, ctx)?;
        if !(3..=5).contains(&raw.fingers) {
            return Err(knuffel::errors::DecodeError::unexpected(
                node,
                "swipe",
                "fingers must be between 3 and 5",
            ));
        }
        if raw.actions.len() != 1 {
            return Err(knuffel::errors::DecodeError::unexpected(
                node,
                "swipe",
                "exactly one action is required",
            ));
        }
        Ok(Self {
            fingers: raw.fingers,
            direction: raw.direction,
            action: raw.actions.into_iter().next().unwrap(),
        })
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct GesturesPart {
    #[knuffel(child)]
    pub dnd_edge_view_scroll: Option<DndEdgeViewScrollPart>,
    #[knuffel(child)]
    pub dnd_edge_workspace_switch: Option<DndEdgeWorkspaceSwitchPart>,
    #[knuffel(child)]
    pub hot_corners: Option<HotCorners>,
    #[knuffel(child, unwrap(argument))]
    pub workspace_switch_fingers: Option<u8>,
    #[knuffel(child, unwrap(argument))]
    pub workspace_switch_axis: Option<SwipeAxis>,
    #[knuffel(child, unwrap(argument))]
    pub view_scroll_fingers: Option<u8>,
    #[knuffel(child, unwrap(argument))]
    pub overview_fingers: Option<u8>,
    #[knuffel(child, unwrap(argument))]
    pub swipe_threshold: Option<FloatOrInt<1, 1000>>,
    #[knuffel(children(name = "swipe"))]
    pub swipes: Vec<SwipeBinding>,
    #[knuffel(children(name = "pinch"))]
    pub pinches: Vec<PinchBinding>,
}

impl MergeWith<GesturesPart> for Gestures {
    fn merge_with(&mut self, part: &GesturesPart) {
        merge!(
            (self, part),
            dnd_edge_view_scroll,
            dnd_edge_workspace_switch,
        );
        merge_clone!(
            (self, part),
            hot_corners,
            workspace_switch_fingers,
            workspace_switch_axis,
            view_scroll_fingers,
            overview_fingers
        );
        merge!((self, part), swipe_threshold);
        for binding in &part.swipes {
            self.swipes
                .retain(|b| b.fingers != binding.fingers || b.direction != binding.direction);
            self.swipes.push(binding.clone());
        }
        for binding in &part.pinches {
            self.pinches
                .retain(|b| b.fingers != binding.fingers || b.direction != binding.direction);
            self.pinches.push(binding.clone());
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DndEdgeViewScroll {
    pub trigger_width: f64,
    pub delay_ms: u16,
    pub max_speed: f64,
}

impl Default for DndEdgeViewScroll {
    fn default() -> Self {
        Self {
            trigger_width: 30., // Taken from GTK 4.
            delay_ms: 100,
            max_speed: 1500.,
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct DndEdgeViewScrollPart {
    #[knuffel(child, unwrap(argument))]
    pub trigger_width: Option<FloatOrInt<0, 65535>>,
    #[knuffel(child, unwrap(argument))]
    pub delay_ms: Option<u16>,
    #[knuffel(child, unwrap(argument))]
    pub max_speed: Option<FloatOrInt<0, 1_000_000>>,
}

impl MergeWith<DndEdgeViewScrollPart> for DndEdgeViewScroll {
    fn merge_with(&mut self, part: &DndEdgeViewScrollPart) {
        merge!((self, part), trigger_width, max_speed);
        merge_clone!((self, part), delay_ms);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DndEdgeWorkspaceSwitch {
    pub trigger_height: f64,
    pub delay_ms: u16,
    pub max_speed: f64,
}

impl Default for DndEdgeWorkspaceSwitch {
    fn default() -> Self {
        Self {
            trigger_height: 50.,
            delay_ms: 100,
            max_speed: 1500.,
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct DndEdgeWorkspaceSwitchPart {
    #[knuffel(child, unwrap(argument))]
    pub trigger_height: Option<FloatOrInt<0, 65535>>,
    #[knuffel(child, unwrap(argument))]
    pub delay_ms: Option<u16>,
    #[knuffel(child, unwrap(argument))]
    pub max_speed: Option<FloatOrInt<0, 1_000_000>>,
}

impl MergeWith<DndEdgeWorkspaceSwitchPart> for DndEdgeWorkspaceSwitch {
    fn merge_with(&mut self, part: &DndEdgeWorkspaceSwitchPart) {
        merge!((self, part), trigger_height, max_speed);
        merge_clone!((self, part), delay_ms);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct HotCorners {
    pub off: bool,
    pub top_left: Option<HotCornerAction>,
    pub top_right: Option<HotCornerAction>,
    pub bottom_left: Option<HotCornerAction>,
    pub bottom_right: Option<HotCornerAction>,
}

#[derive(knuffel::DecodeScalar, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotCornerAction {
    Overview,
    GridOverview,
}

impl HotCorners {
    pub fn has_any_corner(self) -> bool {
        self.top_left.is_some()
            || self.top_right.is_some()
            || self.bottom_left.is_some()
            || self.bottom_right.is_some()
    }

    pub fn with_default_corners(self) -> Self {
        if self.off || self.has_any_corner() {
            return self;
        }

        Self {
            top_left: Some(HotCornerAction::Overview),
            bottom_left: Some(HotCornerAction::GridOverview),
            ..self
        }
    }
}

impl<S> knuffel::Decode<S> for HotCorners
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, knuffel::errors::DecodeError<S>> {
        if let Some(type_name) = &node.type_name {
            ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                type_name,
                "type name",
                "no type name expected for this node",
            ));
        }

        for val in node.arguments.iter() {
            ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                &val.literal,
                "argument",
                "no arguments expected for this node",
            ));
        }

        for name in node.properties.keys() {
            ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                name,
                "property",
                "no properties expected for this node",
            ));
        }

        let mut rv = HotCorners::default();

        for child in node.children() {
            let action = match &**child.node_name {
                "off" => {
                    if let Some(type_name) = &child.type_name {
                        ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                            type_name,
                            "type name",
                            "no type name expected for this node",
                        ));
                    }
                    for val in child.arguments.iter() {
                        ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                            &val.literal,
                            "argument",
                            "no arguments expected for this node",
                        ));
                    }
                    for name in child.properties.keys() {
                        ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                            name,
                            "property",
                            "no properties expected for this node",
                        ));
                    }
                    for grandchild in child.children() {
                        ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                            grandchild,
                            "node",
                            "no child nodes expected for this node",
                        ));
                    }
                    rv.off = true;
                    continue;
                }
                "top-left" | "top-right" | "bottom-left" | "bottom-right" => {
                    if let Some(type_name) = &child.type_name {
                        ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                            type_name,
                            "type name",
                            "no type name expected for this node",
                        ));
                    }
                    for name in child.properties.keys() {
                        ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                            name,
                            "property",
                            "no properties expected for this node",
                        ));
                    }
                    let mut args = child.arguments.iter();
                    let action = if let Some(arg) = args.next() {
                        let action = knuffel::traits::DecodeScalar::decode(arg, ctx)?;
                        if let Some(arg) = args.next() {
                            ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                                &arg.literal,
                                "argument",
                                "unexpected argument",
                            ));
                        }
                        for grandchild in child.children() {
                            ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                                grandchild,
                                "node",
                                "unexpected node",
                            ));
                        }
                        action
                    } else if let Some(grandchild) = child.children().next() {
                        let action = match &**grandchild.node_name {
                            "overview" => HotCornerAction::Overview,
                            "grid-overview" => HotCornerAction::GridOverview,
                            _ => {
                                ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                                    grandchild,
                                    "node",
                                    format!(
                                        "unexpected hot corner action `{}`",
                                        grandchild.node_name.escape_default()
                                    ),
                                ));
                                HotCornerAction::Overview
                            }
                        };
                        if let Some(type_name) = &grandchild.type_name {
                            ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                                type_name,
                                "type name",
                                "no type name expected for this node",
                            ));
                        }
                        for val in grandchild.arguments.iter() {
                            ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                                &val.literal,
                                "argument",
                                "no arguments expected for this node",
                            ));
                        }
                        for name in grandchild.properties.keys() {
                            ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                                name,
                                "property",
                                "no properties expected for this node",
                            ));
                        }
                        for child in grandchild.children() {
                            ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                                child,
                                "node",
                                "no child nodes expected for this node",
                            ));
                        }
                        for grandchild in child.children().skip(1) {
                            ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                                grandchild,
                                "node",
                                "unexpected node",
                            ));
                        }
                        action
                    } else {
                        HotCornerAction::Overview
                    };
                    action
                }
                _ => {
                    ctx.emit_error(knuffel::errors::DecodeError::unexpected(
                        child,
                        "node",
                        format!(
                            "unexpected hot corner `{}`",
                            child.node_name.escape_default()
                        ),
                    ));
                    continue;
                }
            };

            match &**child.node_name {
                "top-left" => rv.top_left = Some(action),
                "top-right" => rv.top_right = Some(action),
                "bottom-left" => rv.bottom_left = Some(action),
                "bottom-right" => rv.bottom_right = Some(action),
                _ => unreachable!(),
            }
        }

        Ok(rv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn parse_hot_corners(text: &str) -> HotCorners {
        let part: GesturesPart = knuffel::parse("test.kdl", text)
            .map_err(miette::Report::new)
            .unwrap();
        part.hot_corners.unwrap()
    }

    #[test]
    fn hot_corner_without_action_defaults_to_overview() {
        let hot_corners = parse_hot_corners(
            r#"
            hot-corners {
                top-left
            }
            "#,
        );

        assert_eq!(hot_corners.top_left, Some(HotCornerAction::Overview));
        assert_eq!(hot_corners.bottom_left, None);
    }

    #[test]
    fn hot_corner_can_bind_grid_overview_with_child_action() {
        let hot_corners = parse_hot_corners(
            r#"
            hot-corners {
                bottom-left { grid-overview; }
            }
            "#,
        );

        assert_eq!(hot_corners.top_left, None);
        assert_eq!(hot_corners.bottom_left, Some(HotCornerAction::GridOverview));
    }

    #[test]
    fn hot_corner_can_bind_grid_overview_with_string_argument() {
        let hot_corners = parse_hot_corners(
            r#"
            hot-corners {
                bottom-left "grid-overview"
            }
            "#,
        );

        assert_eq!(hot_corners.top_left, None);
        assert_eq!(hot_corners.bottom_left, Some(HotCornerAction::GridOverview));
    }

    #[test]
    fn hot_corner_defaults_to_top_left_and_bottom_left() {
        let hot_corners = HotCorners::default().with_default_corners();

        assert_eq!(hot_corners.top_left, Some(HotCornerAction::Overview));
        assert_eq!(hot_corners.top_right, None);
        assert_eq!(hot_corners.bottom_left, Some(HotCornerAction::GridOverview));
        assert_eq!(hot_corners.bottom_right, None);
    }

    #[test]
    fn explicit_hot_corner_disables_implicit_defaults() {
        let hot_corners = parse_hot_corners(
            r#"
            hot-corners {
                top-right { overview; }
            }
            "#,
        )
        .with_default_corners();

        assert_eq!(hot_corners.top_left, None);
        assert_eq!(hot_corners.top_right, Some(HotCornerAction::Overview));
        assert_eq!(hot_corners.bottom_left, None);
        assert_eq!(hot_corners.bottom_right, None);
    }
}
