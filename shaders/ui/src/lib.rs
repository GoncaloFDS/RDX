#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::glam::{uvec2, vec2, vec3, vec4, Mat4, UVec3, Vec2, Vec3, Vec4};
use spirv_std::image::Image;
use spirv_std::Sampler;

pub struct PushConstants {
    pub screen_size: Vec2,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(position)] out_pos: &mut Vec4,
    #[spirv(push_constant)] push_constants: &PushConstants,
    position: Vec2,
    tex_coords: Vec2,
    color: Vec4,
    v_tex_coords: &mut Vec2,
    v_color: &mut Vec4,
) {
    *out_pos = vec4(
        2.0 * position.x / push_constants.screen_size.x - 1.0,
        2.0 * position.y / push_constants.screen_size.y - 1.0,
        0.0,
        1.0,
    );
    *v_color = color;
    *v_tex_coords = tex_coords;
}

#[spirv(fragment)]
pub fn main_fs(
    output: &mut Vec4,
    // #[spirv(descriptor_set = 0, binding = 0)] texture: &Image!(2D, type=f32, sampled),
    // #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    v_tex_coords: Vec2,
    v_color: Vec4,
) {
    // let font: Vec4 = texture.sample(*sampler, v_tex_coords);
    // *output = v_color * font;
    *output = v_color;
}
