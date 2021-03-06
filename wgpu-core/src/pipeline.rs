/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    binding_model::{CreateBindGroupLayoutError, CreatePipelineLayoutError},
    device::{DeviceError, RenderPassContext},
    id::{DeviceId, PipelineLayoutId, ShaderModuleId},
    validation::StageError,
    LifeGuard, RefCount, Stored,
};
use std::borrow::{Borrow, Cow};
use thiserror::Error;
use wgt::{BufferAddress, IndexFormat, InputStepMode};

#[repr(C)]
#[derive(Debug)]
pub enum ShaderModuleSource<'a> {
    SpirV(Cow<'a, [u32]>),
    Wgsl(Cow<'a, str>),
    Naga(naga::Module),
}

#[derive(Debug)]
pub struct ShaderModule<B: hal::Backend> {
    pub(crate) raw: B::ShaderModule,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) module: Option<naga::Module>,
}

#[derive(Clone, Debug, Error)]
pub enum CreateShaderModuleError {
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error(transparent)]
    Validation(#[from] naga::proc::ValidationError),
}

pub type ProgrammableStageDescriptor<'a> = wgt::ProgrammableStageDescriptor<'a, ShaderModuleId>;

/// Number of implicit bind groups derived at pipeline creation.
pub type ImplicitBindGroupCount = u8;

#[derive(Clone, Debug, Error)]
pub enum ImplicitLayoutError {
    #[error("missing IDs for deriving {0} bind groups")]
    MissingIds(ImplicitBindGroupCount),
    #[error("unable to reflect the shader {0:?} interface")]
    ReflectionError(wgt::ShaderStage),
    #[error(transparent)]
    BindGroup(#[from] CreateBindGroupLayoutError),
    #[error(transparent)]
    Pipeline(#[from] CreatePipelineLayoutError),
}

pub type ComputePipelineDescriptor<'a> =
    wgt::ComputePipelineDescriptor<PipelineLayoutId, ProgrammableStageDescriptor<'a>>;

#[derive(Clone, Debug, Error)]
pub enum CreateComputePipelineError {
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error("pipeline layout is invalid")]
    InvalidLayout,
    #[error("unable to derive an implicit layout")]
    Implicit(#[from] ImplicitLayoutError),
    #[error(transparent)]
    Stage(StageError),
}

#[derive(Debug)]
pub struct ComputePipeline<B: hal::Backend> {
    pub(crate) raw: B::ComputePipeline,
    pub(crate) layout_id: Stored<PipelineLayoutId>,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) life_guard: LifeGuard,
}

impl<B: hal::Backend> Borrow<RefCount> for ComputePipeline<B> {
    fn borrow(&self) -> &RefCount {
        self.life_guard.ref_count.as_ref().unwrap()
    }
}

pub type RenderPipelineDescriptor<'a> =
    wgt::RenderPipelineDescriptor<'a, PipelineLayoutId, ProgrammableStageDescriptor<'a>>;

#[derive(Clone, Debug, Error)]
pub enum CreateRenderPipelineError {
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error("pipelie layout is invalid")]
    InvalidLayout,
    #[error("unable to derive an implicit layout")]
    Implicit(#[from] ImplicitLayoutError),
    #[error("incompatible output format at index {index}")]
    IncompatibleOutputFormat { index: u8 },
    #[error("invalid sample count {0}")]
    InvalidSampleCount(u32),
    #[error("vertex buffer {index} stride {stride} does not respect `VERTEX_STRIDE_ALIGNMENT`")]
    UnalignedVertexStride { index: u32, stride: BufferAddress },
    #[error("vertex attribute at location {location} has invalid offset {offset}")]
    InvalidVertexAttributeOffset {
        location: wgt::ShaderLocation,
        offset: BufferAddress,
    },
    #[error("missing required device features {0:?}")]
    MissingFeature(wgt::Features),
    #[error("error in stage {flag:?}: {error}")]
    Stage {
        flag: wgt::ShaderStage,
        error: StageError,
    },
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PipelineFlags: u32 {
        const BLEND_COLOR = 1;
        const STENCIL_REFERENCE = 2;
        const WRITES_DEPTH_STENCIL = 4;
    }
}

#[derive(Debug)]
pub struct RenderPipeline<B: hal::Backend> {
    pub(crate) raw: B::GraphicsPipeline,
    pub(crate) layout_id: Stored<PipelineLayoutId>,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) pass_context: RenderPassContext,
    pub(crate) flags: PipelineFlags,
    pub(crate) index_format: IndexFormat,
    pub(crate) vertex_strides: Vec<(BufferAddress, InputStepMode)>,
    pub(crate) life_guard: LifeGuard,
}

impl<B: hal::Backend> Borrow<RefCount> for RenderPipeline<B> {
    fn borrow(&self) -> &RefCount {
        self.life_guard.ref_count.as_ref().unwrap()
    }
}
