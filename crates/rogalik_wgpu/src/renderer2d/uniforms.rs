use rogalik_common::{Color, EngineError};
use rogalik_math::vectors::Vector2f;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

#[derive(PartialEq, Eq, Hash)]
pub enum UniformKind {
    Globals,
    Lights,
}

#[derive(Default)]
pub struct Uniforms {
    pub globals: GlobalsUniform,
    pub lights: LightsUniform,
}
impl Uniforms {
    pub fn get_bind_groups(
        &self,
        layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
    ) -> HashMap<UniformKind, wgpu::BindGroup> {
        HashMap::from_iter([
            (
                UniformKind::Globals,
                self.globals.get_bind_group(device, layout),
            ),
            (
                UniformKind::Lights,
                self.lights.get_bind_group(device, layout),
            ),
        ])
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::NoUninit, bytemuck::Zeroable)]
pub struct GlobalsUniform {
    time: f32,
}
impl GlobalsUniform {
    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }
    pub fn get_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Buffer"),
            contents: bytemuck::cast_slice(&[*self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            label: Some("Global Bind Group"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::NoUninit, bytemuck::Zeroable)]
pub struct LightsUniform {
    light_count: u32,
    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
    ambient: [f32; 4],
    lights: [PointLight; super::MAX_LIGHTS as usize],
}
impl LightsUniform {
    pub fn frame_end(&mut self) {
        self.light_count = 0;
    }
    pub fn set_ambient(&mut self, color: Color) {
        let srgb = color.as_srgb();
        self.ambient = srgb;
    }
    pub fn add_light(
        &mut self,
        strength: f32,
        color: Color,
        position: Vector2f,
    ) -> Result<(), EngineError> {
        if self.light_count >= super::MAX_LIGHTS {
            return Err(EngineError::GraphicsInternalError);
        }
        self.lights[self.light_count as usize] = PointLight::new(position, color, strength);
        self.light_count += 1;
        Ok(())
    }
    pub fn get_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[*self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            label: Some("Light Bind Group"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLight {
    position: [f32; 3],
    strength: f32,
    color: [f32; 4],
}
impl PointLight {
    pub fn new(position: Vector2f, color: Color, strength: f32) -> Self {
        Self {
            position: [position.x, position.y, 0.],
            strength,
            color: color.as_srgb(),
        }
    }
}
