use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use cgmath::Vector3;
use hatchery::{
    util::{
        buffer::{AbstractBuffer, DeviceBuffer},
        ConstructionContext,
    },
    AutoCommandBufferBuilder,
};
use vulkano::{
    buffer::BufferUsage,
    command_buffer::CommandBufferUsage,
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    impl_vertex,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};

pub struct Particle {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    mass: f32,
}

impl Particle {
    pub fn new(position: Vector3<f32>, velocity: Vector3<f32>, mass: f32) -> Self {
        Self {
            position,
            velocity,
            mass,
        }
    }
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct ParticlePosition {
    p_pos: [f32; 4],
}

impl_vertex!(ParticlePosition, p_pos);

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct ParticleVelocityMass {
    p_vel_mass: [f32; 4],
}

impl_vertex!(ParticleVelocityMass, p_vel_mass);

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/physics/simulation.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

pub struct Simulation {
    pipeline: Arc<ComputePipeline>,
    position_mass: DeviceBuffer<ParticlePosition>,
    velocity: DeviceBuffer<ParticleVelocityMass>,
    num_particles: u64,
}

impl Simulation {
    pub fn new(context: &ConstructionContext, particles: Vec<Particle>) -> Self {
        let cs = cs::load(context.device()).unwrap();
        let pipeline = ComputePipeline::new(
            context.device(),
            cs.entry_point("main").unwrap(),
            &(),
            None,
            |_| {},
        )
        .unwrap();

        Self {
            position_mass: DeviceBuffer::from_vec(
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
            velocity: DeviceBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticleVelocityMass {
                        p_vel_mass: [p.velocity.x, p.velocity.y, p.velocity.z, 0.0],
                    })
                    .collect(),
            ),
            num_particles: particles.len() as u64,
            pipeline,
        }
    }

    pub fn advance(&self, context: &ConstructionContext) {
        let mut builder = AutoCommandBufferBuilder::primary(
            context.command_allocator(),
            context.queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            context.descriptor_allocator(),
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, self.position_mass.buffer()),
                WriteDescriptorSet::buffer(1, self.velocity.buffer()),
            ],
        )
        .unwrap();

        let data = cs::ty::SimulationData {
            buffer_size: self.num_particles as u32,
            dust_max: self.num_particles as u32 / 500,
        };

        builder
            .bind_pipeline_compute(self.pipeline.clone())
            .push_constants(self.pipeline.layout().clone(), 0, data)
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pipeline.layout().clone(),
                0,
                set,
            )
            .dispatch([self.num_particles as u32 / 64 + 1, 1, 1])
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = sync::now(context.device())
            .then_execute(context.queue(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }

    pub fn particles(&self) -> &DeviceBuffer<ParticlePosition> {
        &self.position_mass
    }

    pub fn velocity_mass(&self) -> &DeviceBuffer<ParticleVelocityMass> {
        &self.velocity
    }
}
