use std::sync::Arc;

use super::CommandPool;

use crate::render::{device::Device, VulkanObject};

use ash::{version::DeviceV1_0, vk};

pub struct CommandBuffer {
    device: Arc<Device>,
    command_pool: Arc<CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,
}

impl CommandBuffer {
    pub fn new(device: Arc<Device>, count: u32) -> Arc<Self> {
        let command_pool = device.command_pool();
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*command_pool.vk())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count)
            .build();

        let command_buffers = unsafe { device.vk().allocate_command_buffers(&alloc_info).unwrap() };

        CommandBuffer {
            device,
            command_pool,
            command_buffers,
        }
        .into()
    }

    pub fn begin(&self, index: usize, render_pass_info: &vk::RenderPassBeginInfo) {
        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            self.device.vk().begin_command_buffer(self.command_buffers[index], &begin_info).unwrap();
            self.device.vk().cmd_begin_render_pass(self.command_buffers[index], &render_pass_info, vk::SubpassContents::INLINE);
        };
    }

    pub fn bind_pipeline(&self, index: usize, pipeline: &vk::Pipeline) {
        unsafe {
            self.device.vk().cmd_bind_pipeline(self.command_buffers[index], vk::PipelineBindPoint::GRAPHICS, *pipeline);
        }
    }

    pub fn set_viewport(&self, index: usize, viewport: vk::Viewport) {
        unsafe {
            self.device.vk().cmd_set_viewport(self.command_buffers[index], 0, &[viewport]);
        }
    }

    pub fn set_scissor(&self, index: usize, scissor: vk::Rect2D) {
        unsafe {
            self.device.vk().cmd_set_scissor(self.command_buffers[index], 0, &[scissor]);
        }
    }

    pub fn bind_descriptor_sets(&self, index: usize, pipeline_layout: &vk::PipelineLayout, descriptor_sets: &[vk::DescriptorSet]) {
        unsafe {
            let null = [];
            self.device
                .vk()
                .cmd_bind_descriptor_sets(self.command_buffers[index], vk::PipelineBindPoint::GRAPHICS, *pipeline_layout, 0, descriptor_sets, &null);
        }
    }

    pub fn end(&self, index: usize) {
        unsafe {
            self.device.vk().cmd_end_render_pass(self.command_buffers[index]);
            self.device.vk().end_command_buffer(self.command_buffers[index]).unwrap();
        };
    }

    pub fn get(&self, image_index: usize) -> &vk::CommandBuffer {
        &self.command_buffers[image_index]
    }
}

impl VulkanObject for CommandBuffer {
    type Object = Vec<vk::CommandBuffer>;

    fn vk(&self) -> &Self::Object {
        &self.command_buffers
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        trace!("Dropping Command Buffer");
        unsafe {
            self.device.vk().free_command_buffers(*self.command_pool.vk(), &self.command_buffers);
        }
    }
}
