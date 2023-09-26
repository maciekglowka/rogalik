use rogalik_math::vectors::Vector2F;

use rogalik_engine::traits::ResourceId;
use crate::structs::Vertex;
use super::BindParams;

#[derive(Clone, Copy)]
pub struct SpriteAtlas {
    rows: usize,
    cols: usize,
    pub texture_id: ResourceId,
    pub u_step: f32,
    pub v_step: f32
}
impl SpriteAtlas {
    pub fn new(
        texture_id: ResourceId,
        rows: usize,
        cols: usize,
    ) -> Self {
        Self {
            rows,
            cols,
            texture_id,
            u_step: 1.0,
            v_step: 1.0,
        }
    }
    pub fn get_sprite(
        &self, 
        index: usize,
        camera_id: ResourceId,
        position: Vector2F,
        size: Vector2F
    ) -> ([Vertex; 4], [u16; 6], BindParams) {
        let u = 0.0;
        let v = 0.0;

        let vertices = [
            Vertex { position: [position.x, position.y, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [u, v] },
            Vertex { position: [position.x + size.x, position.y, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [u + self.u_step, v] },
            Vertex { position: [position.x + size.x, position.y + size.y, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [u + self.u_step, v + self.v_step] },
            Vertex { position: [position.x, position.y + size.y, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [u, v + self.v_step] }
        ];
        let indices = [0, 1, 2, 0, 2, 3];
        (vertices, indices, BindParams { texture_id: self.texture_id, camera_id })
    }
}
