use crate::VulkanObject;
use crate::Renderer;
use crate::SwapChain;

use ash::{
    vk,
    Device,
    version::DeviceV1_0
};

pub struct RenderPass {
    render_pass: vk::RenderPass
}

impl RenderPass {
    pub fn new(device: &Device,  swapchain: &SwapChain) -> RenderPass {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain.surface_format().format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();
            
        let sub_pass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref])
            .build();

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[color_attachment])
            .subpasses(&[sub_pass])
            .build();

        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None).unwrap() };

        RenderPass {
            render_pass: render_pass
        }
    }
}


impl VulkanObject for RenderPass {
    type Object = vk::RenderPass;

    fn vulkan_object(&self) -> &Self::Object {
        &self.render_pass
    }

    fn cleanup(&self, _renderer: &Renderer) {
        unsafe {
            _renderer.logical_device.vulkan_object().destroy_render_pass(self.render_pass, None);
        }
    }
}