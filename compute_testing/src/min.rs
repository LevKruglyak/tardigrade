use std::sync::Arc;

use hatchery::util::{
    buffer::{AbstractBuffer, SharedBuffer, DeviceBuffer},
    compute::ComputeShader,
    ConstructionContext,
};
use vulkano::{
    buffer::BufferUsage, descriptor_set::WriteDescriptorSet, device::Device, shader::ShaderModule,
};

use self::cs::ty::Constants;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/min.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

pub struct MinShader {
    data: DeviceBuffer<f32>,
    out: SharedBuffer<f32>,
}

impl ComputeShader for MinShader {
    type Constants = cs::ty::Constants;

    fn push_constants(&self) -> Option<Self::Constants> {
        Some(Constants {
            buffer_size: self.data.len(),
        })
    }

    fn entry_point() -> &'static str {
        "main"
    }

    fn load_module(device: Arc<Device>) -> Arc<ShaderModule> {
        cs::load(device).unwrap()
    }

    fn dispatch_size(&self) -> [u32; 3] {
        [1, 1, 1]
    }

    fn write_descriptors(&self) -> Vec<WriteDescriptorSet> {
        vec![
            WriteDescriptorSet::buffer(0, self.data.buffer()),
            WriteDescriptorSet::buffer(1, self.out.buffer()),
        ]
    }
}

impl MinShader {
    pub fn new(context: &ConstructionContext, points: Vec<f32>) -> Self {
        Self {
            data: DeviceBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                points,
            ),
            out: SharedBuffer::new(
                context,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                1,
            ),
        }
    }

    pub fn min(&self) -> f32 {
        self.out.typed_buffer().read().unwrap()[0]
    }
}
