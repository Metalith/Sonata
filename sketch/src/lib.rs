#[macro_use]
extern crate log;

mod debug;
mod instance;
mod logical_device;
mod physical_device;
mod queue_family;
mod surface;
mod swapchain;
mod renderpass;
mod pipeline;
mod framebuffer;
mod command_buffer;
mod command_pool;
mod shader;
mod utility;

use debug::DebugMessenger;
use instance::Instance;
use logical_device::LogicalDevice;
use physical_device::PhysicalDevice;
use surface::Surface;
use swapchain::SwapChain;
use pipeline::Pipeline;
use renderpass::RenderPass;
use framebuffer::FrameBuffer;
use queue_family::QueueFamily;
use command_pool::CommandPool;
use command_buffer::CommandBuffer;

use ash::{vk, Entry, version::DeviceV1_0, extensions::khr};
use winit::window::Window;

use std::cell::Cell;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
    pub(crate) entry: Entry,
    pub(crate) instance: Instance,
    pub(crate) surface: Surface,
    pub(crate) physical_device: PhysicalDevice,
    pub(crate) logical_device: LogicalDevice,
    pub(crate) swapchain: SwapChain,
    pub(crate) render_pass: RenderPass,
    pub(crate) pipeline: Pipeline,
    pub(crate) framebuffer: FrameBuffer,
    pub(crate) command_pool: CommandPool,
    pub(crate) command_buffer: CommandBuffer,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    current_frame: Cell<usize>
}

impl Renderer {
    pub fn new(win: &Window) -> Self {
        let entry = Entry::new().unwrap();
        let instance = Instance::new(&entry);
        let surface = Surface::new(win, &entry, instance.vulkan_object());
        let physical_device = PhysicalDevice::new(instance.vulkan_object(), &surface);
        let logical_device = LogicalDevice::new(instance.vulkan_object(), &physical_device);
        let swapchain = SwapChain::new(instance.vulkan_object(), logical_device.vulkan_object(), &physical_device, &surface, [800, 680]);
        let render_pass = RenderPass::new(logical_device.vulkan_object(), &swapchain);
        let pipeline = Pipeline::new(logical_device.vulkan_object(), &swapchain, &render_pass);
        let framebuffer = FrameBuffer::new(logical_device.vulkan_object(), &swapchain, &render_pass);
        let command_pool =  CommandPool::new(logical_device.vulkan_object(), &physical_device);
        let command_buffers = CommandBuffer::new(logical_device.vulkan_object(), &command_pool, framebuffer.vulkan_object().len() as u32);

        for i in 0..framebuffer.vulkan_object().len()
        {
            let begin_info = vk::CommandBufferBeginInfo::default();
            unsafe { logical_device.vulkan_object().begin_command_buffer(command_buffers.vulkan_object()[i], &begin_info).unwrap()};

            let clear_color  = vk::ClearValue { color: vk::ClearColorValue{ float32: [0.0f32, 0.1f32, 0.2f32, 1.0f32] } };
            let render_pass_info = vk::RenderPassBeginInfo::builder()
                .render_pass(*render_pass.vulkan_object())
                .framebuffer(framebuffer.vulkan_object()[i])
                .render_area(vk::Rect2D::builder().offset(vk::Offset2D { x: 0, y: 0}).extent(*swapchain.extent()).build())
                .clear_values(&[clear_color])
                .build();

            unsafe { 
                logical_device.vulkan_object().cmd_begin_render_pass(command_buffers.vulkan_object()[i], &render_pass_info, vk::SubpassContents::INLINE);
                logical_device.vulkan_object().cmd_bind_pipeline(command_buffers.vulkan_object()[i], vk::PipelineBindPoint::GRAPHICS, *pipeline.vulkan_object());
                logical_device.vulkan_object().cmd_draw(command_buffers.vulkan_object()[i], 3, 1, 0 , 0);
                logical_device.vulkan_object().cmd_end_render_pass(command_buffers.vulkan_object()[i]);
                logical_device.vulkan_object().end_command_buffer(command_buffers.vulkan_object()[i]).unwrap();
            };
        }

        let mut image_available_semaphores: Vec<vk::Semaphore> = Vec::new();
        let mut render_finished_semaphores: Vec<vk::Semaphore> = Vec::new();

        for _ in 0..MAX_FRAMES_IN_FLIGHT
        {
            let semaphore_info = vk::SemaphoreCreateInfo::default();

            unsafe {
                image_available_semaphores.push(logical_device.vulkan_object().create_semaphore(&semaphore_info, None).unwrap());
                render_finished_semaphores.push(logical_device.vulkan_object().create_semaphore(&semaphore_info, None).unwrap());
            }
        }


        Renderer {
            entry: entry,
            instance: instance,
            surface: surface,
            physical_device: physical_device,
            logical_device: logical_device,
            swapchain: swapchain,
            render_pass: render_pass,
            pipeline: pipeline,
            framebuffer: framebuffer,
            command_pool: command_pool,
            command_buffer: command_buffers,
            image_available_semaphores: image_available_semaphores,
            render_finished_semaphores: render_finished_semaphores,
            current_frame: Cell::new(0)
        }
    }

    pub fn draw_frame(&self) {
        let (image_index, _) = unsafe { self.swapchain.get_loader().acquire_next_image(*self.swapchain.vulkan_object(), std::u64::MAX, self.image_available_semaphores[0], vk::Fence::null()).unwrap() };

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[self.image_available_semaphores[0]])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&[self.command_buffer.vulkan_object()[image_index as usize]])
            .signal_semaphores(&[self.render_finished_semaphores[0]])
            .build();

        unsafe { self.logical_device.vulkan_object().queue_submit(*self.logical_device.graphics_queue(), &[submit_info], vk::Fence::null()).unwrap() };

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[self.render_finished_semaphores[0]])
            .swapchains(&[*self.swapchain.vulkan_object()])
            .image_indices(&[image_index])
            .build();

        unsafe {
            self.swapchain.get_loader().queue_present(*self.logical_device.present_queue(), &present_info).unwrap();
            self.logical_device.vulkan_object().queue_wait_idle(*self.logical_device.present_queue()).unwrap();
        };
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { self.logical_device.vulkan_object().device_wait_idle().unwrap() };

        for i in 0..MAX_FRAMES_IN_FLIGHT
        {
            unsafe { 
                self.logical_device.vulkan_object().destroy_semaphore(self.image_available_semaphores[i], None);
                self.logical_device.vulkan_object().destroy_semaphore(self.render_finished_semaphores[i], None);
            }
        }

        self.command_pool.cleanup(self);
        self.framebuffer.cleanup(self);
        self.pipeline.cleanup(self);
        self.render_pass.cleanup(self);
        self.swapchain.cleanup(self);
        self.logical_device.cleanup(self);
        self.surface.cleanup(self);
        self.instance.cleanup(self);
    }
}

trait VulkanObject {
    type Object;
    fn vulkan_object(&self) -> &Self::Object;
    fn cleanup(&self, _renderer: &Renderer);
}
