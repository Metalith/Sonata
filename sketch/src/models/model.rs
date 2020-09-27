use super::Vertex;
use crate::{
    buffers::{IndexBuffer, VertexBuffer},
    GraphicContext, VulkanObject,
};

use ash::{version::DeviceV1_0, vk, Device};

pub struct Model {
    vertex_buffer: VertexBuffer,
    index_buffer: Option<IndexBuffer>,
}

impl Model {
    pub fn new(vertices: &[Vertex], indices: Option<&[u16]>, context: &GraphicContext) -> Model {
        let vertex_buffer = VertexBuffer::new(vertices, context);
        let index_buffer = match indices {
            Some(indices) => Some(IndexBuffer::new(indices, context)),
            None => None,
        };

        Model { vertex_buffer, index_buffer }
    }

    pub fn render(&self, device: &Device, command_buffer: &vk::CommandBuffer) {
        let vertex_buffers = [*self.vertex_buffer.vulkan_object()];
        let offsets = [0];
        unsafe {
            if let Some(index_buffer) = &self.index_buffer {
                device.cmd_bind_vertex_buffers(*command_buffer, 0, &vertex_buffers, &offsets);
                device.cmd_bind_index_buffer(*command_buffer, *index_buffer.vulkan_object(), 0, vk::IndexType::UINT16);
                device.cmd_draw_indexed(*command_buffer, index_buffer.index_count(), 1, 0, 0, 0);
            } else {
                device.cmd_bind_vertex_buffers(*command_buffer, 0, &vertex_buffers, &offsets);
                device.cmd_draw(*command_buffer, self.vertex_buffer.vertex_count(), 1, 0, 0);
            }
        }
    }

    pub fn cleanup(&self, _context: &GraphicContext) {
        self.vertex_buffer.cleanup(_context);
        if let Some(index_buffer) = &self.index_buffer {
            index_buffer.cleanup(_context);
        }
    }
}
