use super::Buffer;
use crate::GraphicContext;
use crate::VulkanObject;

use ash::vk;

pub struct IndexBuffer {
    indices: Vec<u16>,
    buffer: Buffer,
}

impl IndexBuffer {
    pub fn new(indices: &[u16], context: &GraphicContext) -> IndexBuffer {
        let buffer_size = std::mem::size_of_val(indices) as u64;
        let staging_buffer = Buffer::new(buffer_size, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, context.get_device(), context.get_physical_device());

        staging_buffer.map_memory::<u16, _>(indices, context);

        let index_buffer = Buffer::new(buffer_size, vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER, vk::MemoryPropertyFlags::DEVICE_LOCAL, context.get_device(), context.get_physical_device());
        Buffer::copy_buffer(*staging_buffer.vulkan_object(), *index_buffer.vulkan_object(), buffer_size, context);
        staging_buffer.cleanup(context);

        IndexBuffer {
            indices: indices.to_vec(),
            buffer: index_buffer,
        }
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }
}

impl VulkanObject for IndexBuffer {
    type Object = vk::Buffer;

    fn vulkan_object(&self) -> &Self::Object {
        &self.buffer.vulkan_object()
    }

    fn cleanup(&self, _context: &GraphicContext) {
        self.buffer.cleanup(_context);
    }
}
