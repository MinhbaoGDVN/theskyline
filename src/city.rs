use macroquad::prelude::*;

use crate::config::{rnd, BLAST_RADIUS, BLOCK, BUILDING_HP, DIRECT_DAMAGE, GRID_N, ROAD};
use crate::render::{chunk_meshes, push_box, quad, Batch};
use crate::shadow::{box_corners, push_shadow};

const BUILDING_COLORS: [[f32; 3]; 7] = [
    [0.85, 0.83, 0.78],
    [0.72, 0.76, 0.81],
    [0.88, 0.71, 0.54],
    [0.56, 0.65, 0.72],
    [0.79, 0.66, 0.65],
    [0.65, 0.72, 0.62],
    [0.94, 0.93, 0.91],
];
const GLASS: [f32; 3] = [0.2, 0.3, 0.42];
const LIT: [f32; 3] = [1.0, 0.86, 0.5];

pub struct Building {
    pub min: Vec2,
    pub max: Vec2,
    pub top: f32,
    pub rgb: [f32; 3],
    pub hp: f32,
    pub meshes: Vec<Mesh>,
    pub shadow: Vec<Vertex>,
    center: Vec3,
    size: Vec3,
}

impl Building {
    fn rebake(&mut self) {
        let dim = 0.45 + 0.55 * (self.hp / BUILDING_HP).clamp(0.0, 1.0);
        self.meshes = chunk_meshes(building_verts(self.center, self.size, self.rgb, dim));
    }
}

pub struct City {
    pub roads: Vec<Mesh>,
    pub buildings: Vec<Building>,
}

fn is_lit(i: i32, j: i32, seed: i32) -> bool {
    (i * 31 + j * 17 + seed).rem_euclid(9) == 0
}

/// Một ô cửa sổ: tứ giác đứng nằm trên mặt tòa nhà
fn window(out: &mut Vec<Vertex>, center: Vec3, along: Vec3, normal: Vec3, lit: bool) {
    let up = vec3(0.0, 1.1, 0.0);
    let col = if lit { LIT } else { GLASS };
    quad(
        out,
        [center - along - up, center + along - up, center + along + up, center - along + up],
        normal,
        col,
    );
}

/// Toàn bộ đỉnh của một tòa: chân đế, thân, mái, cửa sổ
fn building_verts(c: Vec3, s: Vec3, rgb: [f32; 3], dim: f32) -> Vec<Vertex> {
    let mut v = Vec::new();
    let (hx, hz) = (s.x * 0.5, s.z * 0.5);

    push_box(&mut v, vec3(c.x, 0.7, c.z), vec3(s.x + 0.6, 1.4, s.z + 0.6), [0.35, 0.35, 0.37]);
    push_box(&mut v, c, s, [rgb[0] * dim, rgb[1] * dim, rgb[2] * dim]);

    let roof_y = c.y + s.y * 0.5 + 0.4;
    push_box(&mut v, vec3(c.x, roof_y, c.z), vec3(s.x + 1.0, 0.8, s.z + 1.0), [0.28, 0.28, 0.3]);
    if s.y > 40.0 {
        push_box(&mut v, vec3(c.x, roof_y + 1.5, c.z), vec3(s.x * 0.4, 3.0, s.z * 0.4), [0.6, 0.62, 0.66]);
    }

    let seed = ((c.x * 7.0) as i32).wrapping_add((c.z * 13.0) as i32);
    let nx = (((s.x - 2.0) / 4.0).floor() as i32 + 1).max(1);
    let nz = (((s.z - 2.0) / 4.0).floor() as i32 + 1).max(1);
    let ny = (((s.y - 4.0) / 4.0).floor() as i32 + 1).clamp(1, 30);

    for j in 0..ny {
        let y = 3.0 + j as f32 * 4.0;
        for i in 0..nx {
            let x = c.x - hx + 2.0 + i as f32 * 4.0;
            window(&mut v, vec3(x, y, c.z + hz + 0.03), vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), is_lit(i, j, seed));
            window(&mut v, vec3(x, y, c.z - hz - 0.03), vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, -1.0), is_lit(i, j, seed + 1));
        }
        for k in 0..nz {
            let z = c.z - hz + 2.0 + k as f32 * 4.0;
            window(&mut v, vec3(c.x + hx + 0.03, y, z), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0), is_lit(k, j, seed + 2));
            window(&mut v, vec3(c.x - hx - 0.03, y, z), vec3(0.0, 0.0, 1.0), vec3(-1.0, 0.0, 0.0), is_lit(k, j, seed + 3));
        }
    }
    v
}

fn make_building(c: Vec2, hw: f32, hd: f32) -> Building {
    let h = if rnd(0.0, 1.0) < 0.1 {
        rnd(90.0, 150.0)
    } else {
        rnd(12.0, 62.0)
    };
    let idx = (rnd(0.0, BUILDING_COLORS.len() as f32) as usize).min(BUILDING_COLORS.len() - 1);
    let rgb = BUILDING_COLORS[idx];
    let center = vec3(c.x, h * 0.5, c.y);
    let size = vec3(hw * 2.0, h, hd * 2.0);
    let min = c - vec2(hw, hd);
    let max = c + vec2(hw, hd);

    let mut shadow = Vec::new();
    push_shadow(&mut shadow, box_corners(vec3(min.x, 0.0, min.y), vec3(max.x, h, max.y)));

    Building {
        min,
        max,
        top: h,
        rgb,
        hp: BUILDING_HP,
        meshes: chunk_meshes(building_verts(center, size, rgb, 1.0)),
        shadow,
        center,
        size,
    }
}

impl City {
    pub fn generate() -> Self {
        let mut batch = Batch::new();
        let grass = [0.42, 0.61, 0.29];
        let asphalt = [0.2, 0.2, 0.22];
        let sidewalk = [0.6, 0.6, 0.6];
        let park = [0.25, 0.56, 0.25];
        let line = [1.0, 0.82, 0.25];

        batch.push_box(vec3(0.0, -0.05, 0.0), vec3(4000.0, 0.1, 4000.0), grass);

        let len = 2.0 * 480.0;
        for k in -GRID_N..=GRID_N {
            let p = k as f32 * BLOCK;
            batch.push_box(vec3(p, 0.03, 0.0), vec3(ROAD, 0.06, len), asphalt);
            batch.push_box(vec3(p, 0.1, 0.0), vec3(0.6, 0.02, len), line);
            batch.push_box(vec3(0.0, 0.03, p), vec3(len, 0.06, ROAD), asphalt);
            batch.push_box(vec3(0.0, 0.1, p), vec3(len, 0.02, 0.6), line);
        }

        let mut buildings = Vec::new();
        let size = BLOCK - ROAD;
        for i in -GRID_N..GRID_N {
            for j in -GRID_N..GRID_N {
                let c = vec2((i as f32 + 0.5) * BLOCK, (j as f32 + 0.5) * BLOCK);

                if !(i == 0 && j == 0) && rnd(0.0, 1.0) < 0.15 {
                    batch.push_box(vec3(c.x, 0.05, c.y), vec3(size, 0.1, size), park);
                    continue;
                }
                batch.push_box(vec3(c.x, 0.05, c.y), vec3(size, 0.1, size), sidewalk);

                if rnd(0.0, 1.0) < 0.5 {
                    buildings.push(make_building(c, rnd(8.0, 26.0), rnd(8.0, 26.0)));
                } else {
                    for dx in [-16.0f32, 16.0] {
                        buildings.push(make_building(c + vec2(dx, 0.0), rnd(6.0, 14.0), rnd(8.0, 20.0)));
                    }
                }
            }
        }

        City {
            roads: batch.finish(),
            buildings,
        }
    }

    /// Điểm p (mặt phẳng xz), bán kính r, độ cao y có chạm tòa nhà không
    pub fn hit(&self, p: Vec2, r: f32, y: f32) -> bool {
        self.buildings.iter().any(|b| {
            y < b.top && p.x + r > b.min.x && p.x - r < b.max.x && p.y + r > b.min.y && p.y - r < b.max.y
        })
    }

    /// Gây sát thương tòa nhà gần điểm nổ. Trả về tâm và đỉnh các tòa đã sập.
    pub fn damage(&mut self, p: Vec2) -> Vec<(Vec3, f32)> {
        let mut dead = Vec::new();
        for (i, b) in self.buildings.iter_mut().enumerate() {
            let d = p.distance(p.clamp(b.min, b.max));
            if d < BLAST_RADIUS {
                b.hp -= DIRECT_DAMAGE * (1.0 - d / BLAST_RADIUS);
                if b.hp <= 0.0 {
                    dead.push(i);
                } else {
                    b.rebake();
                }
            }
        }
        let mut debris = Vec::new();
        for i in dead.into_iter().rev() {
            let b = self.buildings.remove(i);
            debris.push((b.center, b.top));
        }
        debris
    }
}