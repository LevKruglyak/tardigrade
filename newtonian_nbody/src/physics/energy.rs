use std::sync::Arc;

use hatchery::util::{
    buffer::{AbstractBuffer, SharedBuffer},
    compute::ComputeShader,
    ConstructionContext,
};
use vulkano::{
    buffer::BufferUsage, descriptor_set::WriteDescriptorSet, device::Device, shader::ShaderModule,
};

use super::SimulationBuffers;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/physics/energy.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

pub struct EnergyCalculator {
    data: Arc<SimulationBuffers>,
    energy: SharedBuffer<f32>,
}

impl EnergyCalculator {
    pub fn new(data: Arc<SimulationBuffers>, context: &ConstructionContext) -> Self {
        let num_particles = data.num_particles;
        Self {
            data,
            energy: SharedBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                vec![0.0; num_particles as usize],
            ),
        }
    }

    pub fn get_total_energy(&self) -> f32 {
        self.energy.typed_buffer().read().unwrap().iter().sum()
    }
}

impl ComputeShader for EnergyCalculator {
    type Constants = cs::ty::SimulationData;

    fn push_constants(&self) -> Option<Self::Constants> {
        Some(Self::Constants {
            buffer_size: self.data.num_particles,
        })
    }

    fn entry_point() -> &'static str {
        "main"
    }

    fn load_module(device: Arc<Device>) -> Arc<ShaderModule> {
        cs::load(device).unwrap()
    }

    fn dispatch_size(&self) -> [u32; 3] {
        [self.data.num_particles.div_ceil(128), 1, 1]
    }

    fn write_descriptors(&self) -> Vec<WriteDescriptorSet> {
        vec![
            WriteDescriptorSet::buffer(0, self.data.position_mass.buffer()),
            WriteDescriptorSet::buffer(1, self.data.velocity.buffer()),
            WriteDescriptorSet::buffer(2, self.data.acceleration.buffer()),
            WriteDescriptorSet::buffer(3, self.energy.buffer()),
        ]
    }
}
