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
    pub bind_groups: HashMap<UniformKind, wgpu::BindGroup>,
    buffers: HashMap<UniformKind, wgpu::Buffer>,
}
impl Uniforms {
    pub fn create_wgpu_data(&mut self, layout: &wgpu::BindGroupLayout, device: &wgpu::Device) {
        let (globals_bind_group, globals_buffer) = self.globals.get_bind_group(device, layout);
        self.bind_groups
            .insert(UniformKind::Globals, globals_bind_group);
        self.buffers.insert(UniformKind::Globals, globals_buffer);
        let (lights_bind_group, lights_buffer) = self.lights.get_bind_group(device, layout);
        self.bind_groups
            .insert(UniformKind::Lights, lights_bind_group);
        self.buffers.insert(UniformKind::Lights, lights_buffer);
    }
    pub fn write_buffers(&self, queue: &wgpu::Queue) -> Result<(), EngineError> {
        queue.write_buffer(
            self.buffers
                .get(&UniformKind::Globals)
                .ok_or(EngineError::GraphicsNotReady)?,
            0,
            bytemuck::cast_slice(&[self.globals]),
        );
        queue.write_buffer(
            self.buffers
                .get(&UniformKind::Lights)
                .ok_or(EngineError::GraphicsNotReady)?,
            0,
            bytemuck::cast_slice(&[self.lights]),
        );
        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::NoUninit, bytemuck::Zeroable)]
pub struct GlobalsUniform {
    pub time: f32,
    _padding_0: u32,
    _padding_1: u32,
    _padding_2: u32,
    pub render_size: [f32; 2],
    pub viewport_size: [f32; 2],
}
impl GlobalsUniform {
    pub fn get_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> (wgpu::BindGroup, wgpu::Buffer) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Buffer"),
            contents: bytemuck::cast_slice(&[*self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        (
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                label: Some("Global Bind Group"),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            }),
            buffer,
        )
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
        let rgb = color.as_f32();
        self.ambient = rgb;
    }
    pub fn add_light(
        &mut self,
        position: Vector2f,
        radius: f32,
        color: Color,
        falloff: f32,
    ) -> Result<(), EngineError> {
        if self.light_count >= super::MAX_LIGHTS {
            return Err(EngineError::GraphicsInternalError);
        }
        self.lights[self.light_count as usize] = PointLight::new(position, radius, color, falloff);
        self.light_count += 1;
        Ok(())
    }
    pub fn get_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> (wgpu::BindGroup, wgpu::Buffer) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[*self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        (
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                label: Some("Light Bind Group"),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            }),
            buffer,
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLight {
    position: [f32; 3],
    radius: f32,
    color: [f32; 3],
    falloff: f32,
}
impl PointLight {
    pub fn new(position: Vector2f, radius: f32, color: Color, falloff: f32) -> Self {
        let rgba = color.as_f32();

        Self {
            position: [position.x, position.y, 0.],
            radius,
            color: [rgba[0], rgba[1], rgba[2]],
            falloff,
        }
    }
}
