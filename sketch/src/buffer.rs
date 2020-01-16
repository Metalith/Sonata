use crate::PhysicalDevice;
use crate::Renderer;
use crate::Vertex;
use crate::VulkanObject;

use ash::{version::DeviceV1_0, vk, Device};

pub struct Buffer {
    buffer: vk::Buffer,
    buffer_memory: vk::DeviceMemory,
}

impl Buffer {
    pub fn new(vertices: &[Vertex], device: &Device, physical_device: &PhysicalDevice) -> Self {
        let info = vk::BufferCreateInfo::builder()
            .size(std::mem::size_of_val(vertices) as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe { device.create_buffer(&info, None).unwrap() };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Buffer::find_memory_type(
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                physical_device.get_mem_properties(),
            ))
            .build();

        let buffer_memory = unsafe { device.allocate_memory(&alloc_info, None).unwrap() };

        unsafe {
            device.bind_buffer_memory(buffer, buffer_memory, 0).unwrap();

            let data_ptr = device.map_memory(buffer_memory, 0, info.size, vk::MemoryMapFlags::empty()).unwrap();

            let mut align = ash::util::Align::new(data_ptr, std::mem::align_of::<u32>() as _, mem_requirements.size);
            align.copy_from_slice(vertices);

            device.unmap_memory(buffer_memory);
        }

        Buffer {
            buffer: buffer,
            buffer_memory: buffer_memory,
        }
    }

    fn find_memory_type(type_filter: u32, properties: vk::MemoryPropertyFlags, physical_mem_properties: &vk::PhysicalDeviceMemoryProperties) -> u32 {
        for i in 0..physical_mem_properties.memory_type_count {
            if (type_filter & (1 << i)) > 0 && (physical_mem_properties.memory_types[i as usize].property_flags & properties) == properties {
                return i;
            }
        }

        panic!("Failed to find suitable memory");
    }
}

impl VulkanObject for Buffer {
    type Object = vk::Buffer;

    fn vulkan_object(&self) -> &Self::Object {
        &self.buffer
    }

    fn cleanup(&self, _renderer: &Renderer) {
        unsafe {
            _renderer.get_device().destroy_buffer(self.buffer, None);
            _renderer.get_device().free_memory(self.buffer_memory, None);
        }
    }
}
