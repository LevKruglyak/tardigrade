use std::sync::Arc;

use vulkano::{
    buffer::BufferContents,
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    shader::ShaderModule,
    sync::{self, GpuFuture},
};

use super::ConstructionContext;

pub struct ComputeShaderExecutor<G: ComputeShader> {
    module: Arc<ShaderModule>,
    pipeline: Arc<ComputePipeline>,
    descriptor_set: Arc<PersistentDescriptorSet>,
    shader: G,
}

impl<G: ComputeShader> ComputeShaderExecutor<G> {
    pub fn new(context: &ConstructionContext, module: Arc<ShaderModule>, shader: G) -> Self {
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
            G::write_descriptors(),
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
        let future = sync::now(context.device())
            .then_execute(context.queue(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }
}

pub trait ComputeShader {
    fn entry_point() -> &'static str;

    fn dispatch_size(&self) -> [u32; 3];

    fn write_descriptors() -> Vec<WriteDescriptorSet>;

    type Constants: BufferContents;
    fn push_constants(&self) -> Option<Self::Constants> {
        None
    }
}
