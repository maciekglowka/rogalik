use rogalik_math::vectors::{Vector2f, Vector2i};

use rogalik_engine::{ResourceId, Params2d, Color};
use crate::structs::Vertex;
use super::BindParams;

#[derive(Clone, Copy, Debug)]
pub struct SpriteAtlas {
    rows: usize,
    cols: usize,
    pub texture_id: ResourceId,
    pub u_step: f32,
    pub v_step: f32,
    u_size: f32,
    v_size: f32,
    sprite_w: f32,
    sprite_h: f32
}
impl SpriteAtlas {
    pub fn new(
        texture_id: ResourceId,
        texture_size: Vector2i,
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>
    ) -> Self {
        let (sp_w, sp_h) = sprite_pixel_size(texture_size, rows, cols, padding);
        
        Self {
            rows,
            cols,
            texture_id,
            u_step: 1.0 / rows as f32,
            v_step: 1.0 / cols as f32,
            u_size: sp_w / texture_size.x as f32,
            v_size: sp_h / texture_size.y as f32,
            sprite_w: sp_w,
            sprite_h: sp_h,
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
        let l = u; let r = u + self.u_size;
        let b = v + self.v_size; let t = v;

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

    pub fn get_sliced_sprite(
        &self, 
        index: usize,
        camera_id: ResourceId,
        position: Vector2f,
        size: Vector2f,
        params: Params2d
    ) -> ([Vertex; 16], [u16; 54], BindParams) {
        let row = index / self.cols;
        let col = index % self.cols;
        let u = self.u_step * col as f32;
        let v = self.v_step * row as f32;

        let color = params.color.as_srgb();

        let (slice_dim, base_size) = params.slice.unwrap();

        let ratio_w = slice_dim as f32 / self.sprite_w;
        let ratio_h = slice_dim as f32 / self.sprite_h;
        let u_slice = ratio_w * self.u_size;
        let v_slice = ratio_h * self.v_size;
        let w_slice = ratio_w * base_size.x;
        let h_slice = ratio_h * base_size.y;
        let mut us = [u, u + u_slice, u + self.u_size - u_slice, u + self.u_size];
        let mut vs = [v + self.v_size, v + self.v_size - v_slice, v + v_slice, v];
        let xs = [position.x, position.x + w_slice, position.x + size.x - w_slice, position.x + size.x];
        let ys = [position.y, position.y + h_slice, position.y + size.y - h_slice, position.y + size.y];

        if params.flip_x {
            us.reverse();
        }
        if params.flip_y {
            vs.reverse();
        }

        let mut vertices = [Vertex::default(); 16];
        let mut idx = 0;
        for (y, v) in  ys.iter().zip(vs) {
            for (x, u) in xs.iter().zip(us) {
                vertices[idx] = Vertex { position: [*x, *y, 0.0], color, tex_coords: [u, v] };
                idx += 1;
            }
        }
        let indices = [
            12, 8, 13, 8, 9, 13,
            13, 9, 14, 9, 10, 14,
            14, 10, 15, 10, 11, 15,
            8, 4, 9, 4, 5, 9,
            9, 5, 10, 5, 6, 10,
            10, 6, 11, 6, 7, 11,
            4, 0, 5, 0, 1, 5,
            5, 1, 6, 1, 2, 6,
            6, 2, 7, 2, 3, 7
        ];

        (vertices, indices, BindParams { texture_id: self.texture_id, camera_id })
    }
}

pub fn sprite_pixel_size(
    texture_size: Vector2i,
    rows: usize,
    cols: usize,
    padding: Option<(f32, f32)>
) -> (f32,f32) {
    let grid_width = (texture_size.x as f32) / (cols as f32);
    let grid_height = (texture_size.y as f32) / (rows as f32);

    match padding {
        None => (grid_width, grid_height),
        Some((x, y)) => (grid_width - x, grid_height - y)
    }
}