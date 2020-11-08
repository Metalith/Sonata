use std::{
    cell::Cell,
    sync::{Arc, Mutex, Weak},
};

use crate::{
    buffers::{UniformBufferObject, UniformTestObject},
    device::Device,
    VulkanObject,
};

use ash::{version::DeviceV1_0, vk};

use super::DescriptorLayout;

struct Pool {
    device: Arc<Device>,
    descriptor_pool: vk::DescriptorPool,
    remaining_descriptor_count: Cell<u32>,
    remaining_sets_count: Cell<u32>,
}

impl Pool {
    pub fn new(device: Arc<Device>, descriptor_count: u32, set_count: u32) -> Arc<Pool> {
        trace!("Creating Descriptor Pool");
        let pool_sizes = [vk::DescriptorPoolSize::builder().descriptor_count(descriptor_count).build()];

        let pool_info = vk::DescriptorPoolCreateInfo::builder().pool_sizes(&pool_sizes).max_sets(set_count);

        let descriptor_pool = unsafe { device.vk().create_descriptor_pool(&pool_info, None).unwrap() };

        Pool {
            device,
            descriptor_pool,
            remaining_descriptor_count: Cell::new(descriptor_count),
            remaining_sets_count: Cell::new(set_count),
        }
        .into()
    }

    // TODO: Return success/fail
    pub fn alloc(&self, descriptor_layouts: &[Arc<DescriptorLayout>]) -> Vec<vk::DescriptorSet> {
        trace!("Alloc Descriptor Set");
        let layouts = descriptor_layouts.iter().map(|layout| *layout.vk()).collect::<Vec<_>>();
        let alloc_info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(self.descriptor_pool).set_layouts(&layouts).build();

        unsafe { self.device.vk().allocate_descriptor_sets(&alloc_info).unwrap() }
    }
}

impl VulkanObject for Pool {
    type Object = vk::DescriptorPool;

    fn vk(&self) -> &Self::Object {
        &self.descriptor_pool
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        trace!("Dropping Descriptor Pool");
        unsafe {
            self.device.vk().destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

pub struct DescriptorPool {
    device: Arc<Device>,
    pools: Mutex<Vec<Weak<Pool>>>,
}

impl DescriptorPool {
    pub fn new(device: Arc<Device>) -> DescriptorPool {
        trace!("Creating Descriptor Pools");
        DescriptorPool {
            device,
            pools: Mutex::new(Vec::new()),
        }
    }

    pub fn alloc(self: &mut Arc<Self>, descriptor_layouts: &[Arc<DescriptorLayout>]) -> Arc<DescriptorPoolAlloc> {
        let mut pools = self.pools.lock().unwrap();

        for pool_arc in pools.iter_mut() {
            if let Some(pool) = pool_arc.upgrade() {
                if pool.remaining_sets_count.get() == 0 {
                    continue;
                }

                // TODO: Add a descriptor count here
                if pool.remaining_descriptor_count.get() < descriptor_layouts.len() as u32 {
                    continue;
                }

                pool.remaining_sets_count.set(pool.remaining_sets_count.get() - descriptor_layouts.len() as u32);
                pool.remaining_descriptor_count.set(pool.remaining_descriptor_count.get() - descriptor_layouts.len() as u32);

                return DescriptorPoolAlloc {
                    _pool_parent: self.clone(),
                    pool: pool.clone(),
                    sets: pool.alloc(descriptor_layouts),
                }
                .into();
            }
        }

        pools.retain(|x| Option::is_some(&x.upgrade()));

        let pool = Pool::new(self.device.clone(), 40, 40);
        pools.push(Arc::downgrade(&pool));

        pool.remaining_sets_count.set(pool.remaining_sets_count.get() - descriptor_layouts.len() as u32);
        pool.remaining_descriptor_count.set(pool.remaining_descriptor_count.get() - descriptor_layouts.len() as u32);

        DescriptorPoolAlloc {
            _pool_parent: self.clone(),
            pool: pool.clone(),
            sets: pool.alloc(descriptor_layouts),
        }
        .into()
    }
}

pub struct DescriptorPoolAlloc {
    _pool_parent: Arc<DescriptorPool>,
    pool: Arc<Pool>,
    sets: Vec<vk::DescriptorSet>,
}

impl DescriptorPoolAlloc {
    //TODO: Remove this
    pub fn update(&self, ubos: &[UniformBufferObject]) {
        self.sets.iter().zip(ubos.iter()).for_each(|(set, buffer)| {
            let buffer_info = vk::DescriptorBufferInfo::builder()
                .buffer(*buffer.vk())
                .offset(0)
                .range(UniformTestObject::get_size() as vk::DeviceSize)
                .build();

            let buffer_infos = [buffer_info];

            let descriptor_write = vk::WriteDescriptorSet::builder()
                .dst_set(*set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build();
            let descriptor_writes = [descriptor_write];

            let null = [];

            unsafe { self.pool.device.vk().update_descriptor_sets(&descriptor_writes, &null) };
        });
    }
}

impl VulkanObject for DescriptorPoolAlloc {
    type Object = Vec<vk::DescriptorSet>;

    fn vk(&self) -> &Self::Object {
        &self.sets
    }
}

impl Drop for DescriptorPoolAlloc {
    fn drop(&mut self) {
        trace!("Dropping Descriptor Pool Alloc");
    }
}
