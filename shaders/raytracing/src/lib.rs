#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use spirv_std::glam::{uvec2, vec2, vec3, vec4, Mat4, UVec3, Vec2, Vec3, Vec4};
use spirv_std::image::SampledImage;
use spirv_std::ray_tracing::{AccelerationStructure, RayFlags};
use spirv_std::{Image, RuntimeArray, Sampler};

pub struct UniformBufferObject {
    pub view_model: Mat4,
    pub projection: Mat4,
    pub view_model_inverse: Mat4,
    pub projection_inverse: Mat4,
}

pub struct Material {
    pub color: Vec3,
}

pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
}

type Textures = RuntimeArray<Image!(2D, type=f32, sampled)>;

struct TextureSampler<'a> {
    textures: &'a Textures,
    sampler: Sampler,
    uv: Vec2,
}

impl<'a> TextureSampler<'a> {
    fn sample(&self, texture_id: u32) -> Vec4 {
        let texture = unsafe { self.textures.index(texture_id as usize) };
        texture.sample_by_lod(self.sampler, self.uv, 0.0)
    }
}

#[spirv(miss)]
pub fn miss(#[spirv(incoming_ray_payload)] out: &mut Vec3) {
    *out = vec3(0.0, 0.2, 0.8)
}

#[spirv(closest_hit)]
pub fn closest_hit(
    #[spirv(incoming_ray_payload)] out: &mut Vec3,
    #[spirv(primitive_id)] id: u32,
    #[spirv(instance_custom_index)] index: u32,
    #[spirv(hit_attribute)] attribs: &mut Vec2,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)] vertices: &[Vertex],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)] indices: &[u32],
    // #[spirv(descriptor_set = 0, binding = 6, storage_buffer)] materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 7, storage_buffer)] offsets: &[(u32, u32)],
    // #[spirv(descriptor_set = 0, binding = 8)] textures: &Textures,
    // #[spirv(descriptor_set = 0, binding = 9)] sampler: &Sampler,
) {
    let id = id as usize;
    let offsets = &offsets[index as usize];
    let (vertex_offset, index_offset) = (offsets.0 as usize, offsets.1 as usize);
    let indices = (
        indices[index_offset + id * 3] as usize,
        indices[index_offset + id * 3 + 1] as usize,
        indices[index_offset + id * 3 + 2] as usize,
    );
    let v0 = &vertices[vertex_offset + indices.0];
    let v1 = &vertices[vertex_offset + indices.1];
    let v2 = &vertices[vertex_offset + indices.2];

    let barycentrics = vec3(1.0 - attribs.x - attribs.y, attribs.x, attribs.y);

    let uv = v0.uv * barycentrics.x + v1.uv * barycentrics.y + v2.uv * barycentrics.z;

    // let texture_sampler = TextureSampler {
    //     textures,
    //     sampler: *sampler,
    //     uv,
    // };
    //
    // let tex = texture_sampler.sample(0);
    //
    // *out = tex.truncate();
    *out = vec3(0.3, 0.4, 0.6);
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
