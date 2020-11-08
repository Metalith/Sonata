use std::sync::Arc;

use super::Buffer;
use crate::{device::Device, VulkanObject};

use ash::vk;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct UniformTestObject {
    pub model: uv::Mat4,
    pub view: uv::Mat4,
    pub proj: uv::Mat4,
}

impl UniformTestObject {
    pub fn get_size() -> u64 {
        std::mem::size_of::<UniformTestObject>() as u64
    }
}
pub struct UniformBufferObject {
    buffer: Buffer,
}

impl UniformBufferObject {
    pub fn new(device: &Arc<Device>) -> UniformBufferObject {
        let buffer_size = UniformTestObject::get_size();
        let buffer = Buffer::new(
            buffer_size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device.clone(),
        );

        UniformBufferObject { buffer }
    }

    pub fn update2<A, T: Copy>(&self, object: &[T]) {
        self.buffer.map_memory::<f32, _>(object)
    }
}

impl VulkanObject for UniformBufferObject {
    type Object = vk::Buffer;

    fn vk(&self) -> &Self::Object {
        &self.buffer.vk()
    }
}

impl Drop for UniformBufferObject {
    fn drop(&mut self) {
        trace!("Dropping Uniform Buffer Object");
    }
}
