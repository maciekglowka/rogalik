use wgpu::util::DeviceExt;

use rogalik_common::{Camera, EngineError};
use rogalik_math::vectors::Vector2f;

const Z_RANGE: f32 = 100.;

pub struct Camera2D {
    scale: f32,
    target: Vector2f,
    vw: f32, // viewport
    vh: f32,
    rw: f32, // rendering
    rh: f32,
    bind_group: Option<wgpu::BindGroup>,
    buffer: Option<wgpu::Buffer>,
}
impl Camera for Camera2D {
    fn get_scale(&self) -> f32 {
        self.scale
    }
    fn get_target(&self) -> Vector2f {
        self.target
    }
    fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
    fn set_target(&mut self, target: Vector2f) {
        self.target = target;
    }
    fn camera_to_world(&self, v: Vector2f) -> Vector2f {
        // in physical pixels
        let x = v.x * self.rw / self.vw;
        let y = v.y * self.rh / self.vh;
        Vector2f::new(
            (x - 0.5 * self.rw) / self.scale + self.target.x,
            (y - 0.5 * self.rh) / self.scale + self.target.y,
        )
    }
    fn get_bounds(&self) -> (Vector2f, Vector2f) {
        let hx = 0.5 * self.rw / self.scale;
        let hy = 0.5 * self.rh / self.scale;
        (
            Vector2f::new(self.target.x - hx, self.target.y - hy),
            Vector2f::new(self.target.x + hx, self.target.y + hy),
        )
    }
}
impl Camera2D {
    pub fn new(vw: f32, vh: f32, rw: f32, rh: f32, scale: f32, target: Vector2f) -> Self {
        Self {
            scale,
            target,
            vw,
            vh,
            rw,
            rh,
            bind_group: None,
            buffer: None,
        }
    }
    pub fn create_wgpu_data(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
        let (bind_group, buffer) = Camera2D::create_bind_group(device, layout, self.get_matrix());
        self.bind_group = Some(bind_group);
        self.buffer = Some(buffer);
    }
    pub fn write_buffer(&self, queue: &wgpu::Queue) -> Result<(), EngineError> {
        queue.write_buffer(
            self.buffer.as_ref().ok_or(EngineError::GraphicsNotReady)?,
            0,
            bytemuck::cast_slice(&[self.get_matrix()]),
        );
        Ok(())
    }
    pub fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }
    pub fn resize_viewport(&mut self, vw: f32, vh: f32, rw: f32, rh: f32) {
        self.vw = vw;
        self.vh = vh;
        self.rw = rw;
        self.rh = rh;
    }
    fn get_matrix(&self) -> [[f32; 4]; 4] {
        let zoom = 1. / self.scale;
        let n = -Z_RANGE;
        let f = Z_RANGE;
        let l = self.target.x - zoom * self.rw / 2.;
        let r = self.target.x + zoom * self.rw / 2.;
        let t = self.target.y - zoom * self.rh / 2.;
        let b = self.target.y + zoom * self.rh / 2.;

        [
            [2. / (r - l), 0., 0., 0.],
            [0., 2. / (b - t), 0., 0.],
            [0., 0., 1. / (f - n), 0.],
            [-(r + l) / (r - l), -(b + t) / (b - t), -n / (f - n), 1.],
        ]
    }
    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        matrix: [[f32; 4]; 4],
    ) -> (wgpu::BindGroup, wgpu::Buffer) {
        let buffer = get_camera_buffer(device, matrix);
        (
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                label: Some("Camera Bind Group"),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            }),
            buffer,
        )
    }
}

pub fn get_camera_buffer(device: &wgpu::Device, matrix: [[f32; 4]; 4]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[matrix]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}
