use super::Buffer;
use crate::model::Vertex;
use crate::GraphicContext;
use crate::VulkanObject;

use ash::vk;

pub struct VertexBuffer {
    buffer: Buffer,
}

impl VertexBuffer {
    pub fn new(vertices: &[Vertex], context: &GraphicContext) -> VertexBuffer {
        let buffer_size = std::mem::size_of_val(vertices) as u64;
        let staging_buffer = Buffer::new(
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            context
        );
        staging_buffer.map_memory(vertices, context);

        let vertex_buffer = Buffer::new(
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            context
        );
        Buffer::copy_buffer(*staging_buffer.vulkan_object(), *vertex_buffer.vulkan_object(), buffer_size, context);
        staging_buffer.cleanup(context);

        VertexBuffer { buffer: vertex_buffer }
    }
}

impl VulkanObject for VertexBuffer {
    type Object = vk::Buffer;

    fn vulkan_object(&self) -> &Self::Object {
        &self.buffer.vulkan_object()
    }

    fn cleanup(&self, _context: &GraphicContext) {
        self.buffer.cleanup(_context);
    }
}