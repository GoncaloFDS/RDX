#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::glam::{uvec2, vec2, vec3, vec4, Mat4, UVec3, Vec2, Vec3, Vec4};
use spirv_std::ray_tracing::{AccelerationStructure, RayFlags};
use spirv_std::Image;

pub struct UniformBufferObject {
    pub view_model: Mat4,
    pub projection: Mat4,
    pub view_model_inverse: Mat4,
    pub projection_inverse: Mat4,
}

#[spirv(miss)]
pub fn miss(#[spirv(incoming_ray_payload)] out: &mut Vec3) {
    *out = vec3(0.3, 0.6, 0.1)
}

#[spirv(closest_hit)]
pub fn closest_hit(#[spirv(incoming_ray_payload)] out: &mut Vec3, #[spirv(instance_id)] id: u32) {
    *out = vec3(0.9, 0.5, 0.5)
}

#[spirv(ray_generation)]
pub fn raygen(
    #[spirv(launch_id)] launch_id: UVec3,
    #[spirv(launch_size)] launch_size: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] top_level_as: &AccelerationStructure,
    #[spirv(descriptor_set = 0, binding = 1)] accumulation: &Image!(2D, format=rgba32f, sampled=false),
    #[spirv(descriptor_set = 0, binding = 2)] output: &Image!(2D, format=rgba8, sampled=false),
    #[spirv(uniform, descriptor_set = 0, binding = 3)] ubo: &UniformBufferObject,
    #[spirv(ray_payload)] payload: &mut Vec3,
) {
    let pixel_center = vec2(launch_id.x as f32, launch_id.y as f32) + vec2(0.5, 0.5);
    let in_uv = pixel_center / vec2(launch_size.x as f32, launch_size.y as f32);

    let d = in_uv * 2.0 - Vec2::ONE;

    let origin = ubo.view_model_inverse * vec4(0., 0., 0., 1.);
    let target = ubo.projection_inverse * vec4(d.x, d.y, 1., 1.);
    let direction = ubo.view_model_inverse * target.normalize();

    let cull_mask = 0xff;
    let tmin = 0.001;
    let tmax = 1000.0;

    *payload = Vec3::ZERO;

    unsafe {
        top_level_as.trace_ray(
            RayFlags::OPAQUE,
            cull_mask,
            0,
            0,
            0,
            origin.truncate(),
            tmin,
            direction.truncate(),
            tmax,
            payload,
        );
    }

    unsafe {
        output.write(uvec2(launch_id.x, launch_id.y), payload.extend(1.0));
        // output.write(uvec2(launch_id.x, launch_id.y), direction);
    }
}