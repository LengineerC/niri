use std::collections::HashMap;
use std::rc::Rc;

use anyhow::Context as _;
use glam::{Mat3, Vec2};
use smithay::backend::renderer::element::{Element as _, Kind, RenderElement};
use smithay::backend::renderer::gles::{GlesRenderer, Uniform};
use smithay::backend::renderer::Texture;
use smithay::utils::{Logical, Point, Rectangle, Scale, Size};

use crate::render_helpers::offscreen::{OffscreenBuffer, OffscreenData};
use crate::render_helpers::shader_element::ShaderRenderElement;
use crate::render_helpers::shaders::{mat3_uniform, ProgramType, Shaders};

#[derive(Debug, Clone, Copy)]
pub struct MovementShaderParams {
    pub move_from: Point<f64, Logical>,
    pub move_offset: Point<f64, Logical>,
    pub progress: f64,
    pub clamped_progress: f64,
}

impl MovementShaderParams {
    pub fn combined_with(self, other: Self) -> Self {
        Self {
            move_from: self.move_from + other.move_from,
            move_offset: self.move_offset + other.move_offset,
            progress: self.progress.min(other.progress),
            clamped_progress: self.clamped_progress.min(other.clamped_progress),
        }
    }
}

#[derive(Debug)]
pub struct MovementShader {
    random_seed: f32,
    buffer: OffscreenBuffer,
}

impl Default for MovementShader {
    fn default() -> Self {
        Self {
            random_seed: fastrand::f32(),
            buffer: OffscreenBuffer::default(),
        }
    }
}

impl MovementShader {
    pub fn restart(&mut self) {
        self.random_seed = fastrand::f32();
    }

    pub fn has_shader(renderer: &mut GlesRenderer) -> bool {
        Shaders::get(renderer)
            .program(ProgramType::Movement)
            .is_some()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        renderer: &mut GlesRenderer,
        elements: &[impl RenderElement<GlesRenderer>],
        geo_size: Size<f64, Logical>,
        location: Point<f64, Logical>,
        scale: Scale<f64>,
        alpha: f32,
        params: MovementShaderParams,
    ) -> anyhow::Result<(ShaderRenderElement, OffscreenData)> {
        let (elem, _sync_point, mut data) = self
            .buffer
            .render(renderer, scale, elements)
            .context("error rendering to offscreen buffer")?;

        // OffscreenBuffer renders with Transform::Normal and the scale that we passed, so we can
        // assume that below.
        let texture_offset = elem.offset();
        let texture = elem.texture();
        let texture_size = elem.logical_size();

        let current_area = Rectangle::new(location + texture_offset, texture_size);
        let destination = location - params.move_offset;
        let start = destination + params.move_from;
        let destination_area = Rectangle::new(destination + texture_offset, texture_size);
        let start_area = Rectangle::new(start + texture_offset, texture_size);
        let mut area = current_area.merge(destination_area).merge(start_area);

        // Include the whole movement path, then expand it to leave room for deformation and trail
        // effects outside the window geometry.
        let mut target_size = area.size.upscale(1.5);
        target_size.w = f64::max(area.size.w + 1000., target_size.w);
        target_size.h = f64::max(area.size.h + 1000., target_size.h);
        let diff = (target_size.to_point() - area.size.to_point()).downscale(2.);
        let diff = diff.to_physical_precise_round(scale).to_logical(scale);
        area.loc -= diff;
        area.size += diff.upscale(2.).to_size();

        let area_loc = Vec2::new(area.loc.x as f32, area.loc.y as f32);
        let area_size = Vec2::new(area.size.w as f32, area.size.h as f32);

        let geo_loc = Vec2::new(location.x as f32, location.y as f32);
        let geo_size = Vec2::new(geo_size.w as f32, geo_size.h as f32);

        let input_to_geo = Mat3::from_scale(area_size / geo_size)
            * Mat3::from_translation((area_loc - geo_loc) / area_size);

        let tex_scale = Vec2::new(scale.x as f32, scale.y as f32);
        let tex_loc = Vec2::new(texture_offset.x as f32, texture_offset.y as f32);
        let tex_size = Vec2::new(texture.width() as f32, texture.height() as f32) / tex_scale;

        let geo_to_tex =
            Mat3::from_translation(-tex_loc / tex_size) * Mat3::from_scale(geo_size / tex_size);

        let elem = ShaderRenderElement::new(
            ProgramType::Movement,
            area.size,
            None,
            scale.x as f32,
            alpha,
            Rc::new([
                mat3_uniform("niri_input_to_geo", input_to_geo),
                Uniform::new("niri_geo_size", geo_size.to_array()),
                mat3_uniform("niri_geo_to_tex", geo_to_tex),
                Uniform::new("niri_progress", params.progress as f32),
                Uniform::new("niri_clamped_progress", params.clamped_progress as f32),
                Uniform::new(
                    "niri_move_from",
                    [params.move_from.x as f32, params.move_from.y as f32],
                ),
                Uniform::new(
                    "niri_move_offset",
                    [params.move_offset.x as f32, params.move_offset.y as f32],
                ),
                Uniform::new("niri_random_seed", self.random_seed),
            ]),
            HashMap::from([(String::from("niri_tex"), texture.clone())]),
            Kind::Unspecified,
        )
        .with_location(area.loc);

        // We're drawing the shader, not the offscreen itself.
        data.id = elem.id().clone();

        Ok((elem, data))
    }
}
