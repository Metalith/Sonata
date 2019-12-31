use crate::RenderPass;
use crate::Renderer;
use crate::SwapChain;
use crate::VulkanObject;

use ash::{version::DeviceV1_0, vk, Device};

pub struct FrameBuffer {
    framebuffers: Vec<vk::Framebuffer>,
}

impl FrameBuffer {
    pub fn new(device: &Device, swapchain: &SwapChain, render_pass: &RenderPass) -> Self {
        let mut framebuffers: Vec<vk::Framebuffer> = Vec::new();

        for &image_view in swapchain.image_views().iter() {
            let attachments = [image_view];

            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(*render_pass.vulkan_object())
                .attachments(&attachments)
                .width(swapchain.extent().width)
                .height(swapchain.extent().height)
                .layers(1)
                .build();

            unsafe { framebuffers.push(device.create_framebuffer(&framebuffer_info, None).unwrap()) }
        }

        FrameBuffer { framebuffers: framebuffers }
    }
}

impl VulkanObject for FrameBuffer {
    type Object = Vec<vk::Framebuffer>;

    fn vulkan_object(&self) -> &Self::Object {
        &self.framebuffers
    }

    fn cleanup(&self, _renderer: &Renderer) {
        unsafe {
            for &framebuffer in self.framebuffers.iter() {
                _renderer.logical_device.vulkan_object().destroy_framebuffer(framebuffer, None);
            }
        }
    }
}
