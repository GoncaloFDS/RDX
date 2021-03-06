#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::glam::{uvec2, vec2, vec3, vec4, Mat4, UVec3, Vec2, Vec3, Vec4};

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_id: i32,
    #[spirv(position)] out_pos: &mut Vec4,
    v_color: &mut Vec3,
) {
    let positions: [Vec2; 3] = [vec2(0.0, -0.5), vec2(0.5, 0.5), vec2(-0.5, 0.5)];
    let colors: [Vec3; 3] = [
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
    ];
    *out_pos = positions[vert_id as usize].extend(0.0).extend(1.0);
    *v_color = colors[vert_id as usize];
}

#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4, v_color: Vec3) {
    *output = v_color.extend(1.0);
}
