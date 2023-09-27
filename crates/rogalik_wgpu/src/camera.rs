use wgpu::util::DeviceExt;

use rogalik_engine::traits::Camera;
use rogalik_math::vectors::Vector2f;

const Z_RANGE: f32 = 100.;

pub struct Camera2D {
    scale: f32,
    target: Vector2f,
    vw: f32,
    vh: f32,
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
}
impl Camera2D {
    pub fn new(vw: f32, vh: f32, scale: f32, target: Vector2f) -> Self {
        Self {
            scale,
            target,
            vw,
            vh
        }
    }
    pub fn get_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        Camera2D::create_bind_group(device, self.get_matrix())
    }
    pub fn resize_viewport(&mut self, vw: f32, vh: f32) {
        self.vw = vw;
        self.vh = vh;
    }
    fn get_matrix(&self) -> [[f32; 4]; 4] {
        let zoom = 1. / self.scale;
        let n = -Z_RANGE;
        let f = Z_RANGE;
        let l = self.target.x - zoom * self.vw / 2.;
        let r = self.target.x + zoom * self.vw / 2.;
        let t = self.target.y - zoom * self.vh / 2.;
        let b = self.target.y + zoom * self.vh / 2.;

        [
            [2. / (r - l), 0., 0., 0.],
            [0., 2. / (b - t), 0., 0.],
            [0., 0., 1. / (f - n), 0.],
            [-(r + l)/(r - l), -(b + t)/(b - t), -n / (f - n), 1.]
        ]
    }
    fn create_bind_group(
        device: &wgpu::Device,
        matrix: [[f32; 4]; 4]
    ) -> wgpu::BindGroup {
        device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &get_camera_bind_group_layout(device),
                label: Some("Camera Bind Group"),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: get_camera_buffer(device, matrix)
                            .as_entire_binding()
                    }
                ]
            }
        )
    }
}

pub fn get_camera_buffer(device: &wgpu::Device, matrix: [[f32; 4]; 4]) -> wgpu::Buffer {
    device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        }
    )
}

pub fn get_camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor { 
            label: Some("Camera Bind Group Layou"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ]
        }
    )
}
