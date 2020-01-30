use crate::VulkanObject;

use crate::commands::{CommandBuffer, CommandPool};
use crate::device::window::{HINSTANCE, HWND};
use crate::device::Window;
use crate::device::{Instance, LogicalDevice, PhysicalDevice, Surface};
use crate::model::Model;
use crate::model::Vertex;
use crate::pipeline::Pipeline;
use crate::renderpass::{FrameBuffer, RenderPass, SwapChain};
use crate::sync::SyncObjects;
use crate::utility::constants::*;

use ash::{extensions::khr, version::DeviceV1_0, vk, Device, Entry};

pub struct GraphicContext<'a> {
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
    pub sync_objects: SyncObjects,
    window: Window<'a>,
}

impl<'a> GraphicContext<'a> {
    pub fn new<T: Fn() -> (u32, u32) + 'a>(hwnd: HWND, hinstance: HINSTANCE, window_size_cb: T) -> GraphicContext<'a> {
        let entry = Entry::new().unwrap();
        let instance = Instance::new(&entry);
        let window = Window::new(window_size_cb);
        let surface = Surface::new(hwnd, hinstance, &entry, instance.vulkan_object());
        let physical_device = PhysicalDevice::new(instance.vulkan_object(), &surface);
        let logical_device = LogicalDevice::new(instance.vulkan_object(), &physical_device);
        let swapchain = SwapChain::new(instance.vulkan_object(), logical_device.vulkan_object(), &physical_device, &surface, &window);
        let render_pass = RenderPass::new(logical_device.vulkan_object(), &swapchain);
        let pipeline = Pipeline::new(logical_device.vulkan_object(), &render_pass);
        let framebuffer = FrameBuffer::new(logical_device.vulkan_object(), &swapchain, &render_pass);
        let command_pool = CommandPool::new(logical_device.vulkan_object(), &physical_device);
        let command_buffers = CommandBuffer::new(logical_device.vulkan_object(), &command_pool, framebuffer.vulkan_object().len() as u32);
        let sync_objects = SyncObjects::new(logical_device.vulkan_object(), MAX_FRAMES_IN_FLIGHT, swapchain.images().len());

        GraphicContext {
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
            window: window,
        }
    }

    pub fn create_model(&self, vertices: &[Vertex], indices: Option<&[u16]>) -> Model {
        Model::new(vertices, indices, &self)
    }

    pub fn get_logical_device(&self) -> &LogicalDevice {
        &self.logical_device
    }

    pub fn get_physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    pub fn get_device(&self) -> &Device {
        self.logical_device.vulkan_object()
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn get_swapchain_loader(&self) -> &khr::Swapchain {
        self.swapchain.get_loader()
    }

    pub fn get_command_pool(&self) -> &vk::CommandPool {
        self.command_pool.vulkan_object()
    }

    pub fn wait_device(&self) {
        unsafe { self.get_device().device_wait_idle().unwrap() };
    }

    pub fn acquire_next_image(&self) -> Result<usize, &'static str> {
        let image_result = unsafe {
            self.get_swapchain_loader()
                .acquire_next_image(*self.swapchain.vulkan_object(), std::u64::MAX, *self.sync_objects.get_image_semaphore(), vk::Fence::null())
        };

        match image_result {
            Ok((_, true)) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                return Err("Suboptimal swapchain");
            }
            Ok((image_index, false)) => {
                return Ok(image_index as usize);
            }
            _ => panic!("acquire swapchain image failed"),
        };
    }

    pub fn cleanup_swapchain(&self) {
        self.frame_buffers.cleanup(self);
        self.command_buffers.cleanup(self);
        self.render_pass.cleanup(self);
        self.swapchain.cleanup(self);
    }

    pub fn recreate_swapchain(&mut self) {
        self.wait_device();

        self.cleanup_swapchain();

        self.swapchain = SwapChain::new(self.instance.vulkan_object(), self.get_device(), &self.physical_device, &self.surface, &self.window);
        self.render_pass = RenderPass::new(self.get_device(), &self.swapchain);
        self.frame_buffers = FrameBuffer::new(self.get_device(), &self.swapchain, &self.render_pass);
        self.command_buffers = CommandBuffer::new(self.get_device(), &self.command_pool, self.frame_buffers.vulkan_object().len() as u32);
    }

    pub fn begin_command_buffer(&self, image_index: usize) {
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.0f32, 0.1f32, 0.2f32, 1.0f32] },
        };

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(*self.render_pass.vulkan_object())
            .framebuffer(self.frame_buffers.vulkan_object()[image_index])
            .render_area(vk::Rect2D::builder().offset(vk::Offset2D { x: 0, y: 0 }).extent(*self.swapchain.extent()).build())
            .clear_values(&[clear_color])
            .build();

        self.command_buffers
            .begin(image_index, self.get_device(), &render_pass_info, self.swapchain.viewport(), self.swapchain.scissor(), self.pipeline.vulkan_object());
    }

    pub fn end_command_buffer(&self, image_index: usize) {
        self.command_buffers.end(image_index, self.get_device());
    }

    pub fn render_models(&self, image_index: usize, models: &Vec<Model>) {
        for model in models.iter() {
            model.render(self.get_device(), self.command_buffers.get(image_index));
        }
    }

    pub fn submit_queue(&self, image_index: usize) {
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[*self.sync_objects.get_image_semaphore()])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&[*self.command_buffers.get(image_index)])
            .signal_semaphores(&[*self.sync_objects.get_render_semaphore()])
            .build();

        unsafe {
            self.get_device().reset_fences(&[self.sync_objects.get_flight_fence()]).unwrap();
            self.get_device().queue_submit(*self.logical_device.graphics_queue(), &[submit_info], self.sync_objects.get_flight_fence()).unwrap();
        };
    }

    pub fn present_queue(&self, image_index: u32) -> Result<(), &'static str> {
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[*self.sync_objects.get_render_semaphore()])
            .swapchains(&[*self.swapchain.vulkan_object()])
            .image_indices(&[image_index])
            .build();

        unsafe {
            let result = self.get_swapchain_loader().queue_present(*self.logical_device.present_queue(), &present_info);
            match result {
                Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return Err("Out of date swapchain"),
                Ok(false) => {
                    if self.window.has_window_resized() {
                        return Err("Wrong size swapchain");
                    }
                }
                _ => panic!("window present failed"),
            }
        }

        Ok(())
    }

    pub fn cleanup(&self) {
        self.cleanup_swapchain();
        self.sync_objects.cleanup(self);
        self.command_pool.cleanup(self);
        self.pipeline.cleanup(self);
        self.logical_device.cleanup(self);
        self.surface.cleanup(self);
        self.instance.cleanup(self);
    }
}
