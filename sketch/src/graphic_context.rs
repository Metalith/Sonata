use crate::VulkanObject;

use crate::{
    buffers::{UniformBufferObject, UniformTestObject},
    commands::{CommandBuffer, CommandPool},
    device::{
        window::{HINSTANCE, HWND},
        Instance, LogicalDevice, PhysicalDevice, Surface, Window,
    },
    models::{Model, Vertex},
    pipelines::{DescriptorLayout, DescriptorPool, DescriptorSet, Pipeline},
    renderpasses::{FrameBuffer, RenderPass, SwapChain},
    sync::SyncObjects,
    utilities::constants::*,
};

use ash::{extensions::khr, version::DeviceV1_0, vk, Device, Entry};

use imgui_rs_vulkan_renderer::RendererVkContext;

use std::time::Instant;

pub struct GraphicContext {
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
    window: Window,
    start_time: Instant,
    uniform_buffers: Vec<UniformBufferObject>,
    descriptor_layout: DescriptorLayout,
    descriptor_pool: DescriptorPool,
    descriptor_set: DescriptorSet,
}

impl GraphicContext {
    pub fn new(hwnd: HWND, hinstance: HINSTANCE) -> GraphicContext {
        let entry = Entry::new().unwrap();
        let instance = Instance::new(&entry);
        let window = Window::new(hwnd);
        let surface = Surface::new(hwnd, hinstance, &entry, instance.vulkan_object());
        let physical_device = PhysicalDevice::new(instance.vulkan_object(), &surface);
        let logical_device = LogicalDevice::new(instance.vulkan_object(), &physical_device);
        let swapchain = SwapChain::new(instance.vulkan_object(), logical_device.vulkan_object(), &physical_device, &surface, &window);
        let render_pass = RenderPass::new(logical_device.vulkan_object(), &swapchain);
        let descriptor_layout = DescriptorLayout::new(logical_device.vulkan_object());
        let pipeline = Pipeline::new(logical_device.vulkan_object(), &render_pass, *descriptor_layout.vulkan_object());
        let framebuffer = FrameBuffer::new(logical_device.vulkan_object(), &swapchain, &render_pass);
        let command_pool = CommandPool::new(logical_device.vulkan_object(), &physical_device);
        let command_buffers = CommandBuffer::new(logical_device.vulkan_object(), &command_pool, framebuffer.vulkan_object().len() as u32);
        let sync_objects = SyncObjects::new(logical_device.vulkan_object(), MAX_FRAMES_IN_FLIGHT, swapchain.images().len());
        let start_time = Instant::now();

        let mut u_buffers = Vec::new();
        for _ in 0..swapchain.images().len() {
            u_buffers.push(UniformBufferObject::new(logical_device.vulkan_object(), &physical_device));
        }

        let descriptor_pool = DescriptorPool::new(logical_device.vulkan_object(), swapchain.images().len() as u32);
        let descriptor_set = DescriptorSet::new(
            logical_device.vulkan_object(),
            *descriptor_layout.vulkan_object(),
            swapchain.images().len() as u32,
            &descriptor_pool,
            &u_buffers,
        );

        GraphicContext {
            _entry: entry,
            instance,
            surface,
            physical_device,
            logical_device,
            swapchain,
            render_pass,
            pipeline,
            frame_buffers: framebuffer,
            command_pool,
            command_buffers,
            sync_objects,
            window,
            start_time,
            uniform_buffers: u_buffers,
            descriptor_layout,
            descriptor_pool,
            descriptor_set,
        }
    }

    pub fn create_model(&self, vertices: &[Vertex], indices: Option<&[u16]>) -> Model {
        Model::new(vertices, indices, &self)
    }

    pub fn get_instance(&self) -> &ash::Instance {
        &self.instance.vulkan_object()
    }

    pub fn get_logical_device(&self) -> &LogicalDevice {
        &self.logical_device
    }

    pub fn get_physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }
    pub fn get_vk_physical_device(&self) -> &vk::PhysicalDevice {
        &self.physical_device.vulkan_object()
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

    pub fn get_render_pass(&self) -> &vk::RenderPass {
        self.render_pass.vulkan_object()
    }

    pub fn get_image_count(&self) -> usize {
        self.swapchain.images().len()
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
            Ok((_, true)) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Err("Suboptimal swapchain"),
            Ok((image_index, false)) => Ok(image_index as usize),
            _ => panic!("acquire swapchain image failed"),
        }
    }

    pub fn cleanup_swapchain(&self) {
        self.frame_buffers.cleanup(self);
        self.command_buffers.cleanup(self);
        self.render_pass.cleanup(self);
        self.swapchain.cleanup(self);

        for ubuffer in self.uniform_buffers.iter() {
            ubuffer.cleanup(self);
        }

        self.descriptor_pool.cleanup(self);
    }

    pub fn recreate_swapchain(&mut self) {
        self.wait_device();

        self.cleanup_swapchain();

        self.swapchain = SwapChain::new(self.instance.vulkan_object(), self.get_device(), &self.physical_device, &self.surface, &self.window);
        self.render_pass = RenderPass::new(self.get_device(), &self.swapchain);
        self.frame_buffers = FrameBuffer::new(self.get_device(), &self.swapchain, &self.render_pass);
        self.uniform_buffers = Vec::new();
        for _ in 0..self.swapchain.images().len() {
            self.uniform_buffers.push(UniformBufferObject::new(self.get_device(), &self.physical_device));
        }
        self.descriptor_pool = DescriptorPool::new(self.get_device(), self.get_image_count() as u32);
        self.descriptor_set = DescriptorSet::new(
            self.get_device(),
            *self.descriptor_layout.vulkan_object(),
            self.get_image_count() as u32,
            &self.descriptor_pool,
            &self.uniform_buffers,
        );
        self.command_buffers = CommandBuffer::new(self.get_device(), &self.command_pool, self.frame_buffers.vulkan_object().len() as u32);
    }

    pub fn begin_command_buffer(&self, image_index: usize) {
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0f32, 0.1f32, 0.2f32, 1.0f32],
            },
        };

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(*self.render_pass.vulkan_object())
            .framebuffer(self.frame_buffers.vulkan_object()[image_index])
            .render_area(vk::Rect2D::builder().offset(vk::Offset2D { x: 0, y: 0 }).extent(*self.swapchain.extent()).build())
            .clear_values(&[clear_color])
            .build();

        self.command_buffers.begin(image_index, self.get_device(), &render_pass_info);
        self.command_buffers.bind_pipeline(image_index, self.get_device(), self.pipeline.vulkan_object());
        self.command_buffers.set_scissor(image_index, self.get_device(), self.swapchain.scissor());
        self.command_buffers.set_viewport(image_index, self.get_device(), self.swapchain.viewport());
        self.command_buffers.bind_descriptor_sets(
            image_index,
            self.get_device(),
            self.pipeline.get_layout(),
            &self.descriptor_set.vulkan_object()[image_index..=image_index],
        );
    }

    pub fn get_command_buffer(&self, image_index: usize) -> &vk::CommandBuffer {
        self.command_buffers.get(image_index)
    }

    pub fn end_command_buffer(&self, image_index: usize) {
        self.command_buffers.end(image_index, self.get_device());
    }

    pub fn render_models(&self, image_index: usize, models: &[Model]) {
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
            self.get_device()
                .queue_submit(*self.logical_device.graphics_queue(), &[submit_info], self.sync_objects.get_flight_fence())
                .unwrap();
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

        self.uniform_buffers[image_index].update2::<f32, _>(self, &ubos);
    }

    pub fn cleanup(&self) {
        self.cleanup_swapchain();
        self.descriptor_layout.cleanup(self);
        self.sync_objects.cleanup(self);
        self.command_pool.cleanup(self);
        self.pipeline.cleanup(self);
        self.logical_device.cleanup(self);
        self.surface.cleanup(self);
        self.instance.cleanup(self);
    }
}

impl RendererVkContext for GraphicContext {
    fn instance(&self) -> &ash::Instance {
        &self.get_instance()
    }

    fn physical_device(&self) -> ash::vk::PhysicalDevice {
        *self.get_vk_physical_device()
    }

    fn device(&self) -> &ash::Device {
        &self.get_device()
    }

    fn queue(&self) -> vk::Queue {
        *self.get_logical_device().graphics_queue()
    }

    fn command_pool(&self) -> vk::CommandPool {
        *self.get_command_pool()
    }
}
