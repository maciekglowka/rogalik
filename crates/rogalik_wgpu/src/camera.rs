use wgpu::util::DeviceExt;

use rogalik_math::vectors::Vector2F;

pub struct Camera {
    pub matrix: [[f32; 4]; 4],
    bind_group: wgpu::BindGroup
}
impl Camera {
    pub fn new(device: &wgpu::Device) -> Self {
        let matrix = [[0.; 4]; 4];
        let mut camera = Self {
            matrix,
            bind_group: Camera::create_bind_group(device, matrix)
        };
        camera.update_matrix(device);
        camera
    }
    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    fn update_matrix(&mut self, device: &wgpu::Device) {
        let l = 0.;
        let r = 800.;
        let t = 0.;
        let b = 600.;
        let n = -100.;
        let f = 100.;
        // self.matrix = [
        //     [2. / (r - l), 0., 0., -(r + l)/(r - l)],
        //     [0., 2. / (b - t), 0., -(b + t)/(b - t)],
        //     [0., 0., 1. / (f - n), -n / (f - n)],
        //     [0., 0., 0., 1.]
        // ];
        self.matrix = [
            [2. / (r - l), 0., 0., 0.],
            [0., 2. / (b - t), 0., 0.],
            [0., 0., 1. / (f - n), 0.],
            [-(r + l)/(r - l), -(b + t)/(b - t), -n / (f - n), 1.]
        ];
        // self.matrix = [
        //     [1., 0., 0., 0.],
        //     [0., 1., 0., 0.],
        //     [0., 0., 1., 0.],
        //     [0., 0., 0., 1.],
        // ];
        self.bind_group = Camera::create_bind_group(device, self.matrix);
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
