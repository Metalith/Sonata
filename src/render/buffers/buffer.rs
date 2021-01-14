use std::sync::Arc;

use ash::{version::DeviceV1_0, vk};

use crate::render::{device::Device, VulkanObject};

pub struct Buffer {
    device: Arc<Device>,
    buffer: vk::Buffer,
    buffer_memory: vk::DeviceMemory,
}

impl Buffer {
    pub fn new(size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags, device: Arc<Device>) -> Self {
        let info = vk::BufferCreateInfo::builder().size(size).usage(usage).sharing_mode(vk::SharingMode::EXCLUSIVE).build();

        let buffer = unsafe { device.vk().create_buffer(&info, None).unwrap() };

        let mem_requirements = unsafe { device.vk().get_buffer_memory_requirements(buffer) };

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Buffer::find_memory_type(mem_requirements.memory_type_bits, properties, device.physical_device().get_mem_properties()))
            .build();

        let buffer_memory = unsafe { device.vk().allocate_memory(&alloc_info, None).unwrap() };

        unsafe {
            device.vk().bind_buffer_memory(buffer, buffer_memory, 0).unwrap();
        }

        Buffer { device, buffer, buffer_memory }
    }

    fn find_memory_type(type_filter: u32, properties: vk::MemoryPropertyFlags, physical_mem_properties: &vk::PhysicalDeviceMemoryProperties) -> u32 {
        for i in 0..physical_mem_properties.memory_type_count {
            if (type_filter & (1 << i)) > 0 && (physical_mem_properties.memory_types[i as usize].property_flags & properties) == properties {
                return i;
            }
        }

        panic!("Failed to find suitable memory");
    }

    pub fn map_memory<A, T: Copy>(&self, object: &[T]) {
        #[allow(clippy::useless_conversion)]
        let size: vk::DeviceSize = vk::DeviceSize::from(std::mem::size_of_val(object) as u64);
        unsafe {
            let data_ptr = self.device.vk().map_memory(self.buffer_memory, 0, size, vk::MemoryMapFlags::empty()).unwrap();

            let mut align = ash::util::Align::new(data_ptr, std::mem::align_of::<A>() as _, size);
            align.copy_from_slice(object);

            self.device.vk().unmap_memory(self.buffer_memory);
        }
    }

    //TODO: Move this to some sort of common command buffer commands class
    pub fn copy_buffer(src: &Buffer, dst: &Buffer, size: vk::DeviceSize, device: &Arc<Device>) {
        let command_pool = device.command_pool();
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*command_pool.vk())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1)
            .build();

        let command_buffers = unsafe { device.vk().allocate_command_buffers(&alloc_info).unwrap() };

        let begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT).build();
        let copy_region = vk::BufferCopy::builder().size(size).build();
        let submit_info = vk::SubmitInfo::builder().command_buffers(&command_buffers).build();

        unsafe {
            device.vk().begin_command_buffer(command_buffers[0], &begin_info).unwrap();
            device.vk().cmd_copy_buffer(command_buffers[0], *src.vk(), *dst.vk(), &[copy_region]);
            device.vk().end_command_buffer(command_buffers[0]).unwrap();
            device.vk().queue_submit(*device.graphics_queue(), &[submit_info], vk::Fence::null()).unwrap();
            device.vk().device_wait_idle().unwrap();
        }
    }
}

impl VulkanObject for Buffer {
    type Object = vk::Buffer;

    fn vk(&self) -> &Self::Object {
        &self.buffer
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        trace!("Dropping Buffer");
        unsafe {
            self.device.vk().device_wait_idle();
            self.device.vk().destroy_buffer(self.buffer, None);
            self.device.vk().free_memory(self.buffer_memory, None);
        }
    }
}
