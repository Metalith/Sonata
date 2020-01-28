use crate::GraphicContext;
use crate::VulkanObject;

use ash::{version::DeviceV1_0, vk};

pub struct Buffer {
    buffer: vk::Buffer,
    buffer_memory: vk::DeviceMemory,
}

impl Buffer {
    pub fn new(size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags, context: &GraphicContext) -> Self {
        let info = vk::BufferCreateInfo::builder().size(size).usage(usage).sharing_mode(vk::SharingMode::EXCLUSIVE).build();

        let buffer = unsafe { context.get_device().create_buffer(&info, None).unwrap() };

        let mem_requirements = unsafe { context.get_device().get_buffer_memory_requirements(buffer) };

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Buffer::find_memory_type(mem_requirements.memory_type_bits, properties, context.get_physical_device().get_mem_properties()))
            .build();

        let buffer_memory = unsafe { context.get_device().allocate_memory(&alloc_info, None).unwrap() };

        unsafe {
            context.get_device().bind_buffer_memory(buffer, buffer_memory, 0).unwrap();
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

    pub fn map_memory<T: Copy>(&self, vertices: &[T], context: &GraphicContext) {
        let size = vk::DeviceSize::from(std::mem::size_of_val(vertices) as u64);
        unsafe {
            let data_ptr = context.get_device().map_memory(self.buffer_memory, 0, size, vk::MemoryMapFlags::empty()).unwrap();

            let mut align = ash::util::Align::new(data_ptr, std::mem::align_of::<u32>() as _, size);
            align.copy_from_slice(vertices);

            context.get_device().unmap_memory(self.buffer_memory);
        }
    }

    pub fn copy_buffer(src: vk::Buffer, dst: vk::Buffer, size: vk::DeviceSize, context: &GraphicContext) {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*context.get_command_pool())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1)
            .build();

        let command_buffers = unsafe { context.get_device().allocate_command_buffers(&alloc_info).unwrap() };

        let begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT).build();
        let copy_region = vk::BufferCopy::builder().size(size).build();
        let submit_info = vk::SubmitInfo::builder().command_buffers(&command_buffers).build();

        unsafe {
            context.get_device().begin_command_buffer(command_buffers[0], &begin_info).unwrap();
            context.get_device().cmd_copy_buffer(command_buffers[0], src, dst, &[copy_region]);
            context.get_device().end_command_buffer(command_buffers[0]).unwrap();
            context.get_device().queue_submit(*context.get_logical_device().graphics_queue(), &[submit_info], vk::Fence::null()).unwrap();
            context.wait_device();
        }
    }
}

impl VulkanObject for Buffer {
    type Object = vk::Buffer;

    fn vulkan_object(&self) -> &Self::Object {
        &self.buffer
    }

    fn cleanup(&self, _context: &GraphicContext) {
        unsafe {
            _context.get_device().destroy_buffer(self.buffer, None);
            _context.get_device().free_memory(self.buffer_memory, None);
        }
    }
}
