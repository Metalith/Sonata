mod buffers;
mod commands;
mod constants;
pub mod device;
pub mod models;
mod pipelines;
mod renderpasses;
mod sync;
mod utilities;

use ash::{version::DeviceV1_0, vk};
use imgui_rs_vulkan_renderer::RendererVkContext;

use std::{sync::Arc, time::Instant};

use buffers::{UniformBufferObject, UniformTestObject};
use commands::CommandBuffer;
use constants::*;
use device::{
    window::{HINSTANCE, HWND},
    DebugMessenger, Device, Instance, PhysicalDevice, Surface, Window,
};
use pipelines::{DescriptorLayout, DescriptorPoolAlloc, Pipeline};
use renderpasses::{FrameBuffer, RenderPass, SwapChain};
use sync::SyncObjects;

use models::MeshFactory;

pub struct GraphicContext {
    _instance: Arc<Instance>,
    _debug_messenger: Option<Arc<DebugMessenger>>,
    surface: Arc<Surface>,
    device: Arc<Device>,
    swapchain: Arc<SwapChain>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<Pipeline>,
    frame_buffers: Arc<FrameBuffer>,
    command_buffers: Arc<CommandBuffer>,
    pub sync_objects: SyncObjects,
    window: Window,
    start_time: Instant,
    uniform_buffers: Vec<UniformBufferObject>,
    descriptor_layout: Arc<DescriptorLayout>,
    descriptor_set: Arc<DescriptorPoolAlloc>,
}

impl GraphicContext {
    pub fn new(hwnd: HWND, hinstance: HINSTANCE) -> GraphicContext {
        let validation_enabled = validation_enabled();

        let instance = Instance::new(validation_enabled);
        let debug_messenger = if validation_enabled { Some(DebugMessenger::new(&instance)) } else { None };

        let window = Window::new(hwnd);
        let surface = Surface::new(hwnd, hinstance, instance.clone());

        let physical_device = PhysicalDevice::new(instance.clone(), &surface);
        let device = Device::new(physical_device, validation_enabled);

        let swapchain = SwapChain::new(device.clone(), surface.clone(), &window, None);

        let render_pass = RenderPass::new(device.clone(), swapchain.surface_format().format);
        let descriptor_layout = DescriptorLayout::new(device.clone());
        let pipeline = Pipeline::new(device.clone(), &render_pass, &descriptor_layout);
        let framebuffer = FrameBuffer::new(device.clone(), &swapchain, &render_pass);
        let command_buffers = CommandBuffer::new(device.clone(), framebuffer.vk().len() as u32);
        let sync_objects = SyncObjects::new(device.clone(), MAX_FRAMES_IN_FLIGHT, swapchain.images().len());
        let start_time = Instant::now();

        let mut u_buffers = Vec::new();
        for _ in 0..swapchain.images().len() {
            u_buffers.push(UniformBufferObject::new(&device));
        }

        let layouts = (0..swapchain.images().len()).map(|_| descriptor_layout.clone()).collect::<Vec<_>>();
        let descriptor_set = device.descriptor_pool().alloc(&layouts);
        descriptor_set.update(&u_buffers);

        GraphicContext {
            _instance: instance,
            _debug_messenger: debug_messenger,
            surface,
            device,
            swapchain,
            render_pass,
            pipeline,
            frame_buffers: framebuffer,
            command_buffers,
            sync_objects,
            window,
            start_time,
            uniform_buffers: u_buffers,
            descriptor_layout,
            descriptor_set,
        }
    }

    pub fn get_device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn get_render_pass(&self) -> &vk::RenderPass {
        self.render_pass.vk()
    }

    pub fn wait_device(&self) {
        unsafe { self.device.vk().device_wait_idle().unwrap() };
    }

    pub fn acquire_next_image(&self) -> Result<usize, &'static str> {
        let image_result = unsafe {
            self.swapchain
                .get_loader()
                .acquire_next_image(*self.swapchain.vk(), std::u64::MAX, *self.sync_objects.get_image_semaphore(), vk::Fence::null())
        };

        match image_result {
            Ok((_, true)) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err("Suboptimal swapchain"),
            Ok((image_index, false)) => Ok(image_index as usize),
            _ => panic!("acquire swapchain image failed"),
        }
    }

    pub fn recreate_swapchain(&mut self) {
        trace!("Recreating wapchain");
        self.wait_device();

        self.swapchain = SwapChain::new(self.device.clone(), self.surface.clone(), &self.window, Some(&self.swapchain));
        self.render_pass = RenderPass::new(self.device.clone(), self.swapchain.surface_format().format);
        self.frame_buffers = FrameBuffer::new(self.device.clone(), &self.swapchain, &self.render_pass);
        self.uniform_buffers = Vec::new();
        for _ in 0..self.swapchain.images().len() {
            self.uniform_buffers.push(UniformBufferObject::new(&self.device));
        }

        let layouts = (0..self.swapchain.images().len()).map(|_| self.descriptor_layout.clone()).collect::<Vec<_>>();
        self.descriptor_set = self.device.descriptor_pool().alloc(&layouts);
        self.descriptor_set.update(&self.uniform_buffers);
        self.command_buffers = CommandBuffer::new(self.device.clone(), self.frame_buffers.vk().len() as u32);
    }

    pub fn begin_command_buffer(&self, image_index: usize) {
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0f32, 0.1f32, 0.2f32, 1.0f32],
            },
        };

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(*self.render_pass.vk())
            .framebuffer(self.frame_buffers.vk()[image_index])
            .render_area(vk::Rect2D::builder().offset(vk::Offset2D { x: 0, y: 0 }).extent(*self.swapchain.extent()).build())
            .clear_values(&[clear_color])
            .build();

        self.command_buffers.begin(image_index, &render_pass_info);
        self.command_buffers.bind_pipeline(image_index, self.pipeline.vk());
        self.command_buffers.set_scissor(image_index, self.swapchain.scissor());
        self.command_buffers.set_viewport(image_index, self.swapchain.viewport());
        self.command_buffers
            .bind_descriptor_sets(image_index, self.pipeline.get_layout(), &self.descriptor_set.vk()[image_index..=image_index]);
    }

    pub fn get_command_buffer(&self, image_index: usize) -> &vk::CommandBuffer {
        self.command_buffers.get(image_index)
    }

    pub fn end_command_buffer(&self, image_index: usize) {
        self.command_buffers.end(image_index);
    }

    pub fn submit_queue(&self, image_index: usize) {
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[*self.sync_objects.get_image_semaphore()])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&[*self.command_buffers.get(image_index)])
            .signal_semaphores(&[*self.sync_objects.get_render_semaphore()])
            .build();

        unsafe {
            self.device.vk().reset_fences(&[self.sync_objects.get_flight_fence()]).unwrap();
            self.device
                .vk()
                .queue_submit(*self.device.graphics_queue(), &[submit_info], self.sync_objects.get_flight_fence())
                .unwrap();
        };
    }

    pub fn present_queue(&self, image_index: u32) -> Result<(), &'static str> {
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[*self.sync_objects.get_render_semaphore()])
            .swapchains(&[*self.swapchain.vk()])
            .image_indices(&[image_index])
            .build();

        unsafe {
            let result = self.swapchain.get_loader().queue_present(*self.device.present_queue(), &present_info);
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

    pub fn update_uniforms(&self, image_index: usize, camera_pos: &uv::Vec3, camera_dir: &uv::Vec3, camera_up: &uv::Vec3) {
        let time = Instant::now().duration_since(self.start_time).as_millis();

        let aspect = self.swapchain.extent().width as f32 / self.swapchain.extent().height as f32;

        // let model = glm::rotate(&glm::Mat4::identity(), (time as f32 * 0.180).to_radians(), &glm::Vec3::new(0.0, 0.0, 1.0));
        let model = uv::Mat4::identity();

        let view = uv::Mat4::look_at(*camera_pos, *camera_pos + *camera_dir, *camera_up);

        let mut proj = uv::projection::perspective_gl(45f32.to_radians(), aspect, 0.1, 10.0);
        proj[1][1] *= -1.0;

        let ubo = UniformTestObject { model, view, proj };
        let ubos = [ubo];

        self.uniform_buffers[image_index].update2::<f32, _>(&ubos);
    }

    pub fn create_mesh_factory(&self) -> MeshFactory {
        MeshFactory::new(self.device.clone())
    }
}

impl RendererVkContext for GraphicContext {
    fn instance(&self) -> &ash::Instance {
        self.device.instance().vk()
    }

    fn physical_device(&self) -> ash::vk::PhysicalDevice {
        *self.device.physical_device().vk()
    }

    fn device(&self) -> &ash::Device {
        self.device.vk()
    }

    fn queue(&self) -> vk::Queue {
        *self.device.graphics_queue()
    }

    fn command_pool(&self) -> vk::CommandPool {
        *self.device.command_pool().vk()
    }
}

fn validation_enabled() -> bool {
    if std::env::var("WIND_VK_VALIDATION").is_ok() {
        std::env::var("WIND_VK_VALIDATION").unwrap().parse::<bool>().unwrap()
    } else {
        false
    }
}

trait VulkanObject {
    type Object;
    fn vk(&self) -> &Self::Object;
}
