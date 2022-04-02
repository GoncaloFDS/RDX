use crate::triangle_pass::TrianglePass;
use crate::vulkan::command_pool::CommandPool;
use crate::vulkan::device::Device;
use crate::vulkan::frame::Frame;
use crate::vulkan::image_view::ImageView;
use crate::vulkan::instance::Instance;
use crate::vulkan::semaphore::Semaphore;
use crate::vulkan::subresource_range::SubresourceRange;
use crate::vulkan::surface::Surface;
use crate::vulkan::swapchain::Swapchain;
use erupt::{vk, SmallVec};
use std::slice;
use std::time::Instant;
use winit::window::Window;

pub struct App {
    device: Device,
    instance: Instance,
    surface: Surface,
    epoch: Instant,
    swapchain: Swapchain,
    swapchain_image_views: Vec<ImageView>,
    command_pool: CommandPool,
    frames: Vec<Frame>,
    triangle_pass: TrianglePass,
}

impl App {
    pub fn new(window: &Window) -> Self {
        let instance = Instance::new(window);
        let surface = Surface::new(&instance, window);
        let device = Device::new(&instance, surface);
        let swapchain = Swapchain::new(window, &instance, surface, &device);

        let command_pool = CommandPool::new(&device, device.queue_index(), true);
        let command_buffers = command_pool.allocate(&device, swapchain.frames_in_flight() as u32);
        let frames = command_buffers
            .iter()
            .map(|&command_buffer| {
                let cmd_complete_semaphore = Semaphore::new(&device);
                Frame::new(command_buffer, cmd_complete_semaphore)
            })
            .collect();

        let triangle_pass = TrianglePass::new(&device, swapchain.surface_format());

        App {
            device,
            instance,
            surface,
            epoch: Instant::now(),
            swapchain,
            swapchain_image_views: Vec::new(),
            command_pool,
            frames,
            triangle_pass,
        }
    }

    pub fn resize(&mut self, size: vk::Extent2D) {
        self.swapchain.resize(size);
    }

    pub fn draw(&mut self) {
        let acq = self
            .swapchain
            .acquire(&self.instance, &self.device, u64::MAX)
            .unwrap();

        // Recreate swapchain image views when necessary
        if acq.invalidate_images {
            for image_view in &self.swapchain_image_views {
                image_view.destoy(&self.device);
            }

            let format = self.swapchain.format();
            self.swapchain_image_views = self
                .swapchain
                .images()
                .iter()
                .map(|&swapchain_image| {
                    ImageView::new(
                        &self.device,
                        swapchain_image,
                        format.format,
                        vk::ImageAspectFlags::COLOR,
                    )
                })
                .collect();
        }

        let in_flight = &self.frames[acq.frame_index];
        let swapchain_image = self.swapchain.images()[acq.image_index];
        let swapchain_image_view = self.swapchain_image_views[acq.image_index];

        let extend = self.swapchain.extent();
        let rect = vk::Rect2DBuilder::new().extent(extend);

        in_flight.command_buffer.begin(&self.device);

        in_flight.command_buffer.bind_pipeline(
            &self.device,
            vk::PipelineBindPoint::GRAPHICS,
            self.triangle_pass.pipeline.handle(),
        );

        in_flight
            .command_buffer
            .set_scissor(&self.device, 0, &[rect]);

        let viewports = vk::ViewportBuilder::new()
            .height(extend.height as f32)
            .width(extend.width as f32)
            .max_depth(1.0);
        in_flight
            .command_buffer
            .set_viewport(&self.device, 0, &[viewports]);

        in_flight.command_buffer.image_memory_barrier(
            &self.device,
            swapchain_image,
            SubresourceRange::with_aspect(vk::ImageAspectFlags::COLOR),
            vk::PipelineStageFlags2::NONE,
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );

        let t = (self.epoch.elapsed().as_secs_f32().sin() + 1.0) * 0.5;
        let color_attachment = vk::RenderingAttachmentInfoBuilder::new()
            .image_view(swapchain_image_view.handle())
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, t, 0.0, 1.0],
                },
            })
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .resolve_image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let rendering_info = vk::RenderingInfoBuilder::new()
            .color_attachments(slice::from_ref(&color_attachment))
            .layer_count(1)
            .render_area(vk::Rect2D {
                offset: Default::default(),
                extent: self.swapchain.extent(),
            });
        in_flight
            .command_buffer
            .begin_rendering(&self.device, &rendering_info);

        self.triangle_pass
            .draw(&self.device, in_flight.command_buffer);

        in_flight.command_buffer.end_rendering(&self.device);

        in_flight.command_buffer.image_memory_barrier(
            &self.device,
            swapchain_image,
            SubresourceRange::with_aspect(vk::ImageAspectFlags::COLOR),
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            vk::AccessFlags2::COLOR_ATTACHMENT_READ | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );

        // Submit commands and queue present
        in_flight.command_buffer.end(&self.device);

        self.device.submit(
            &[vk::SemaphoreSubmitInfoBuilder::new()
                .semaphore(acq.ready)
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)],
            &[vk::SemaphoreSubmitInfoBuilder::new()
                .semaphore(in_flight.complete.handle())
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)],
            &[vk::CommandBufferSubmitInfoBuilder::new()
                .command_buffer(in_flight.command_buffer.handle())],
            acq.complete,
        );

        self.swapchain
            .queue_present(
                &self.device,
                self.device.queue(),
                in_flight.complete.handle(),
                acq.image_index,
            )
            .unwrap();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.device.wait_idle();
        for image_view in &self.swapchain_image_views {
            image_view.destoy(&self.device);
        }

        for frame in &self.frames {
            frame.complete.destroy(&self.device);
        }

        self.triangle_pass.destroy(&self.device);
        let command_buffers = self
            .frames
            .iter()
            .map(|frame| frame.command_buffer)
            .collect::<SmallVec<_>>();
        self.command_pool
            .free_command_buffers(&self.device, &command_buffers);
        self.command_pool.destroy(&self.device);
        self.swapchain.destroy(&self.device);
        self.surface.destroy(&self.instance);
        self.device.destroy();
        self.instance.destroy();
    }
}
