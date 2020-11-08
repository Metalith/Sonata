use std::sync::Arc;

use super::Buffer;

use crate::{device::Device, models::Vertex, VulkanObject};

use ash::vk;

pub struct VertexBuffer {
    vertices: Vec<Vertex>,
    buffer: Buffer,
}

impl VertexBuffer {
    pub fn new(vertices: &[Vertex], device: &Arc<Device>) -> VertexBuffer {
        let buffer_size = std::mem::size_of_val(vertices) as u64;
        let staging_buffer = Buffer::new(
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device.clone(),
        );
        staging_buffer.map_memory::<u32, _>(vertices);

        let vertex_buffer = Buffer::new(
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device.clone(),
        );
        Buffer::copy_buffer(&staging_buffer, &vertex_buffer, buffer_size, &device);

        VertexBuffer {
            vertices: vertices.to_vec(),
            buffer: vertex_buffer,
        }
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }
}

impl VulkanObject for VertexBuffer {
    type Object = vk::Buffer;

    fn vk(&self) -> &Self::Object {
        &self.buffer.vk()
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        trace!("Dropping Vertex Buffer");
    }
}
