### Overview

<sup>Since: 25.02</sup>

The `gestures` config section contains gesture settings.
For an overview of all niri gestures, see the [Gestures](./Gestures.md) wiki page.

Here's a quick glance at the available settings along with their default values.

```kdl
gestures {
    workspace-switch-fingers 4
    workspace-switch-axis "vertical"
    view-scroll-fingers 0
    overview-fingers 0
    swipe-threshold 8
    pinch-threshold 1
    pinch-sensitivity 2.5
    view-transition-max-speed 4
    swipe fingers=4 direction="left" { focus-column-right; }
    swipe fingers=4 direction="right" { focus-column-left; }
    pinch fingers=4 direction="in" { open-grid-overview; }
    pinch fingers=4 direction="out" { open-overview; }

    dnd-edge-view-scroll {
        trigger-width 30
        delay-ms 100
        max-speed 1500
    }

    dnd-edge-workspace-switch {
        trigger-height 50
        delay-ms 100
        max-speed 1500
    }

    hot-corners {
        // off
        top-left
        // top-right
        bottom-left { grid-overview; }
        // bottom-right
    }
}
```

### Touchpad swipes

The built-in swipes are configurable and retain their finger-following animations:

- `workspace-switch-fingers`: finger count for switching workspaces (default `4`).
- `workspace-switch-axis`: `"vertical"` (default) or `"horizontal"`. This changes the input axis, not niri's vertical workspace arrangement.
- `view-scroll-fingers`: finger count for scrolling the tiling view horizontally (default `0`, disabled).
- `overview-fingers`: finger count for opening/closing overview vertically (default `0`, disabled).
- Set a finger count to `0` to disable that built-in gesture. Use `3`, `4`, or `5` for touchpads capable of reporting those gestures.
- `swipe-threshold`: accumulated movement needed to choose a direction, in libinput's unaccelerated gesture units (default `8`, range `1`–`1000`). Smaller values recognize swipes sooner. Movement accumulated before recognition is included in the first interactive scrolling update.

Use `swipe fingers=4 direction="up" { open-overview; }` to bind a direction to any action accepted in `binds`.
Each binding takes exactly one action. Supported counts are `3`–`5`; directions are `left`, `right`, `up`, and `down`.
Directions refer to physical finger movement and do not invert with `natural-scroll`.
Near-diagonal startup motion is left undecided until the dominant axis reaches the threshold and leads the other axis by at least half the threshold. Short back-and-forth movement must also have net displacement of at least 65% of its path length. Clear motion is recognized without waiting for another event; the decision depends on displacement rather than packet count. Once selected, the direction stays locked until release.
The built-in scrolling gestures continue to honor natural scrolling as before.

Bindings take precedence over built-in gestures in that direction. Otherwise, workspace switching takes precedence over view scrolling, then overview.
An action is selected when the threshold is crossed and runs once when the fingers are released; cancellation or a short swipe does not run an action.
Exception: open/close/toggle actions for grid overview and overview update the transition continuously while the fingers move. Reversing the gesture reverses the transition; release settles toward the nearest endpoint, and cancellation returns toward the starting endpoint. A full swipe transition spans 200 unaccelerated gesture units.
Changing the config during a gesture takes effect on the next gesture.
Later bindings with the same finger count and direction replace earlier bindings, including across included files.

When any binding or built-in gesture uses a finger count, niri owns the complete gesture sequence for that count, including unmapped directions.
Unused finger counts are forwarded to applications. Compositor swipes are suppressed while locked, in the screenshot/MRU/portal picker UI, or when starting during a pointer grab.
Ordinary two-finger scrolling is unchanged. Holding Mod while moving two fingers resizes the window under the pointer using the nearest corner, so both width and height can change even when the pointer starts near the middle of an edge or window. Mouse right-button resizing keeps its original edge selection. This takes precedence over touchpad bindings and the wheel-binding fallback; actual mouse wheel bindings remain unchanged. Release the fingers or Mod to finish resizing. Natural scrolling does not reverse resize direction. libinput must supply both scroll axes; niri cannot recover motion already suppressed by the device's scroll-axis locking.

### Touchpad pinches

With the default paired view-opening bindings, the opposite pinch exits the open view first: spread four fingers to leave grid overview, or pinch inward to leave overview. Exit transitions follow the fingers; cancellation keeps the original view open. This uses the opposite-direction binding for the same finger count, so swapping the two opening bindings also swaps the exit directions. Explicit non-opening actions (including close/toggle actions) retain their configured meaning.

Use `pinch fingers=4 direction="in" { open-grid-overview; }` or `direction="out"` for spreading the fingers. Finger counts `3`–`5` and exactly one action are supported. The defaults are four-finger pinch-in for grid overview and pinch-out for overview. A physical left swipe focuses the window/column on the right, and a right swipe focuses the one on the left, following content-dragging direction. These focus actions also work in the floating layout. Explicit bindings still override these defaults.

`pinch-threshold` controls recognition in percent of logarithmic scale (default `1`, range `1`–`50`); the default is approximately a 1% scale change. `pinch-sensitivity` scales view-transition progress (default `2.5`, range `1`–`10`); larger values need less movement. With the default sensitivity, approximately 10% scale change is enough to settle toward the new view state on release, and a factor of 0.82 inward or 1.22 outward spans a full transition. These settings are separate from `input.touchpad.pinch-sensitivity`, which affects client application zoom only.

Small pinches require continued scale change in the same direction (an additional quarter of the effective threshold); repeated stationary packets cannot confirm a pinch. Scale change reaching twice the effective threshold is recognized immediately. Centroid translation smoothly raises the effective logarithmic scale threshold by 0.15 percentage points per pixel, capped at 3%, without lowering the configured threshold. This filters placement drift while allowing off-center pinches. These checks operate on gestures reported by libinput, not raw finger contacts.

`view-transition-max-speed` caps the displayed progress of gesture-driven grid/overview transitions (default `4`, range `1`–`20`, in full transitions per second). At `4`, a full-range transition takes at least 250 ms; use `3` for at least 333 ms. Input targets remain responsive, while displayed progress catches up at the capped rate, including after release or cancellation. Release decisions use the input target, not the lagging display. This applies to configured swipe/pinch view actions and the built-in overview gesture; keyboard-triggered animations retain their existing animation settings.

Other actions run once on release. Direction selection stays fixed until release, so reversing a pinch retreats the same transition. Unbound finger counts, including normal two-finger application zoom, are forwarded to clients. The touchpad must report the requested finger count and pinch gestures through libinput.

Pinch bindings merge by finger count and direction, just like swipe bindings. Existing explicit swipe bindings still override the new defaults: remove old four-finger up/down bindings to use vertical workspace switching.

For a Mac-style setup:

```kdl
input {
    touchpad {
        three-finger-drag true
        natural-scroll
    }
}
gestures {
    workspace-switch-fingers 4
    workspace-switch-axis "vertical"
    view-scroll-fingers 0
    overview-fingers 0
    swipe-threshold 8
    pinch-threshold 1
    pinch-sensitivity 2.5
    swipe fingers=4 direction="left" { focus-column-right; }
    swipe fingers=4 direction="right" { focus-column-left; }
    pinch fingers=4 direction="in" { open-grid-overview; }
    pinch fingers=4 direction="out" { open-overview; }
}
```

This provides three-finger dragging, interactive four-finger vertical workspace switching, horizontal window focus, and interactive pinch transitions.
Grid overview is a workspace-wide window picker, not macOS's application-only App Exposé.
To use vertical swiping for overview instead, set `workspace-switch-fingers 0` and `overview-fingers 4`, and remove any explicit up/down swipe bindings.
The complete example is also available in `resources/mac-touchpad.kdl`.

### `dnd-edge-view-scroll`

Scroll the tiling view when moving the mouse cursor against a monitor edge during drag-and-drop (DnD).
Also works on a touchscreen.

This will work for regular drag-and-drop (e.g. dragging a file from a file manager), and for window interactive move when targeting the tiling layout.

The options are:

- `trigger-width`: size of the area near the monitor edge that will trigger the scrolling, in logical pixels.
- `delay-ms`: delay in milliseconds before the scrolling starts.
Avoids unwanted scrolling when dragging things across monitors.
- `max-speed`: maximum scrolling speed in logical pixels per second.
The scrolling speed increases linearly as you move your mouse cursor from `trigger-width` to the very edge of the monitor.

```kdl
gestures {
    // Increase the trigger area and maximum speed.
    dnd-edge-view-scroll {
        trigger-width 100
        max-speed 3000
    }
}
```

### `dnd-edge-workspace-switch`

<sup>Since: 25.05</sup>

Scroll the workspaces up/down when moving the mouse cursor against a monitor edge during drag-and-drop (DnD) while in the overview.
Also works on a touchscreen.

The options are:

- `trigger-height`: size of the area near the monitor edge that will trigger the scrolling, in logical pixels.
- `delay-ms`: delay in milliseconds before the scrolling starts.
Avoids unwanted scrolling when dragging things across monitors.
- `max-speed`: maximum scrolling speed; 1500 corresponds to one screen height per second.
The scrolling speed increases linearly as you move your mouse cursor from `trigger-width` to the very edge of the monitor.

```kdl
gestures {
    // Increase the trigger area and maximum speed.
    dnd-edge-workspace-switch {
        trigger-height 100
        max-speed 3000
    }
}
```

### `hot-corners`

<sup>Since: 25.05</sup>

Put your mouse at the very top-left corner of a monitor to toggle the overview, or at the bottom-left corner to toggle the grid overview.
Also works during drag-and-dropping something.

`off` disables the hot corners.

```kdl
// Disable the hot corners.
gestures {
    hot-corners {
        off
    }
}
```

<sup>Since: 25.11</sup> You can choose specific hot corners by name: `top-left`, `top-right`, `bottom-left`, `bottom-right`.
If no corners are explicitly set, the top-left corner will toggle the overview, and the bottom-left corner will toggle the grid overview.
Writing a corner without an action will make it toggle the overview.
To bind a corner to the grid overview, put `grid-overview` inside it.

```kdl
// Enable the top-right hot corner for the overview, and the bottom-right hot corner for the grid overview.
gestures {
    hot-corners {
        top-right
        bottom-right { grid-overview; }
    }
}
```

You can also customize hot corners per-output [in the output config](./Configuration:-Outputs.md#hot-corners).
