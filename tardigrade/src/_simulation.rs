use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use cgmath::Vector3;
use rand::Rng;
use rand_distr::UnitBall;
use tardigrade_launcher::vulkano;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer},
    command_buffer::{
        allocator::{CommandBufferAllocator, StandardCommandBufferAllocator},
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::{Device, Queue},
    memory::allocator::StandardMemoryAllocator,
    pipeline::graphics::{
        input_assembly::InputAssemblyState,
        vertex_input::{BuffersDefinition, Vertex},
    },
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod, Vertex)]
pub struct ParticleVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 4],
}

#[allow(clippy::needless_question_mark)]
mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/simulation.glsl"
    }
}

pub struct Simulation {
    pipeline: Arc<ComputePipeline>,
    positions: Arc<DeviceLocalBuffer<[ParticleVertex]>>,
    velocities: Arc<DeviceLocalBuffer<[ParticleVertex]>>,
    queue: Arc<Queue>,
    num_particles: usize,
}

fn create_temp_buffer(
    allocator: &StandardMemoryAllocator,
    device: Arc<Device>,
    data: Vec<ParticleVertex>,
) -> Arc<CpuAccessibleBuffer<[ParticleVertex]>> {
    CpuAccessibleBuffer::from_iter(
        allocator,
        BufferUsage {
            transfer_src: true,
            ..BufferUsage::empty()
        },
        false,
        data,
    )
    .unwrap()
}

fn create_local_buffer(
    allocator: &StandardMemoryAllocator,
    device: Arc<Device>,
    size: usize,
    vertex: bool,
) -> Arc<DeviceLocalBuffer<[ParticleVertex]>> {
    DeviceLocalBuffer::<[ParticleVertex]>::array(
        allocator,
        size as vulkano::DeviceSize,
        BufferUsage {
            storage_buffer: true,
            vertex_buffer: vertex,
            transfer_dst: true,
            ..BufferUsage::empty()
        },
        device.active_queue_family_indices().into_iter(),
    )
    .unwrap()
}

fn copy_buffer(
    allocator: &StandardMemoryAllocator,
    queue: Arc<Queue>,
    source: Arc<CpuAccessibleBuffer<[ParticleVertex]>>,
    dest: Arc<DeviceLocalBuffer<[ParticleVertex]>>,
) {
    let mut cb_builder = AutoCommandBufferBuilder::primary(
        allocator,
        queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    cb_builder
        .copy_buffer(CopyBufferInfo::buffers(source, dest))
        .unwrap();

    let cb = cb_builder.build().unwrap();
    let future = sync::now(queue.device().clone())
        .then_execute(queue.clone(), cb)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();
}

impl Simulation {
    pub fn new(
        allocator: &StandardMemoryAllocator,
        queue: Arc<Queue>,
        num_particles: usize,
    ) -> Self {
        // Generate data
        let mut rng = rand::thread_rng();
        let position_data: Vec<ParticleVertex> = (0..num_particles)
            .map(|_| Vector3::from(rng.sample(UnitBall)) * 60.0)
            .map(|v: Vector3<f32>| ParticleVertex {
                position: [v.x, v.y, v.z, 1.0],
            })
            .collect();

        let velocity_data: Vec<ParticleVertex> = (0..num_particles)
            .map(|_| Vector3::from(rng.sample(UnitBall)) * 0.0001)
            .map(|v: Vector3<f32>| ParticleVertex {
                position: [v.x, v.y, v.z, 1.0],
            })
            .collect();

        // Create temporary CPU accessible buffers
        let positions = {
            let temp_positions =
                create_temp_buffer(allocator, queue.device().clone(), position_data);
            let positions =
                create_local_buffer(allocator, queue.device().clone(), num_particles, true);
            copy_buffer(allocator, queue.clone(), temp_positions, positions.clone());
            positions
        };

        let velocities = {
            let temp_velocities =
                create_temp_buffer(allocator, queue.device().clone(), velocity_data);
            let velocities =
                create_local_buffer(allocator, queue.device().clone(), num_particles, true);
            copy_buffer(
                allocator,
                queue.clone(),
                temp_velocities,
                velocities.clone(),
            );
            velocities
        };

        let cs = cs::load(queue.device().clone()).expect("failed to create shader module");

        let pipeline = ComputePipeline::new(
            queue.device().clone(),
            cs.entry_point("main").unwrap(),
            &(),
            None,
            |_| {},
        )
        .expect("failed to create compute pipeline");

        Self {
            pipeline,
            queue,
            positions,
            velocities,
            num_particles,
        }
    }

    pub fn advance(&self, allocator: &StandardCommandBufferAllocator) {
        let mut builder = AutoCommandBufferBuilder::primary(
            allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            allocator,
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, self.positions.clone()),
                WriteDescriptorSet::buffer(1, self.velocities.clone()),
            ],
        )
        .unwrap();

        let data = cs::ty::SimulationData {
            buffer_size: self.num_particles as u32,
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
            .dispatch([self.num_particles as u32 / 64, 1, 1])
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.queue.device().clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }

    pub fn positions(&self) -> Arc<DeviceLocalBuffer<[ParticleVertex]>> {
        self.positions.clone()
    }
}