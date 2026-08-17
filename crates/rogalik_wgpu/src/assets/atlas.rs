use rogalik_math::vectors::Vector2f;

use crate::structs::{Quad, Vertex};
use rogalik_common::{structs::AtlasPosition, SpriteParams};

#[derive(Clone, Copy, Debug)]
pub(crate) struct AtlasEntry {
    pub(crate) u: f32,
    pub(crate) u_size: f32,
    pub(crate) v: f32,
    pub(crate) v_size: f32,
    pub(crate) w: u32,
    pub(crate) h: u32,
}

#[derive(Clone, Debug, Default)]
pub struct SpriteAtlas {
    entries: Vec<AtlasEntry>,
    pub(crate) texture_size: (u32, u32),
}
impl SpriteAtlas {
    pub fn from_grid(
        texture_size: (u32, u32),
        rows: usize,
        cols: usize,
        padding: Option<(u32, u32)>,
    ) -> Self {
        let (sp_w, sp_h) =
            grid_sprite_pixel_size(texture_size.0, texture_size.1, rows, cols, padding);

        let u_step = 1.0 / cols as f32;
        let v_step = 1.0 / rows as f32;
        let u_size = sp_w as f32 / texture_size.0 as f32;
        let v_size = sp_h as f32 / texture_size.1 as f32;

        let mut entries = vec![];
        for row in 0..rows {
            for col in 0..cols {
                entries.push(AtlasEntry {
                    u: col as f32 * u_step,
                    u_size,
                    v: row as f32 * v_step,
                    v_size,
                    w: sp_w,
                    h: sp_h,
                });
            }
        }

        Self {
            entries,
            texture_size,
        }
    }
    pub(crate) fn from_entries(entries: &[AtlasPosition], texture_size: (u32, u32)) -> Self {
        let entries = entries
            .iter()
            .map(|e| AtlasEntry {
                w: e.w,
                h: e.h,
                u: e.x as f32 / texture_size.0 as f32,
                v: e.y as f32 / texture_size.1 as f32,
                u_size: e.w as f32 / texture_size.0 as f32,
                v_size: e.h as f32 / texture_size.1 as f32,
            })
            .collect();

        Self {
            entries,
            texture_size,
        }
    }
    pub(crate) fn get_entry(&self, index: usize) -> Option<&AtlasEntry> {
        self.entries.get(index)
    }

    pub fn get_sprite(
        &self,
        index: usize,
        position: Vector2f,
        size: Vector2f,
        params: SpriteParams,
    ) -> Quad {
        let entry = &self.entries[index];

        let color = params.color.as_srgb();
        let l = entry.u;
        let r = entry.u + entry.u_size;
        let b = entry.v + entry.v_size;
        let t = entry.v;

        let mut uvs = [[l, b], [r, b], [r, t], [l, t]];

        if params.flip_x {
            for uv in uvs.iter_mut() {
                if uv[0] == l {
                    uv[0] = r
                } else {
                    uv[0] = l
                }
            }
        }
        if params.flip_y {
            for uv in uvs.iter_mut() {
                if uv[1] == b {
                    uv[1] = t
                } else {
                    uv[1] = b
                }
            }
        }

        let mut vertices = [
            Vertex {
                position: [position.x, position.y, 0.0],
                color,
                tex_coords: uvs[0],
            },
            Vertex {
                position: [position.x + size.x, position.y, 0.0],
                color,
                tex_coords: uvs[1],
            },
            Vertex {
                position: [position.x + size.x, position.y + size.y, 0.0],
                color,
                tex_coords: uvs[2],
            },
            Vertex {
                position: [position.x, position.y + size.y, 0.0],
                color,
                tex_coords: uvs[3],
            },
        ];
        if params.rotate != 0. {
            // not tested for performance :)
            // perhaps should be moved to the shader
            let cx = position.x + 0.5 * size.x;
            let cy = position.y + 0.5 * size.y;
            rotate_verts(&mut vertices, params.rotate, cx, cy);
        }
        let indices = [0, 1, 2, 0, 2, 3];
        (vertices, indices)
    }

    pub fn get_sliced_sprite(
        &self,
        index: usize,
        position: Vector2f,
        size: Vector2f,
        params: SpriteParams,
    ) -> ([Vertex; 16], [u16; 54]) {
        let entry = &self.entries[index];

        let color = params.color.as_srgb();

        let slice_dim = params.slice.unwrap();

        let sprite_w = self.texture_size.0 as f32 * entry.u_size;
        let sprite_h = self.texture_size.1 as f32 * entry.v_size;

        let ratio_w = slice_dim as f32 / sprite_w;
        let ratio_h = slice_dim as f32 / sprite_h;

        let u_slice = ratio_w * entry.u_size;
        let v_slice = ratio_h * entry.v_size;

        let mut us = [
            entry.u,
            entry.u + u_slice,
            entry.u + entry.u_size - u_slice,
            entry.u + entry.u_size,
        ];
        let mut vs = [
            entry.v + entry.v_size,
            entry.v + entry.v_size - v_slice,
            entry.v + v_slice,
            entry.v,
        ];

        let xs = [
            position.x,
            position.x + slice_dim as f32,
            position.x + size.x - slice_dim as f32,
            position.x + size.x,
        ];
        let ys = [
            position.y,
            position.y + slice_dim as f32,
            position.y + size.y - slice_dim as f32,
            position.y + size.y,
        ];

        if params.flip_x {
            us.reverse();
        }
        if params.flip_y {
            vs.reverse();
        }

        let mut vertices = [Vertex::default(); 16];
        let mut idx = 0;
        for (y, v) in ys.iter().zip(vs) {
            for (x, u) in xs.iter().zip(us) {
                vertices[idx] = Vertex {
                    position: [*x, *y, 0.0],
                    color,
                    tex_coords: [u, v],
                };
                idx += 1;
            }
        }
        let indices = [
            12, 8, 13, 8, 9, 13, 13, 9, 14, 9, 10, 14, 14, 10, 15, 10, 11, 15, 8, 4, 9, 4, 5, 9, 9,
            5, 10, 5, 6, 10, 10, 6, 11, 6, 7, 11, 4, 0, 5, 0, 1, 5, 5, 1, 6, 1, 2, 6, 6, 2, 7, 2,
            3, 7,
        ];

        if params.rotate != 0. {
            let cx = position.x + 0.5 * size.x;
            let cy = position.y + 0.5 * size.y;
            rotate_verts(&mut vertices, params.rotate, cx, cy);
        }

        (vertices, indices)
    }
}

/// Calculate single sprite size for an even-grid atlas.
fn grid_sprite_pixel_size(
    texture_w: u32,
    texture_h: u32,
    rows: usize,
    cols: usize,
    padding: Option<(u32, u32)>,
) -> (u32, u32) {
    let grid_width = texture_w / cols as u32;
    let grid_height = texture_h / rows as u32;

    match padding {
        None => (grid_width, grid_height),
        Some((x, y)) => (grid_width - x, grid_height - y),
    }
}

fn rotate_verts(vertices: &mut [Vertex], angle: f32, cx: f32, cy: f32) {
    // not tested for performance :)
    // perhaps should be moved to the shader
    let c = angle.cos();
    let s = angle.sin();

    for v in vertices {
        v.position[0] -= cx;
        v.position[1] -= cy;

        let x = v.position[0];
        v.position[0] = x * c - v.position[1] * s;
        v.position[1] = x * s + v.position[1] * c;

        v.position[0] += cx;
        v.position[1] += cy;
    }
}
