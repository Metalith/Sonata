use std::sync::Arc;

use super::Vertex;
use crate::render::{
    buffers::{IndexBuffer, VertexBuffer},
    device::Device,
    VulkanObject,
};

use ash::{version::DeviceV1_0, vk};

pub struct Mesh {
    vertex_buffer: VertexBuffer,
    index_buffer: Option<IndexBuffer>,
}

impl Mesh {
    pub fn new(vertices: &[Vertex], indices: Option<&[u16]>, device: &Arc<Device>) -> Mesh {
        let vertex_buffer = VertexBuffer::new(vertices, device);
        let index_buffer = match indices {
            Some(indices) => Some(IndexBuffer::new(indices, device)),
            None => None,
        };

        Mesh { vertex_buffer, index_buffer }
    }

    pub fn render(&self, device: &Arc<Device>, command_buffer: &vk::CommandBuffer) {
        let vertex_buffers = [*self.vertex_buffer.vk()];
        let offsets = [0];
        unsafe {
            if let Some(index_buffer) = &self.index_buffer {
                device.vk().cmd_bind_vertex_buffers(*command_buffer, 0, &vertex_buffers, &offsets);
                device.vk().cmd_bind_index_buffer(*command_buffer, *index_buffer.vk(), 0, vk::IndexType::UINT16);
                device.vk().cmd_draw_indexed(*command_buffer, index_buffer.index_count(), 1, 0, 0, 0);
            } else {
                device.vk().cmd_bind_vertex_buffers(*command_buffer, 0, &vertex_buffers, &offsets);
                device.vk().cmd_draw(*command_buffer, self.vertex_buffer.vertex_count(), 1, 0, 0);
            }
        }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        trace!("Dropping Mesh");
    }
}

pub struct MeshFactory {
    device: Arc<Device>,
}

impl MeshFactory {
    pub fn new(device: Arc<Device>) -> MeshFactory {
        MeshFactory { device }
    }

    pub fn create_mesh(&self, vertices: &[Vertex], indices: Option<&[u16]>) -> Mesh {
        Mesh::new(vertices, indices, &self.device)
    }
}
