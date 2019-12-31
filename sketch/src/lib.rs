#[macro_use]
extern crate log;

mod command_buffer;
mod command_pool;
mod debug;
mod framebuffer;
mod instance;
mod logical_device;
mod physical_device;
mod pipeline;
mod queue_family;
mod renderpass;
mod shader;
mod surface;
mod swapchain;
mod utility;

use command_buffer::CommandBuffer;
use command_pool::CommandPool;
use debug::DebugMessenger;
use framebuffer::FrameBuffer;
use instance::Instance;
use logical_device::LogicalDevice;
use physical_device::PhysicalDevice;
use pipeline::Pipeline;
use queue_family::QueueFamily;
use renderpass::RenderPass;
use surface::Surface;
use swapchain::SwapChain;

use ash::{version::DeviceV1_0, vk, Entry};
use winit::window::Window;

use std::cell::Cell;
use std::cell::RefCell;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
    pub(crate) _entry: Entry,
    pub(crate) instance: Instance,
    pub(crate) surface: Surface,
    pub(crate) _physical_device: PhysicalDevice,
    pub(crate) logical_device: LogicalDevice,
    pub(crate) swapchain: SwapChain,
    pub(crate) render_pass: RenderPass,
    pub(crate) pipeline: Pipeline,
    pub(crate) framebuffer: FrameBuffer,
    pub(crate) command_pool: CommandPool,
    pub(crate) command_buffer: CommandBuffer,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: RefCell<Vec<vk::Fence>>,
    current_frame: Cell<usize>,
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
        let command_pool = CommandPool::new(logical_device.vulkan_object(), &physical_device);
        let command_buffers = CommandBuffer::new(logical_device.vulkan_object(), &command_pool, framebuffer.vulkan_object().len() as u32);

        for i in 0..framebuffer.vulkan_object().len() {
            let begin_info = vk::CommandBufferBeginInfo::default();
            unsafe { logical_device.vulkan_object().begin_command_buffer(command_buffers.vulkan_object()[i], &begin_info).unwrap() };

            let clear_color = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0f32, 0.1f32, 0.2f32, 1.0f32],
                },
            };
            let render_pass_info = vk::RenderPassBeginInfo::builder()
                .render_pass(*render_pass.vulkan_object())
                .framebuffer(framebuffer.vulkan_object()[i])
                .render_area(vk::Rect2D::builder().offset(vk::Offset2D { x: 0, y: 0 }).extent(*swapchain.extent()).build())
                .clear_values(&[clear_color])
                .build();

            unsafe {
                logical_device
                    .vulkan_object()
                    .cmd_begin_render_pass(command_buffers.vulkan_object()[i], &render_pass_info, vk::SubpassContents::INLINE);
                logical_device
                    .vulkan_object()
                    .cmd_bind_pipeline(command_buffers.vulkan_object()[i], vk::PipelineBindPoint::GRAPHICS, *pipeline.vulkan_object());
                logical_device.vulkan_object().cmd_draw(command_buffers.vulkan_object()[i], 3, 1, 0, 0);
                logical_device.vulkan_object().cmd_end_render_pass(command_buffers.vulkan_object()[i]);
                logical_device.vulkan_object().end_command_buffer(command_buffers.vulkan_object()[i]).unwrap();
            };
        }

        let mut image_available_semaphores: Vec<vk::Semaphore> = Vec::new();
        let mut render_finished_semaphores: Vec<vk::Semaphore> = Vec::new();
        let mut in_flight_fences: Vec<vk::Fence> = Vec::new();
        let images_in_flight: RefCell<Vec<vk::Fence>> = RefCell::new(Vec::new());

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED).build();

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                in_flight_fences.push(logical_device.vulkan_object().create_fence(&fence_info, None).unwrap());
                image_available_semaphores.push(logical_device.vulkan_object().create_semaphore(&semaphore_info, None).unwrap());
                render_finished_semaphores.push(logical_device.vulkan_object().create_semaphore(&semaphore_info, None).unwrap());
            }
        }

        images_in_flight.borrow_mut().resize(swapchain.images().len(), vk::Fence::null());

        Renderer {
            _entry: entry,
            instance: instance,
            surface: surface,
            _physical_device: physical_device,
            logical_device: logical_device,
            swapchain: swapchain,
            render_pass: render_pass,
            pipeline: pipeline,
            framebuffer: framebuffer,
            command_pool: command_pool,
            command_buffer: command_buffers,
            image_available_semaphores: image_available_semaphores,
            render_finished_semaphores: render_finished_semaphores,
            in_flight_fences: in_flight_fences,
            images_in_flight: images_in_flight,
            current_frame: Cell::new(0),
        }
    }

    pub fn draw_frame(&self) {
        unsafe {
            self.logical_device
                .vulkan_object()
                .wait_for_fences(&[self.in_flight_fences[self.current_frame.get()]], true, std::u64::MAX)
                .unwrap();
        }

        let (image_index, _) = unsafe {
            self.swapchain
                .get_loader()
                .acquire_next_image(
                    *self.swapchain.vulkan_object(),
                    std::u64::MAX,
                    self.image_available_semaphores[self.current_frame.get()],
                    vk::Fence::null(),
                )
                .unwrap()
        };

        if self.images_in_flight.borrow()[image_index as usize] != vk::Fence::null() {
            unsafe {
                self.logical_device
                    .vulkan_object()
                    .wait_for_fences(&[self.images_in_flight.borrow()[image_index as usize]], true, std::u64::MAX)
                    .unwrap();
            }
        }
        self.images_in_flight.borrow_mut()[image_index as usize] = self.in_flight_fences[self.current_frame.get()];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[self.image_available_semaphores[self.current_frame.get()]])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&[self.command_buffer.vulkan_object()[image_index as usize]])
            .signal_semaphores(&[self.render_finished_semaphores[self.current_frame.get()]])
            .build();

        unsafe {
            self.logical_device.vulkan_object().reset_fences(&[self.in_flight_fences[self.current_frame.get()]]).unwrap();
            self.logical_device
                .vulkan_object()
                .queue_submit(*self.logical_device.graphics_queue(), &[submit_info], self.in_flight_fences[self.current_frame.get()])
                .unwrap();
        }

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[self.render_finished_semaphores[self.current_frame.get()]])
            .swapchains(&[*self.swapchain.vulkan_object()])
            .image_indices(&[image_index])
            .build();

        unsafe {
            self.swapchain.get_loader().queue_present(*self.logical_device.present_queue(), &present_info).unwrap();
        }

        self.current_frame.set((self.current_frame.get() + 1) % MAX_FRAMES_IN_FLIGHT);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { self.logical_device.vulkan_object().device_wait_idle().unwrap() };

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                self.logical_device.vulkan_object().destroy_fence(self.in_flight_fences[i], None);
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
