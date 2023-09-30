use rogalik_math::vectors::Vector2f;

use rogalik_engine::{ResourceId, Params2d};
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
            u_step: 1.0 / rows as f32,
            v_step: 1.0 / cols as f32,
        }
    }
    pub fn get_sprite(
        &self, 
        index: usize,
        camera_id: ResourceId,
        position: Vector2f,
        size: Vector2f,
        params: Params2d
    ) -> ([Vertex; 4], [u16; 6], BindParams) {
        let row = index / self.cols;
        let col = index % self.cols;
        let u = self.u_step * col as f32;
        let v = self.v_step * row as f32;

        let color = params.color.as_srgb();
        let l = u; let r = u + self.u_step;
        let b = v + self.v_step; let t = v;

        let mut uvs = [[l, b], [r, b], [r, t], [l, t]];

        if params.flip_x {
            for uv in uvs.iter_mut() {
                if uv[0] == l { uv[0] = r } else { uv[0] = l }
            }
        }
        if params.flip_y {
            for uv in uvs.iter_mut() {
                if uv[1] == b { uv[1] = t } else { uv[1] = b }
            }
        }

        let vertices = [
            Vertex { position: [position.x, position.y, 0.0], color, tex_coords: uvs[0] },
            Vertex { position: [position.x + size.x, position.y, 0.0], color, tex_coords: uvs[1] },
            Vertex { position: [position.x + size.x, position.y + size.y, 0.0], color, tex_coords: uvs[2] },
            Vertex { position: [position.x, position.y + size.y, 0.0], color, tex_coords: uvs[3] }
        ];
        let indices = [0, 1, 2, 0, 2, 3];
        (vertices, indices, BindParams { texture_id: self.texture_id, camera_id })
    }
}
