use crate::VulkanObject;

use crate::buffers::Buffer;
use crate::commands::{CommandBuffer, CommandPool};
use crate::device::{Instance, LogicalDevice, PhysicalDevice, Surface};
use crate::model::Vertex;
use crate::pipeline::Pipeline;
use crate::renderpass::{FrameBuffer, RenderPass, SwapChain};
use crate::utility::constants::*;

use ash::{extensions::khr, version::DeviceV1_0, vk, Device, Entry};
use winit::window::Window;

use std::cell::RefCell;

pub struct Renderer<'a> {
    _entry: Entry,
    instance: Instance,
    surface: Surface,
    physical_device: PhysicalDevice,
    logical_device: LogicalDevice,
    swapchain: SwapChain,
    render_pass: RenderPass,
    pipeline: Pipeline,
    frame_buffers: FrameBuffer,
    command_pool: CommandPool,
    command_buffers: CommandBuffer,
    sync_objects: SyncObjects,
    vertex_buffer: Buffer,
    current_frame: usize,
    last_win_size: (u32, u32),
    window_size_cb: Box<dyn Fn() -> (u32, u32) + 'a>,
}

impl<'a> Renderer<'a> {
    pub fn new<T: Fn() -> (u32, u32) + 'a>(win: &'a Window, window_size_cb: T, vertices: &[Vertex]) -> Renderer<'a> {
        let entry = Entry::new().unwrap();
        let instance = Instance::new(&entry);
        let surface = Surface::new(win, &entry, instance.vulkan_object());
        let physical_device = PhysicalDevice::new(instance.vulkan_object(), &surface);
        let logical_device = LogicalDevice::new(instance.vulkan_object(), &physical_device);
        let swapchain = SwapChain::new(instance.vulkan_object(), logical_device.vulkan_object(), &physical_device, &surface, &window_size_cb);
        let render_pass = RenderPass::new(logical_device.vulkan_object(), &swapchain);
        let pipeline = Pipeline::new(logical_device.vulkan_object(), &render_pass);
        let framebuffer = FrameBuffer::new(logical_device.vulkan_object(), &swapchain, &render_pass);
        let command_pool = CommandPool::new(logical_device.vulkan_object(), &physical_device);
        let command_buffers = CommandBuffer::new(logical_device.vulkan_object(), &command_pool, framebuffer.vulkan_object().len() as u32);
        let sync_objects = SyncObjects::new(logical_device.vulkan_object(), MAX_FRAMES_IN_FLIGHT, swapchain.images().len());
        let vertex_buffer = Buffer::new(vertices, logical_device.vulkan_object(), &physical_device);

        Renderer {
            _entry: entry,
            instance: instance,
            surface: surface,
            physical_device: physical_device,
            logical_device: logical_device,
            swapchain: swapchain,
            render_pass: render_pass,
            pipeline: pipeline,
            frame_buffers: framebuffer,
            command_pool: command_pool,
            command_buffers: command_buffers,
            sync_objects: sync_objects,
            vertex_buffer: vertex_buffer,
            current_frame: 0,
            last_win_size: window_size_cb(),
            window_size_cb: Box::new(window_size_cb),
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

            let viewport = vk::Viewport::builder()
                .x(0f32)
                .y(0f32)
                .width(self.swapchain.extent().width as f32)
                .height(self.swapchain.extent().height as f32)
                .min_depth(0f32)
                .max_depth(1f32)
                .build();
            let scissor = vk::Rect2D::builder().offset(vk::Offset2D { x: 0, y: 0 }).extent(*self.swapchain.extent()).build();

            unsafe {
                self.get_device().cmd_begin_render_pass(self.command_buffers.vulkan_object()[i], &render_pass_info, vk::SubpassContents::INLINE);
                self.get_device().cmd_bind_pipeline(self.command_buffers.vulkan_object()[i], vk::PipelineBindPoint::GRAPHICS, *self.pipeline.vulkan_object());
                self.get_device().cmd_set_viewport(self.command_buffers.vulkan_object()[i], 0, &[viewport]);
                self.get_device().cmd_set_scissor(self.command_buffers.vulkan_object()[i], 0, &[scissor]);

                let buffers = [*self.vertex_buffer.vulkan_object()];
                let offsets = [0];
                self.get_device().cmd_bind_vertex_buffers(self.command_buffers.vulkan_object()[i], 0, &buffers, &offsets);
                self.get_device().cmd_draw(self.command_buffers.vulkan_object()[i], 3, 1, 0, 0);
                self.get_device().cmd_end_render_pass(self.command_buffers.vulkan_object()[i]);
                self.get_device().end_command_buffer(self.command_buffers.vulkan_object()[i]).unwrap();
            };
        }
    }

    fn recreate_swapchain(&mut self) {
        if self.window_is_minimized() {
            return;
        }

        unsafe { self.get_device().device_wait_idle().unwrap() };

        self.cleanup_swapchain();

        self.swapchain = SwapChain::new(self.instance.vulkan_object(), self.get_device(), &self.physical_device, &self.surface, &self.window_size_cb);
        self.render_pass = RenderPass::new(self.get_device(), &self.swapchain);
        self.frame_buffers = FrameBuffer::new(self.get_device(), &self.swapchain, &self.render_pass);
        self.command_buffers = CommandBuffer::new(self.get_device(), &self.command_pool, self.frame_buffers.vulkan_object().len() as u32);

        self.setup();
    }

    fn window_is_minimized(&self) -> bool {
        self.get_window_size() == (0, 0)
    }

    pub fn draw_frame(&mut self) {
        unsafe {
            self.get_device().wait_for_fences(&[self.sync_objects.in_flight_fences[self.current_frame]], true, std::u64::MAX).unwrap();
        }

        let image_result = unsafe {
            self.get_swapchain_loader()
                .acquire_next_image(*self.swapchain.vulkan_object(), std::u64::MAX, self.sync_objects.image_available_semaphores[self.current_frame], vk::Fence::null())
        };

        match image_result {
            Ok((_, true)) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                if self.window_is_minimized() {
                    return;
                }
                self.recreate_swapchain();
                return;
            }
            Ok((_, false)) => {}
            _ => panic!("acquire swapchain image failed"),
        };

        let (image_index_32, _) = image_result.unwrap();
        let image_index = image_index_32 as usize;

        if self.sync_objects.get_image_in_flight(image_index) != vk::Fence::null() {
            unsafe {
                self.get_device().wait_for_fences(&[self.sync_objects.get_image_in_flight(image_index)], true, std::u64::MAX).unwrap();
            }
        }
        self.sync_objects.set_image_in_flight(image_index, self.current_frame);

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[self.sync_objects.image_available_semaphores[self.current_frame]])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&[self.command_buffers.vulkan_object()[image_index]])
            .signal_semaphores(&[self.sync_objects.render_finished_semaphores[self.current_frame]])
            .build();

        unsafe {
            self.get_device().reset_fences(&[self.sync_objects.in_flight_fences[self.current_frame]]).unwrap();
            self.get_device()
                .queue_submit(*self.logical_device.graphics_queue(), &[submit_info], self.sync_objects.in_flight_fences[self.current_frame])
                .unwrap();
        };

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[self.sync_objects.render_finished_semaphores[self.current_frame]])
            .swapchains(&[*self.swapchain.vulkan_object()])
            .image_indices(&[image_index_32])
            .build();

        unsafe {
            let result = self.get_swapchain_loader().queue_present(*self.logical_device.present_queue(), &present_info);
            match result {
                Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.recreate_swapchain(),
                Ok(false) => {
                    if self.last_win_size != self.get_window_size() {
                        self.recreate_swapchain()
                    }
                }
                _ => panic!("window present failed"),
            }
        }

        self.last_win_size = self.get_window_size();
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn cleanup_swapchain(&self) {
        self.frame_buffers.cleanup(self);
        self.command_buffers.cleanup(self);
        self.render_pass.cleanup(self);
        self.swapchain.cleanup(self);
    }

    pub(crate) fn get_device(&self) -> &Device {
        self.logical_device.vulkan_object()
    }

    pub(crate) fn get_swapchain_loader(&self) -> &khr::Swapchain {
        self.swapchain.get_loader()
    }

    pub(crate) fn get_command_pool(&self) -> &vk::CommandPool {
        self.command_pool.vulkan_object()
    }

    fn get_window_size(&self) -> (u32, u32) {
        (*self.window_size_cb)()
    }
}

impl<'a> Drop for Renderer<'a> {
    fn drop(&mut self) {
        unsafe { self.get_device().device_wait_idle().unwrap() };

        self.cleanup_swapchain();
        self.vertex_buffer.cleanup(self);
        self.sync_objects.cleanup(self);
        self.command_pool.cleanup(self);
        self.pipeline.cleanup(self);
        self.logical_device.cleanup(self);
        self.surface.cleanup(self);
        self.instance.cleanup(self);
    }
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

    fn cleanup(&self, _renderer: &Renderer) {
        for i in 0..self.in_flight_fences.len() {
            unsafe {
                _renderer.get_device().destroy_fence(self.in_flight_fences[i], None);
                _renderer.get_device().destroy_semaphore(self.image_available_semaphores[i], None);
                _renderer.get_device().destroy_semaphore(self.render_finished_semaphores[i], None);
            }
        }
    }

    fn set_image_in_flight(&self, index: usize, frame: usize) {
        self.images_in_flight.borrow_mut()[index] = self.in_flight_fences[frame];
    }

    fn get_image_in_flight(&self, index: usize) -> vk::Fence {
        self.images_in_flight.borrow()[index]
    }
}
