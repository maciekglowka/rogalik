use std::collections::HashMap;
use wgpu::util::DeviceExt;

use crate::camera;
use crate::structs::Vertex;
use super::texture::Texture2d;
use super::Triangle;

pub struct SpritePass {
    clear_color: wgpu::Color,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}
impl SpritePass {
    pub fn new(
        clear_color: wgpu::Color,
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat
    ) -> Self {
        let shader = device.create_shader_module(
            wgpu::include_wgsl!("sprite_shader.wgsl")
        );

        let bind_group_layout = get_bind_group_layout(device);

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Sprite Pipeline Layout"),
                bind_group_layouts: &[
                    &bind_group_layout,
                    &camera::get_camera_bind_group_layout(device)
                ],
                push_constant_ranges: &[]
            }
        );
        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Sprite pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        Vertex::layout()
                    ]
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: *texture_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL
                    })]
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                multiview: None
            }
        );

        Self {
            clear_color,
            pipeline,
            bind_group_layout,
        }
    }
    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
    pub fn render(
        &mut self,
        cameras: &Vec<camera::Camera2D>,
        textures: &Vec<Texture2d>,
        verts: &Vec<Vertex>,
        tris: &Vec<Triangle>,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue
    ) -> Result<(), wgpu::SurfaceError> {
        if tris.len() == 0 { return Ok(()) };
        let mut camera_bind_groups = HashMap::new();
        for (i, camera) in cameras.iter().enumerate() {
            camera_bind_groups.insert(i, camera.get_bind_group(device));
        }

        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );
        
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Sprite vertex buffer"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX
            }
        );
        let indices = tris.iter().map(|t| t.indices).flatten().collect::<Vec<_>>();
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Sprite index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX
            }
        );

        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Sprite Encoder")}
        );
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sprite Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: true
                        },
                    })
                ],
                depth_stencil_attachment: None
            });

            pass.set_pipeline(&self.pipeline);
            let mut offset = 0;
            let mut batch_start = 0;
            let mut current_params = tris[0].params;

            pass.set_vertex_buffer(0, vertex_buffer.slice(..)); 
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16); 
            pass.set_bind_group(0, textures[current_params.texture_id.0].get_bind_group(), &[]);
            // TODO bind camera
            pass.set_bind_group(1, camera_bind_groups.get(&current_params.camera_id.0).unwrap(), &[]);

            for tri in tris {
                let end = offset + 3 as u32;

                if current_params != tri.params {
                    // draw the previous batch first
                    pass.draw_indexed(batch_start..offset, 0, 0..1);

                    if current_params.texture_id != tri.params.texture_id {
                        pass.set_bind_group(0, textures[tri.params.texture_id.0].get_bind_group(), &[]);
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
        output.present();
        Ok(())
    }
}

fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("Sprite Bind Group Layout")
    })
}