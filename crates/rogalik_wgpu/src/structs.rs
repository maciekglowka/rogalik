use crate::data::U8_TO_SRGB;
use crate::{Camera2d, Material, Shader};

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use rogalik_arena::ResourceId;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Color(pub u8, pub u8, pub u8, pub u8);
impl Color {
    pub fn as_f32(&self) -> [f32; 4] {
        [
            self.0 as f32 / 255.,
            self.1 as f32 / 255.,
            self.2 as f32 / 255.,
            self.3 as f32 / 255.,
        ]
    }

    #[inline(always)]
    pub fn as_srgb(&self) -> [f32; 4] {
        [
            U8_TO_SRGB[self.0 as usize],
            U8_TO_SRGB[self.1 as usize],
            U8_TO_SRGB[self.2 as usize],
            self.3 as f32 / 255.,
        ]
    }
}
impl Default for Color {
    fn default() -> Self {
        Self(255, 255, 255, 255)
    }
}

pub(crate) type Quad = ([Vertex; 4], [u16; 6]);

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}
impl Vertex {
    const ATTRS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
        2 => Float32x2,
    ];
    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Triangle {
    pub indices: [u16; 3],
    pub z_index: i32,
    pub params: BindParams,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BindParams {
    pub shader_id: ResourceId<Shader>,
    pub material_id: ResourceId<Material>,
    pub camera_id: ResourceId<Camera2d>,
}
