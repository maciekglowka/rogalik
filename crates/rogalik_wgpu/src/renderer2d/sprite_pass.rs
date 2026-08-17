use rogalik_common::EngineError;
use std::collections::HashMap;

use crate::assets::WgpuAssets;
use crate::structs::{BindParams, Triangle, Vertex};

use super::uniforms::UniformKind;

struct DynamicBuffer {
    buffer: Option<wgpu::Buffer>,
    usage: wgpu::BufferUsages,
}
impl DynamicBuffer {
    fn ensure_size(&mut self, size: u64, device: &wgpu::Device) {
        if let Some(buffer) = &self.buffer {
            if buffer.size() >= size {
                return;
            }
        }

        // Add some headroom.
        let new_size = size.next_power_of_two();
        log::debug!("Allocating new sprite pass buffer with size: {new_size}");

        self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite pass buffer"),
            mapped_at_creation: false,
            size: new_size,
            usage: self.usage | wgpu::BufferUsages::COPY_DST,
        }));
    }
    fn unchecked(&self) -> &wgpu::Buffer {
        self.buffer.as_ref().unwrap()
    }
    fn clear(&mut self) {
        self.buffer = None;
    }
}

pub(crate) struct SpritePass {
    pub clear_color: wgpu::Color,
    vertex_queue: Vec<Vertex>,
    triangle_queue: Vec<Triangle>,
    vertex_buffer: DynamicBuffer,
    index_buffer: DynamicBuffer,
    /// Reusable temp buffer.
    indices: Vec<u16>,
}
impl SpritePass {
    pub(crate) fn new(clear_color: wgpu::Color) -> Self {
        Self {
            clear_color,
            vertex_queue: Vec::new(),
            triangle_queue: Vec::new(),
            vertex_buffer: DynamicBuffer {
                buffer: None,
                usage: wgpu::BufferUsages::VERTEX,
            },
            index_buffer: DynamicBuffer {
                buffer: None,
                usage: wgpu::BufferUsages::INDEX,
            },
            indices: vec![],
        }
    }
    pub(crate) fn create_wgpu_data(&mut self) {
        // Currently only clear dynamic buffers, so they will get recreated on a first
        // draw.
        self.vertex_buffer.clear();
        self.index_buffer.clear();
    }
    pub(crate) fn add_to_queue(
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
    pub(crate) fn render(
        &mut self,
        assets: &WgpuAssets,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uniform_bind_groups: &HashMap<UniformKind, wgpu::BindGroup>,
        view: &wgpu::TextureView,
    ) -> Result<(), EngineError> {
        if self.triangle_queue.is_empty() {
            self.vertex_queue.clear();
            return Ok(());
        };

        let vertex_size = self.vertex_queue.len() * std::mem::size_of::<Vertex>();
        self.vertex_buffer.ensure_size(vertex_size as u64, device);

        queue.write_buffer(
            self.vertex_buffer.unchecked(),
            0,
            bytemuck::cast_slice(&self.vertex_queue),
        );

        self.triangle_queue.sort_by(|a, b| {
            a.z_index
                .cmp(&b.z_index)
                .then(a.params.shader_id.cmp(&b.params.shader_id))
                .then(a.params.material_id.cmp(&b.params.material_id))
                .then(a.params.camera_id.cmp(&b.params.camera_id))
        });

        self.indices.clear();
        self.indices
            .extend(self.triangle_queue.iter().flat_map(|t| t.indices));
        // let indices = self
        //     .triangle_queue
        //     .iter()
        //     .flat_map(|t| t.indices)
        //     .collect::<Vec<_>>();

        // Single tri size == 6 Bytes (3 * u16).
        let index_size = 6 * self.triangle_queue.len();
        self.index_buffer.ensure_size(index_size as u64, device);

        let index_data = bytemuck::cast_slice(&self.indices);
        queue.write_buffer(self.index_buffer.unchecked(), 0, index_data);

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sprite Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
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
                assets
                    .cameras
                    .get(current_params.camera_id.0)
                    .ok_or(EngineError::ResourceNotFound)?
                    .get_bind_group()
                    .ok_or(EngineError::GraphicsNotReady)?,
                &[],
            );
            pass.set_bind_group(2, uniform_bind_groups.get(&UniformKind::Globals), &[]);
            pass.set_bind_group(3, uniform_bind_groups.get(&UniformKind::Lights), &[]);

            pass.set_vertex_buffer(0, self.vertex_buffer.unchecked().slice(..));
            pass.set_index_buffer(
                self.index_buffer.unchecked().slice(..),
                wgpu::IndexFormat::Uint16,
            );

            for tri in self.triangle_queue.iter() {
                let end = offset + 3;

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
                        pass.set_bind_group(
                            1,
                            assets
                                .cameras
                                .get(tri.params.camera_id.0)
                                .ok_or(EngineError::ResourceNotFound)?
                                .get_bind_group()
                                .ok_or(EngineError::GraphicsNotReady)?,
                            &[],
                        );
                    }
                    current_params = tri.params;
                    batch_start = offset;
                }
                offset = end;
            }
            pass.draw_indexed(batch_start..offset, 0, 0..1);
        }
        // let start = std::time::Instant::now();
        // output.present();
        // println!("Present: {:?}, {}", start.elapsed(), counter);

        self.vertex_queue.clear();
        self.triangle_queue.clear();
        Ok(())
    }
}
