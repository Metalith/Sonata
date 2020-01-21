use ash::{version::InstanceV1_0, vk, Instance};

pub struct QueueFamily {
    pub index: u32,
    pub count: u32,
    pub flags: vk::QueueFlags,
    pub priority: f32,
}

impl QueueFamily {
    pub fn all(instance: &Instance, device: vk::PhysicalDevice) -> Vec<QueueFamily> {
        let mut queue_families: Vec<QueueFamily> = Vec::new();

        let families = unsafe { instance.get_physical_device_queue_family_properties(device) };
        for i in 0..families.len() {
            if families[i].queue_count > 0 {
                queue_families.push(QueueFamily::new(i, families[i]))
            }
        }

        queue_families
    }

    fn new(i: usize, family: vk::QueueFamilyProperties) -> Self {
        QueueFamily {
            index: i as u32,
            count: family.queue_count,
            flags: family.queue_flags,
            priority: 1.0f32,
        }
    }
}
