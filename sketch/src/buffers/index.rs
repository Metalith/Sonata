use std::sync::Arc;

use super::Buffer;
use crate::{device::Device, VulkanObject};

use ash::vk;

pub struct IndexBuffer {
    indices: Vec<u16>,
    buffer: Buffer,
}

impl IndexBuffer {
    pub fn new(indices: &[u16], device: &Arc<Device>) -> IndexBuffer {
        let buffer_size = std::mem::size_of_val(indices) as u64;
        let staging_buffer = Buffer::new(
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device.clone(),
        );

        staging_buffer.map_memory::<u16, _>(indices);

        let index_buffer = Buffer::new(
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device.clone(),
        );
        Buffer::copy_buffer(&staging_buffer, &index_buffer, buffer_size, &device);

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

    fn vk(&self) -> &Self::Object {
        &self.buffer.vk()
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        trace!("Dropping Index Buffer");
    }
}
