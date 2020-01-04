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

use ash::{extensions::khr, version::DeviceV1_0, vk, Device, Entry};
use winit::window::Window;

use std::cell::Cell;
use std::cell::RefCell;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
    _entry: Entry,
    instance: Instance,
    surface: Surface,
    _physical_device: PhysicalDevice,
    logical_device: LogicalDevice,
    swapchain: SwapChain,
    render_pass: RenderPass,
    pipeline: Pipeline,
    frame_buffers: FrameBuffer,
    command_pool: CommandPool,
    command_buffers: CommandBuffer,
    sync_objects: SyncObjects,
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
        let sync_objects = SyncObjects::new(logical_device.vulkan_object(), MAX_FRAMES_IN_FLIGHT, swapchain.images().len());

        Renderer {
            _entry: entry,
            instance: instance,
            surface: surface,
            _physical_device: physical_device,
            logical_device: logical_device,
            swapchain: swapchain,
            render_pass: render_pass,
            pipeline: pipeline,
            frame_buffers: framebuffer,
            command_pool: command_pool,
            command_buffers: command_buffers,
            sync_objects: sync_objects,
            current_frame: Cell::new(0),
        }
    }

    pub fn setup(&self) {
        for i in 0..self.frame_buffers.vulkan_object().len() {
            let begin_info = vk::CommandBufferBeginInfo::default();
            unsafe { self.get_device().begin_command_buffer(self.command_buffers.vulkan_object()[i], &begin_info).unwrap() };

            let clear_color = vk::ClearValue {
                color: vk::ClearColorValue { float32: [0.0f32, 0.1f32, 0.2f32, 1.0f32] },
            };
            let render_pass_info = vk::RenderPassBeginInfo::builder()
                .render_pass(*self.render_pass.vulkan_object())
                .framebuffer(self.frame_buffers.vulkan_object()[i])
                .render_area(vk::Rect2D::builder().offset(vk::Offset2D { x: 0, y: 0 }).extent(*self.swapchain.extent()).build())
                .clear_values(&[clear_color])
                .build();

            unsafe {
                self.get_device().cmd_begin_render_pass(self.command_buffers.vulkan_object()[i], &render_pass_info, vk::SubpassContents::INLINE);
                self.get_device().cmd_bind_pipeline(self.command_buffers.vulkan_object()[i], vk::PipelineBindPoint::GRAPHICS, *self.pipeline.vulkan_object());
                self.get_device().cmd_draw(self.command_buffers.vulkan_object()[i], 3, 1, 0, 0);
                self.get_device().cmd_end_render_pass(self.command_buffers.vulkan_object()[i]);
                self.get_device().end_command_buffer(self.command_buffers.vulkan_object()[i]).unwrap();
            };
        }
    }

    pub fn draw_frame(&self) {
        unsafe {
            self.get_device().wait_for_fences(&[self.sync_objects.in_flight_fences[self.current_frame.get()]], true, std::u64::MAX).unwrap();
        }

        let (image_index, _) = unsafe {
            self.swapchain
                .get_loader()
                .acquire_next_image(*self.swapchain.vulkan_object(), std::u64::MAX, self.sync_objects.image_available_semaphores[self.current_frame.get()], vk::Fence::null())
                .unwrap()
        };

        if self.sync_objects.images_in_flight.borrow()[image_index as usize] != vk::Fence::null() {
            unsafe {
                self.get_device().wait_for_fences(&[self.sync_objects.images_in_flight.borrow()[image_index as usize]], true, std::u64::MAX).unwrap();
            }
        }
        self.sync_objects.images_in_flight.borrow_mut()[image_index as usize] = self.sync_objects.in_flight_fences[self.current_frame.get()];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[self.sync_objects.image_available_semaphores[self.current_frame.get()]])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&[self.command_buffers.vulkan_object()[image_index as usize]])
            .signal_semaphores(&[self.sync_objects.render_finished_semaphores[self.current_frame.get()]])
            .build();

        unsafe {
            self.get_device().reset_fences(&[self.sync_objects.in_flight_fences[self.current_frame.get()]]).unwrap();
            self.get_device().queue_submit(*self.logical_device.graphics_queue(), &[submit_info], self.sync_objects.in_flight_fences[self.current_frame.get()]).unwrap();
        };

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[self.sync_objects.render_finished_semaphores[self.current_frame.get()]])
            .swapchains(&[*self.swapchain.vulkan_object()])
            .image_indices(&[image_index])
            .build();

        unsafe {
            self.get_swapchain_loader().queue_present(*self.logical_device.present_queue(), &present_info).unwrap();
        }

        self.current_frame.set((self.current_frame.get() + 1) % MAX_FRAMES_IN_FLIGHT);
    }

    pub(crate) fn get_device(&self) -> &Device {
        self.logical_device.vulkan_object()
    }

    pub(crate) fn get_swapchain_loader(&self) -> &khr::Swapchain {
        self.swapchain.get_loader()
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { self.get_device().device_wait_idle().unwrap() };

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                self.get_device().destroy_fence(self.sync_objects.in_flight_fences[i], None);
                self.get_device().destroy_semaphore(self.sync_objects.image_available_semaphores[i], None);
                self.get_device().destroy_semaphore(self.sync_objects.render_finished_semaphores[i], None);
            }
        }

        self.command_pool.cleanup(self);
        self.frame_buffers.cleanup(self);
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

struct SyncObjects {
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: RefCell<Vec<vk::Fence>>,
}

impl SyncObjects {
    fn new(device: &Device, max_frames: usize, num_images: usize) -> Self {
        let mut image_available_semaphores: Vec<vk::Semaphore> = Vec::new();
        let mut render_finished_semaphores: Vec<vk::Semaphore> = Vec::new();
        let mut in_flight_fences: Vec<vk::Fence> = Vec::new();
        let images_in_flight: RefCell<Vec<vk::Fence>> = RefCell::new(Vec::new());

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED).build();

        for _ in 0..max_frames {
            unsafe {
                in_flight_fences.push(device.create_fence(&fence_info, None).unwrap());
                image_available_semaphores.push(device.create_semaphore(&semaphore_info, None).unwrap());
                render_finished_semaphores.push(device.create_semaphore(&semaphore_info, None).unwrap());
            }
        }

        images_in_flight.borrow_mut().resize(num_images, vk::Fence::null());

        SyncObjects {
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        }
    }
}