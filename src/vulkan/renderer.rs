use crate::vulkan::command_buffers::CommandBuffers;
use crate::vulkan::debug_utils::DebugMessenger;
use crate::vulkan::depth_buffer::DepthBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::fence::Fence;
use crate::vulkan::framebuffer::Framebuffer;
use crate::vulkan::graphics_pipeline::GraphicsPipeline;
use crate::vulkan::scene::Scene;
use crate::vulkan::semaphore::Semaphore;
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::uniform_buffer::UniformBuffer;
use erupt::vk;
use std::rc::Rc;
use winit::window::Window;

pub struct Renderer {
    device: Rc<Device>,
    _debug_messenger: DebugMessenger,
    command_buffers: CommandBuffers,
    swapchain: Option<Swapchain>,
    depth_buffer: Option<DepthBuffer>,
    graphics_pipeline: Option<GraphicsPipeline>,
    framebuffers: Vec<Framebuffer>,
    present_semaphores: Vec<Semaphore>,
    render_semaphores: Vec<Semaphore>,
    fences: Vec<Fence>,
    uniform_buffers: Vec<UniformBuffer>,
    current_frame: usize,
}

impl Renderer {
    fn swapchain(&self) -> &Swapchain {
        self.swapchain.as_ref().unwrap()
    }

    fn depth_buffer(&self) -> &DepthBuffer {
        self.depth_buffer.as_ref().unwrap()
    }

    fn graphics_pipeline(&self) -> &GraphicsPipeline {
        self.graphics_pipeline.as_ref().unwrap()
    }

    pub fn new(device: Rc<Device>) -> Self {
        let debug_messenger = DebugMessenger::new(device.clone());

        let mut command_buffers =
            CommandBuffers::new(device.clone(), device.graphics_family_index(), true);

        Renderer {
            device,
            _debug_messenger: debug_messenger,
            command_buffers,
            swapchain: None,
            depth_buffer: None,
            graphics_pipeline: None,
            framebuffers: vec![],
            present_semaphores: vec![],
            render_semaphores: vec![],
            fences: vec![],
            uniform_buffers: vec![],
            current_frame: 0,
        }
    }

    pub fn create_swapchain(&mut self, window: &Window, scene: &Scene) {
        let extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };
        self.command_buffers.allocate(3);

        self.swapchain = Some(Swapchain::new(
            self.device.clone(),
            window,
            vk::PresentModeKHR::MAILBOX_KHR,
        ));

        self.depth_buffer = Some(DepthBuffer::new(
            self.device.clone(),
            &self.command_buffers,
            extent,
        ));

        let swapchain = self.swapchain.as_ref().unwrap();
        for _ in swapchain.images() {
            self.present_semaphores
                .push(Semaphore::new(self.device.clone()));
            self.render_semaphores
                .push(Semaphore::new(self.device.clone()));
            self.fences.push(Fence::new(self.device.clone(), true));
            self.uniform_buffers
                .push(UniformBuffer::new(self.device.clone()));
        }

        let depth_buffer = self.depth_buffer.as_ref().unwrap();
        self.graphics_pipeline = Some(GraphicsPipeline::new(
            self.device.clone(),
            swapchain,
            depth_buffer,
            &self.uniform_buffers,
            scene,
            false,
        ));

        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();

        for swapchain_image_view in swapchain.image_views() {
            self.framebuffers.push(Framebuffer::new(
                self.device.clone(),
                swapchain_image_view,
                graphics_pipeline.render_pass(),
                swapchain,
                depth_buffer,
            ));
        }

        self.command_buffers.allocate(swapchain.images().len() as _);
    }

    pub fn delete_swapchain(&mut self) {}
    pub fn recreate_swapchain(&mut self) {}

    pub fn draw_frame(&mut self) {
        let fence = &self.fences[self.current_frame];
        let render_semaphore = &self.render_semaphores[self.current_frame];
        let present_semaphore = &self.present_semaphores[self.current_frame];

        let timeout = u64::MAX;
        fence.wait(timeout);

        let swapchain = self.swapchain.as_ref().unwrap();
        if let Some(image_index) =
            swapchain.acquire_next_image(timeout, Some(render_semaphore.handle()))
        {
            let command_buffer = self.command_buffers.begin(image_index as usize);
            self.render(command_buffer, image_index);
            self.command_buffers.end(image_index as usize);

            let command_buffers = [command_buffer];
            let wait_semaphores = [render_semaphore.handle()];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [present_semaphore.handle()];
            let submit_info = [vk::SubmitInfoBuilder::new()
                .command_buffers(&command_buffers)
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .signal_semaphores(&signal_semaphores)];

            fence.reset();

            unsafe {
                self.device
                    .queue_submit(
                        self.device.graphics_queue(),
                        &submit_info,
                        Some(fence.handle()),
                    )
                    .unwrap();
            }

            let swapchains = [swapchain.handle()];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHRBuilder::new()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            unsafe {
                self.device
                    .queue_present_khr(self.device.graphics_queue(), &present_info)
                    .unwrap();
            }

            self.current_frame = (self.current_frame + 1) % self.fences.len();
        } else {
            self.recreate_swapchain();
        }
    }

    fn render(&self, command_buffer: vk::CommandBuffer, image_index: u32) {
        let clean_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.5, 0.2, 0.2, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();

        let render_pass_info = vk::RenderPassBeginInfoBuilder::new()
            .render_pass(graphics_pipeline.render_pass().handle())
            .framebuffer(self.framebuffers[image_index as usize].handle())
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent(),
            })
            .clear_values(&clean_values);

        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            {
                self.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline.handle(),
                );

                // bind descriptors sets
                // bind vertex and index buffers
                // draw
            }
            self.device.cmd_end_render_pass(command_buffer);
        }
    }

    pub fn shutdown(&self) {
        unsafe { self.device.device_wait_idle().unwrap() }
    }
}
