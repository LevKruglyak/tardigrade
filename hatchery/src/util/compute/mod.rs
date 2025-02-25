use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use vulkano::{
    buffer::BufferContents,
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::Device,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    shader::ShaderModule,
    sync::{self, GpuFuture},
};

use super::ConstructionContext;

// TODO: multiple invocations

pub struct ComputeShaderExecutor<G: ComputeShader> {
    module: Arc<ShaderModule>,
    pipeline: Arc<ComputePipeline>,
    descriptor_set: Arc<PersistentDescriptorSet>,
    shader: G,
}

impl<G: ComputeShader> ComputeShaderExecutor<G> {
    pub fn new(context: &ConstructionContext, shader: G) -> Self {
        let module = G::load_module(context.device());
        let pipeline = ComputePipeline::new(
            context.device(),
            module.entry_point(G::entry_point()).unwrap(),
            &(),
            None,
            |_| {},
        )
        .unwrap();

        let layout = pipeline.layout().set_layouts().get(0).unwrap();
        let descriptor_set = PersistentDescriptorSet::new(
            context.descriptor_allocator(),
            layout.clone(),
            shader.write_descriptors(),
        )
        .unwrap();

        Self {
            module,
            pipeline,
            descriptor_set,
            shader,
        }
    }

    pub fn execute(&self, context: &ConstructionContext) {
        let mut builder = AutoCommandBufferBuilder::primary(
            context.command_allocator(),
            context.queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let constants = self.shader.push_constants();

        builder
            .bind_pipeline_compute(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pipeline.layout().clone(),
                0,
                self.descriptor_set.clone(),
            );

        if let Some(constants) = constants {
            builder.push_constants(self.pipeline.layout().clone(), 0, constants);
        }

        builder.dispatch(self.shader.dispatch_size()).unwrap();

        let command_buffer = builder.build().unwrap();
        sync::now(context.device())
            .then_execute(context.queue(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
    }
}

impl<G: ComputeShader> Deref for ComputeShaderExecutor<G> {
    type Target = G;

    fn deref(&self) -> &Self::Target {
        &self.shader
    }
}

impl<G: ComputeShader> DerefMut for ComputeShaderExecutor<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shader
    }
}

pub trait ComputeShader {
    fn load_module(device: Arc<Device>) -> Arc<ShaderModule>;

    fn entry_point() -> &'static str {
        "main"
    }

    fn dispatch_size(&self) -> [u32; 3];

    fn write_descriptors(&self) -> Vec<WriteDescriptorSet>;

    type Constants: BufferContents;
    fn push_constants(&self) -> Option<Self::Constants> {
        None
    }
}
