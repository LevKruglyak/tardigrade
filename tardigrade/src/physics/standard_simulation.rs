use std::sync::Arc;

use hatchery::util::{
    buffer::{AbstractBuffer, SharedBuffer},
    compute::ComputeShader,
    ConstructionContext,
};
use vulkano::{
    buffer::BufferUsage, descriptor_set::WriteDescriptorSet, device::Device, shader::ShaderModule,
};

use super::{Particle, ParticlePosition, ParticleVelocityMass};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/physics/standard_simulation.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

// m v^2 = kg m^2 / s^2
// F kg m / s^2 = G kg^2 / m^2
// m^3 / s^2 kg = G
// PE = -Gm_1m^2/r

pub struct SimulationShader {
    position_mass: SharedBuffer<ParticlePosition>,
    potential: SharedBuffer<f32>,
    velocity: SharedBuffer<ParticleVelocityMass>,
    pub kinetic_energy: Vec<f32>,
    pub potential_energy: Vec<f32>,
    num_particles: u64,
}

impl ComputeShader for SimulationShader {
    type Constants = cs::ty::SimulationData;

    fn push_constants(&self) -> Option<Self::Constants> {
        Some(Self::Constants {
            buffer_size: self.num_particles as u32,
            dust_max: self.num_particles as u32,
        })
    }

    fn entry_point() -> &'static str {
        "main"
    }

    fn load_module(device: Arc<Device>) -> Arc<ShaderModule> {
        cs::load(device).unwrap()
    }

    fn dispatch_size(&self) -> [u32; 3] {
        [self.num_particles as u32 / 128 + 1, 1, 1]
    }

    fn write_descriptors(&self) -> Vec<WriteDescriptorSet> {
        vec![
            WriteDescriptorSet::buffer(0, self.position_mass.buffer()),
            WriteDescriptorSet::buffer(1, self.velocity.buffer()),
            WriteDescriptorSet::buffer(2, self.potential.buffer()),
        ]
    }
}

impl SimulationShader {
    pub fn new(context: &ConstructionContext, particles: Vec<Particle>) -> Self {
        Self {
            position_mass: SharedBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticlePosition {
                        p_pos: [p.position.x, p.position.y, p.position.z, p.mass],
                    })
                    .collect(),
            ),
            potential: SharedBuffer::new(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles.len() as u64,
            ),
            velocity: SharedBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticleVelocityMass {
                        p_vel_mass: [p.velocity.x, p.velocity.y, p.velocity.z, p.mass],
                    })
                    .collect(),
            ),
            num_particles: particles.len() as u64,
            kinetic_energy: Vec::new(),
            potential_energy: Vec::new(),
        }
    }

    pub fn particles(&self) -> &SharedBuffer<ParticlePosition> {
        &self.position_mass
    }

    pub fn velocity_mass(&self) -> &SharedBuffer<ParticleVelocityMass> {
        &self.velocity
    }

    pub fn calculate_energy(&mut self) {
        let kinetic_energy: f32 = self
            .velocity_mass()
            .typed_buffer()
            .read()
            .unwrap()
            .iter()
            .map(|vm| {
                0.5 * vm.p_vel_mass[3]
                    * (vm.p_vel_mass[0] * vm.p_vel_mass[0]
                        + vm.p_vel_mass[1] * vm.p_vel_mass[1]
                        + vm.p_vel_mass[2] * vm.p_vel_mass[2])
            })
            .sum();
        self.kinetic_energy.push(kinetic_energy);
        let potential_energy: f32 = self.potential.typed_buffer().read().unwrap().iter().sum();
        self.potential_energy.push(potential_energy);

        println!("total {}", kinetic_energy + potential_energy);
    }
}
