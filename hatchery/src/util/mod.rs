use std::sync::Arc;

use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{Device, Queue},
    memory::allocator::{MemoryAllocator, StandardMemoryAllocator},
};

pub mod buffer;
pub mod quad;

pub struct ConstructionContext {
    memory_allocator: StandardMemoryAllocator,
    command_allocator: StandardCommandBufferAllocator,
    descriptor_allocator: StandardDescriptorSetAllocator,
    queue: Arc<Queue>,
    device: Arc<Device>,
}

impl ConstructionContext {
    pub fn new(queue: Arc<Queue>) -> Self {
        Self {
            memory_allocator: StandardMemoryAllocator::new_default(queue.device().clone()),
            command_allocator: StandardCommandBufferAllocator::new(
                queue.device().clone(),
                Default::default(),
            ),
            descriptor_allocator: StandardDescriptorSetAllocator::new(queue.device().clone()),
            queue: queue.clone(),
            device: queue.device().clone(),
        }
    }

    pub fn memory_allocator(&self) -> &impl MemoryAllocator {
        &self.memory_allocator
    }

    pub fn command_allocator(&self) -> &StandardCommandBufferAllocator {
        &self.command_allocator
    }

    pub fn descriptor_allocator(&self) -> &StandardDescriptorSetAllocator {
        &self.descriptor_allocator
    }

    pub fn family(&self) -> u32 {
        self.queue.queue_family_index()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn queue_family_indices(&self) -> Vec<u32> {
        self.device.active_queue_family_indices().to_vec()
    }
}
