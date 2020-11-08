use std::sync::Arc;

use crate::{device::Device, VulkanObject};

use ash::{version::DeviceV1_0, vk};

pub struct RenderPass {
    device: Arc<Device>,
    render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(device: Arc<Device>, format: vk::Format) -> Arc<RenderPass> {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL).build();

        let sub_pass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref])
            .build();

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::default())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[color_attachment])
            .subpasses(&[sub_pass])
            .dependencies(&[dependency])
            .build();

        let render_pass = unsafe { device.vk().create_render_pass(&render_pass_info, None).unwrap() };

        RenderPass { device, render_pass }.into()
    }
}

impl VulkanObject for RenderPass {
    type Object = vk::RenderPass;

    fn vk(&self) -> &Self::Object {
        &self.render_pass
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        trace!("Dropping Renderpass");
        unsafe {
            self.device.vk().destroy_render_pass(self.render_pass, None);
        }
    }
}
