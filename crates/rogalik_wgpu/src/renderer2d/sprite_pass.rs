use rogalik_common::EngineError;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

use crate::assets::WgpuAssets;
use crate::structs::{BindParams, Triangle, Vertex};

pub struct SpritePass {
    pub clear_color: wgpu::Color,
    vertex_queue: Vec<Vertex>,
    triangle_queue: Vec<Triangle>,
    // pipeline: wgpu::RenderPipeline,
    // pub bind_group_layout: wgpu::BindGroupLayout,
}
impl SpritePass {
    pub fn new(clear_color: wgpu::Color) -> Self {
        Self {
            clear_color,
            vertex_queue: Vec::new(),
            triangle_queue: Vec::new(),
        }
    }
    pub fn add_to_queue(
        &mut self,
        vertices: &[Vertex],
        indices: &[u16],
        z_index: i32,
        params: BindParams,
    ) {
        // TODO add error if indices are not divisible by 3
        let offset = self.vertex_queue.len() as u16;
        self.vertex_queue.extend(vertices);
        self.triangle_queue
            .extend(indices.chunks(3).map(|v| Triangle {
                indices: [v[0] + offset, v[1] + offset, v[2] + offset],
                z_index,
                params,
            }))
    }
    pub fn render(
        &mut self,
        assets: &WgpuAssets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        time_bind_group: &wgpu::BindGroup,
        view: &wgpu::TextureView,
    ) -> Result<(), EngineError> {
        if self.triangle_queue.len() == 0 {
            self.vertex_queue.clear();
            return Ok(());
        };

        // create on the fly as camera's position might have changed
        let mut camera_bind_groups = HashMap::new();
        for (i, camera) in assets.iter_cameras().enumerate() {
            camera_bind_groups.insert(
                i,
                camera.get_bind_group(
                    device,
                    assets
                        .bind_group_layouts
                        .get(&crate::assets::bind_groups::BindGroupKind::Camera)
                        .ok_or(EngineError::GraphicsInternalError)?,
                ),
            );
        }

        // let output = surface
        //     .get_current_texture()
        //     .map_err(|_| EngineError::GraphicsNotReady)?;

        // let view = output
        //     .texture
        //     .create_view(&wgpu::TextureViewDescriptor::default());

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite vertex buffer"),
            contents: bytemuck::cast_slice(&self.vertex_queue),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // let start = std::time::Instant::now();
        self.triangle_queue.sort_by(|a, b| {
            a.z_index
                .cmp(&b.z_index)
                .then(a.params.shader_id.cmp(&b.params.shader_id))
                .then(a.params.material_id.cmp(&b.params.material_id))
                .then(a.params.camera_id.cmp(&b.params.camera_id))
        });
        // log::debug!("Triangle sort: {:?}", start.elapsed());

        let indices = self
            .triangle_queue
            .iter()
            .map(|t| t.indices)
            .flatten()
            .collect::<Vec<_>>();

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Sprite Encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sprite Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let mut offset = 0;
            let mut batch_start = 0;
            let mut current_params = self.triangle_queue[0].params;

            let pipeline = assets
                .get_shader(current_params.shader_id)
                .ok_or(EngineError::GraphicsInternalError)?
                .pipeline
                .as_ref()
                .ok_or(EngineError::GraphicsNotReady)?;
            pass.set_pipeline(pipeline);

            let bind_group = assets
                .get_material(current_params.material_id)
                .ok_or(EngineError::GraphicsInternalError)?
                .bind_group
                .as_ref()
                .ok_or(EngineError::GraphicsNotReady)?;
            pass.set_bind_group(0, bind_group, &[]);

            pass.set_bind_group(
                1,
                camera_bind_groups.get(&current_params.camera_id.0).unwrap(),
                &[],
            );
            pass.set_bind_group(2, time_bind_group, &[]);

            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            for tri in self.triangle_queue.iter() {
                let end = offset + 3 as u32;

                if current_params != tri.params {
                    // draw the previous batch first
                    pass.draw_indexed(batch_start..offset, 0, 0..1);
                    // counter += 1;
                    if current_params.shader_id != tri.params.shader_id {
                        let pipeline = assets
                            .get_shader(tri.params.shader_id)
                            .ok_or(EngineError::GraphicsInternalError)?
                            .pipeline
                            .as_ref()
                            .ok_or(EngineError::GraphicsNotReady)?;
                        pass.set_pipeline(pipeline);
                    }
                    if current_params.material_id != tri.params.material_id {
                        let bind_group = assets
                            .get_material(tri.params.material_id)
                            .ok_or(EngineError::GraphicsInternalError)?
                            .bind_group
                            .as_ref()
                            .ok_or(EngineError::GraphicsNotReady)?;
                        pass.set_bind_group(0, bind_group, &[]);
                    }
                    if current_params.camera_id != tri.params.camera_id {
                        pass.set_bind_group(1, &camera_bind_groups[&tri.params.camera_id.0], &[]);
                    }
                    current_params = tri.params;
                    batch_start = offset;
                }
                offset = end;
            }
            pass.draw_indexed(batch_start..offset, 0, 0..1);
        }
        queue.submit(std::iter::once(encoder.finish()));
        // let start = std::time::Instant::now();
        // output.present();
        // println!("Present: {:?}, {}", start.elapsed(), counter);

        self.vertex_queue.clear();
        self.triangle_queue.clear();
        Ok(())
    }
}
