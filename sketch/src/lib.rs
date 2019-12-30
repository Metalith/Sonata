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

use ash::Entry;
use winit::window::Window;

pub struct Renderer {
    pub entry: Entry,
    pub instance: Instance,
    pub surface: Surface,
    pub physical_device: PhysicalDevice,
    pub logical_device: LogicalDevice,
    pub swapchain: SwapChain,
    pub render_pass: RenderPass,
    pub pipeline: Pipeline
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

        Renderer {
            entry: entry,
            instance: instance,
            surface: surface,
            physical_device: physical_device,
            logical_device: logical_device,
            swapchain: swapchain,
            render_pass: render_pass,
            pipeline: pipeline
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
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
