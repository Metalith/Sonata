use super::SwapChain;
use crate::GraphicContext;
use crate::VulkanObject;

use ash::{version::DeviceV1_0, vk, Device};

pub struct RenderPass {
    render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(device: &Device, swapchain: &SwapChain) -> RenderPass {
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

        let color_attachment_ref = vk::AttachmentReference::builder().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL).build();

        let sub_pass = vk::SubpassDescription::builder().pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS).color_attachments(&[color_attachment_ref]).build();

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::default())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        let render_pass_info = vk::RenderPassCreateInfo::builder().attachments(&[color_attachment]).subpasses(&[sub_pass]).dependencies(&[dependency]).build();

        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None).unwrap() };

        RenderPass { render_pass: render_pass }
    }
}

impl VulkanObject for RenderPass {
    type Object = vk::RenderPass;

    fn vulkan_object(&self) -> &Self::Object {
        &self.render_pass
    }

    fn cleanup(&self, _context: &GraphicContext) {
        unsafe {
            _context.get_device().destroy_render_pass(self.render_pass, None);
        }
    }
}
