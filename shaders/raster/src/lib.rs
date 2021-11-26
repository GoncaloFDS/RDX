#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::glam::{uvec2, vec2, vec3, vec4, Mat4, UVec3, Vec2, Vec3, Vec4};

pub struct UBO {
    pub view_model: Mat4,
    pub projection: Mat4,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position)] out_pos: &mut Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO,
    position: Vec3,
) {
    *out_pos = ubo.projection * ubo.view_model * position.extend(1.0);
}

#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4, #[spirv(uniform, descriptor_set = 0, binding = 0)] ubo: &UBO) {
    *output = vec4(0.2, 0.4, 0.2, 1.0);
}
