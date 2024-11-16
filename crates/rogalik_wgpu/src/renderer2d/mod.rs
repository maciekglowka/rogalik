use wgpu::util::DeviceExt;

use rogalik_common::{EngineError, ResourceId, SpriteParams};
use rogalik_math::vectors::Vector2f;

use crate::assets::WgpuAssets;
use crate::create_surface_state;
use crate::structs::{BindParams, Triangle, Vertex};

mod sprite_pass;

pub struct Renderer2d {
    sprite_pass: sprite_pass::SpritePass,
    // post_processing_passes: Vec
}
impl Renderer2d {
    pub fn new(clear_color: wgpu::Color) -> Self {
        let sprite_pass = sprite_pass::SpritePass::new(clear_color);
        Self { sprite_pass }
    }
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.sprite_pass.clear_color = color;
    }

    pub fn draw_atlas_sprite(
        &mut self,
        assets: &WgpuAssets,
        index: usize,
        material_name: &str,
        camera_id: ResourceId,
        position: Vector2f,
        z_index: i32,
        size: Vector2f,
        params: SpriteParams,
    ) -> Result<(), EngineError> {
        let &material_id = assets
            .get_material_id(material_name)
            .ok_or(EngineError::ResourceNotFound)?;
        let material = assets
            .get_material(material_id)
            .ok_or(EngineError::ResourceNotFound)?;

        let bind_params = BindParams {
            camera_id,
            material_id,
            shader_id: material.shader_id,
        };

        println!("{:?}", material);
        println!("{:?}", bind_params);
        if let Some(_) = params.slice {
            let s = material
                .atlas
                .ok_or(EngineError::InvalidResource)?
                .get_sliced_sprite(index, position, size, params);
            self.sprite_pass
                .add_to_queue(&s.0, &s.1, z_index, bind_params);
        } else {
            let s = material
                .atlas
                .ok_or(EngineError::InvalidResource)?
                .get_sprite(index, position, size, params);
            self.sprite_pass
                .add_to_queue(&s.0, &s.1, z_index, bind_params);
        };
        println!("Drew");
        Ok(())
    }
    pub fn draw_text(
        &mut self,
        assets: &WgpuAssets,
        font: &str,
        text: &str,
        camera_id: ResourceId,
        position: Vector2f,
        z_index: i32,
        size: f32,
        params: SpriteParams,
    ) -> Result<(), EngineError> {
        // let font = assets.get_font(font).ok_or(EngineError::ResourceNotFound)?;
        // let bind_params = BindParams {
        //     camera_id,
        //     texture_id: font.atlas.texture_id,
        // };
        // for s in font.get_sprites(text, position, size, params) {
        //     self.add_to_queue(&s.0, &s.1, z_index, bind_params);
        // }
        Ok(())
    }
    pub fn render(
        &mut self,
        assets: &WgpuAssets,
        time: f32,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), EngineError> {
        let time_bind_group = create_time_bind_group(
            device,
            assets
                .bind_group_layouts
                .get(&crate::assets::bind_groups::BindGroupKind::Time)
                .ok_or(EngineError::GraphicsNotReady)?,
            time,
        );
        self.sprite_pass
            .render(assets, surface, device, queue, &time_bind_group)?;
        Ok(())
    }
}

pub fn create_time_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    time: f32,
) -> wgpu::BindGroup {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Time Buffer"),
        contents: bytemuck::cast_slice(&[time]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        label: Some("Time Bind Group"),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    })
}
