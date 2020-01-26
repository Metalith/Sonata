use super::Vertex;
use crate::buffers::Buffer;
use crate::device::PhysicalDevice;
use crate::GraphicContext;
use crate::VulkanObject;

use ash::{version::DeviceV1_0, vk, Device};

pub struct Model {
    vertices: Vec<Vertex>,
    vertex_buffer: Buffer,
}

impl Model {
    pub fn new(vertices: &[Vertex], device: &Device, physical_device: &PhysicalDevice) -> Model {
        let buffer = Buffer::new(vertices, device, &physical_device);

        Model {
            vertices: vertices.to_vec(),
            vertex_buffer: buffer,
        }
    }

    pub fn render(&self, device: &Device, command_buffer: &vk::CommandBuffer) {
        let buffers = [*self.vertex_buffer.vulkan_object()];
        let offsets = [0];
        unsafe {
            device.cmd_bind_vertex_buffers(*command_buffer, 0, &buffers, &offsets);
            device.cmd_draw(*command_buffer, self.vertices.len() as u32, 1, 0, 0);
        }
    }

    pub fn cleanup(&self, _context: &GraphicContext) {
        self.vertex_buffer.cleanup(_context);
    }
}
