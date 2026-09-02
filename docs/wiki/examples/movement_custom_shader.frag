// Your shader must contain one function (see the bottom of this file).
//
// It should not contain uniform definitions or anything else, as niri provides
// them for you.
//
// All symbols defined by niri have a niri_ prefix, so do not use it for your
// own variables and functions.

// The function that you must define looks like this:
vec4 movement_color(vec3 coords_geo, vec3 size_geo) {
    vec4 color = /* ...compute the color... */;
    return color;
}

// coords_geo contains homogeneous coordinates relative to the window's current
// geometry. The [0, 1] range lies inside the window. Pixels outside the window
// have coordinates below 0 or above 1 because the shader area covers the whole
// movement path and has extra room for effects.
//
// size_geo is the window geometry size in logical pixels. Its Z component is 1.
//
// Return premultiplied-alpha color. Niri applies the final window opacity after
// this function returns.

// The window texture.
uniform sampler2D niri_tex;

// Converts geometry coordinates to window texture coordinates. The texture may
// extend beyond the geometry, for example for client-side decoration shadows.
uniform mat3 niri_geo_to_tex;

// Animation progress. It goes from 0 to 1 and may overshoot or oscillate for a
// spring animation. If both axes animate independently, this is the progress of
// the least advanced axis.
uniform float niri_progress;

// Progress clamped to [0, 1], stopping at 1 when the destination is first
// reached.
uniform float niri_clamped_progress;

// Displacement from the final destination to the animation's starting point,
// in logical pixels.
uniform vec2 niri_move_from;

// Displacement from the final destination to the current window position, in
// logical pixels. It approaches vec2(0.0) as the animation completes.
uniform vec2 niri_move_offset;

// Random float in [0, 1), stable for the duration of one movement animation.
uniform float niri_random_seed;

// Example: bend the window perpendicular to its direction of movement. The
// bend follows the remaining distance, including spring overshoot.
vec4 wobble(vec3 coords_geo, vec3 size_geo) {
    vec2 direction = niri_move_from;
    float distance = length(direction);
    if (distance > 0.001)
        direction /= distance;

    vec2 perpendicular = vec2(-direction.y, direction.x);
    float remaining = length(niri_move_offset);
    float strength = min(remaining * 0.08, 32.0);
    float wave = sin(coords_geo.x * 3.14159265) * sin(coords_geo.y * 3.14159265);

    vec2 displacement = perpendicular * strength * wave;
    coords_geo.xy -= displacement / size_geo.xy;

    vec3 coords_tex = niri_geo_to_tex * coords_geo;
    return texture2D(niri_tex, coords_tex.st);
}

// This is the function that you must define.
vec4 movement_color(vec3 coords_geo, vec3 size_geo) {
    return wobble(coords_geo, size_geo);
}
