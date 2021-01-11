use std::sync::Arc;

use super::{RenderPass, SwapChain};
use crate::render::{device::Device, VulkanObject};

use ash::{version::DeviceV1_0, vk};

//TODO: Rewrite this without vec
pub struct FrameBuffer {
    device: Arc<Device>,
    framebuffers: Vec<vk::Framebuffer>,
}

impl FrameBuffer {
    pub fn new(device: Arc<Device>, swapchain: &Arc<SwapChain>, render_pass: &Arc<RenderPass>) -> Arc<Self> {
        let mut framebuffers: Vec<vk::Framebuffer> = Vec::new();

        for &image_view in swapchain.image_views().iter() {
            let attachments = [image_view];

            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(*render_pass.vk())
                .attachments(&attachments)
                .width(swapchain.extent().width)
                .height(swapchain.extent().height)
                .layers(1)
                .build();

            unsafe { framebuffers.push(device.vk().create_framebuffer(&framebuffer_info, None).unwrap()) }
        }

        FrameBuffer { device, framebuffers }.into()
    }
}

impl VulkanObject for FrameBuffer {
    type Object = Vec<vk::Framebuffer>;

    fn vk(&self) -> &Self::Object {
        &self.framebuffers
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        trace!("Dropping Framebuffers");
        unsafe {
            for &framebuffer in self.framebuffers.iter() {
                self.device.vk().destroy_framebuffer(framebuffer, None);
            }
        }
    }
}
