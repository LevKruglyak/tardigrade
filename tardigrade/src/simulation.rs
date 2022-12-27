use std::sync::Arc;

use cgmath::Vector3;
use rand::Rng;
use rand_distr::UnitBall;
use tardigrade_launcher::vulkano;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::{Device, Queue},
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};

#[allow(clippy::needless_question_mark)]
mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/simulation.glsl"
    }
}

pub struct Simulation {
    pipeline: Arc<ComputePipeline>,
    positions: Arc<DeviceLocalBuffer<[[f32; 4]]>>,
    velocities: Arc<DeviceLocalBuffer<[[f32; 4]]>>,
    queue: Arc<Queue>,
    num_particles: usize,
}

fn create_temp_buffer(
    device: Arc<Device>,
    data: Vec<[f32; 4]>,
) -> Arc<CpuAccessibleBuffer<[[f32; 4]]>> {
    CpuAccessibleBuffer::from_iter(
        device,
        BufferUsage {
            transfer_src: true,
            ..BufferUsage::none()
        },
        false,
        data,
    )
    .unwrap()
}

fn create_local_buffer(
    device: Arc<Device>,
    size: usize,
    vertex: bool,
) -> Arc<DeviceLocalBuffer<[[f32; 4]]>> {
    DeviceLocalBuffer::<[[f32; 4]]>::array(
        device.clone(),
        size as vulkano::DeviceSize,
        BufferUsage {
            storage_buffer: true,
            vertex_buffer: vertex,
            transfer_dst: true,
            ..BufferUsage::none()
        },
        device.active_queue_families(),
    )
    .unwrap()
}

fn copy_buffer(
    queue: Arc<Queue>,
    source: Arc<CpuAccessibleBuffer<[[f32; 4]]>>,
    dest: Arc<DeviceLocalBuffer<[[f32; 4]]>>,
) {
    let mut cb_builder = AutoCommandBufferBuilder::primary(
        queue.device().clone(),
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
    pub fn new(queue: Arc<Queue>, num_particles: usize) -> Self {
        // Generate data
        let mut rng = rand::thread_rng();
        let position_data: Vec<[f32; 4]> = (0..num_particles)
            .map(|_| Vector3::from(rng.sample(UnitBall)) * 60.0)
            .map(|v: Vector3<f32>| [v.x, v.y, v.z, 1.0])
            .collect();

        let velocity_data: Vec<[f32; 4]> = (0..num_particles)
            .map(|_| Vector3::from(rng.sample(UnitBall)) * 0.0001)
            .map(|v: Vector3<f32>| [v.x, v.y, v.z, 1.0])
            .collect();

        // Create temporary CPU accessible buffers
        let positions = {
            let temp_positions = create_temp_buffer(queue.device().clone(), position_data);
            let positions = create_local_buffer(queue.device().clone(), num_particles, true);
            copy_buffer(queue.clone(), temp_positions, positions.clone());
            positions
        };

        let velocities = {
            let temp_velocities = create_temp_buffer(queue.device().clone(), velocity_data);
            let velocities = create_local_buffer(queue.device().clone(), num_particles, true);
            copy_buffer(queue.clone(), temp_velocities, velocities.clone());
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

    pub fn advance(&self) {
        let mut builder = AutoCommandBufferBuilder::primary(
            self.queue.device().clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
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

    pub fn positions(&self) -> Arc<DeviceLocalBuffer<[[f32; 4]]>> {
        self.positions.clone()
    }
}
