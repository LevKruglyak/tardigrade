use std::sync::Arc;

use bytemuck::Pod;
use vulkano::{
    buffer::{
        BufferAccess, BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer, TypedBufferAccess,
    },
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo},
    sync::{self, GpuFuture},
};

use super::ConstructionContext;

// Usages

// Trait alias workaround
pub trait BufferData: Pod + Send + Sync {}
impl<T> BufferData for T where T: Pod + Send + Sync {}

#[allow(clippy::len_without_is_empty)]
pub trait AbstractBuffer<T: BufferData> {
    type Buffer: TypedBufferAccess<Content = [T]>;

    /// Create an empty buffer with specified length
    fn new(context: &ConstructionContext, usage: BufferUsage, len: u64) -> Self;

    /// Create a buffer from a vector of data
    fn from_iter<I>(context: &ConstructionContext, usage: BufferUsage, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator;

    fn buffer(&self) -> Arc<dyn BufferAccess>;

    fn typed_buffer(&self) -> Arc<Self::Buffer>;

    fn copy<B: AbstractBuffer<T>>(&self, context: &ConstructionContext, src: &B) {
        let mut cb_builder = AutoCommandBufferBuilder::primary(
            context.command_allocator(),
            context.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        cb_builder
            .copy_buffer(CopyBufferInfo::buffers(src.buffer(), self.buffer()))
            .unwrap();

        let cb = cb_builder.build().unwrap();
        let future = sync::now(context.device())
            .then_execute(context.queue(), cb)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }

    /// Length of the buffer
    fn len(&self) -> u32;
}

/// Buffer that is accessible only from the GPU
pub struct DeviceBuffer<T: BufferData> {
    buffer: Arc<DeviceLocalBuffer<[T]>>,
}

impl<T: BufferData> AbstractBuffer<T> for DeviceBuffer<T> {
    type Buffer = DeviceLocalBuffer<[T]>;

    fn new(context: &ConstructionContext, usage: BufferUsage, len: u64) -> Self {
        Self {
            buffer: DeviceLocalBuffer::array(
                context.memory_allocator(),
                len,
                usage,
                context.queue_family_indices(),
            )
            .unwrap(),
        }
    }

    fn from_iter<I>(context: &ConstructionContext, usage: BufferUsage, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        // Create temporary shared buffer
        let temp = SharedBuffer::from_iter(
            context,
            BufferUsage {
                transfer_src: true,
                ..BufferUsage::empty()
            },
            data,
        );

        let result = DeviceBuffer::new(
            context,
            usage.union(&BufferUsage {
                transfer_dst: true,
                ..BufferUsage::empty()
            }),
            temp.len() as u64,
        );
        result.copy(context, &temp);

        result
    }

    fn len(&self) -> u32 {
        self.buffer.len() as u32
    }

    fn buffer(&self) -> Arc<dyn BufferAccess> {
        self.buffer.clone()
    }

    fn typed_buffer(&self) -> Arc<Self::Buffer> {
        self.buffer.clone()
    }
}

/// Buffer that is accessible by both GPU and CPU
pub struct SharedBuffer<T: BufferData> {
    buffer: Arc<CpuAccessibleBuffer<[T]>>,
}

impl<T: BufferData> AbstractBuffer<T> for SharedBuffer<T> {
    type Buffer = CpuAccessibleBuffer<[T]>;

    fn new(context: &ConstructionContext, usage: BufferUsage, len: u64) -> Self {
        Self {
            // Sort of unecessary, could just call `from_vec` with empty array
            buffer: unsafe {
                CpuAccessibleBuffer::uninitialized_array(
                    context.memory_allocator(),
                    len,
                    usage,
                    false,
                )
                .unwrap()
            },
        }
    }

    fn from_iter<I>(context: &ConstructionContext, usage: BufferUsage, data: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        Self {
            buffer: CpuAccessibleBuffer::from_iter(context.memory_allocator(), usage, false, data)
                .unwrap(),
        }
    }

    fn len(&self) -> u32 {
        self.buffer.len() as u32
    }

    fn buffer(&self) -> Arc<dyn BufferAccess> {
        self.buffer.clone()
    }

    fn typed_buffer(&self) -> Arc<Self::Buffer> {
        self.buffer.clone()
    }
}
