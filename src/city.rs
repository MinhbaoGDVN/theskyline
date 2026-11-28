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

/// Nửa chiều rộng lối vào nhà (m)
pub const DOOR_HALF: f32 = 3.0;

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
    crate::house::body(&mut v, c, s, [rgb[0] * dim, rgb[1] * dim, rgb[2] * dim]);

    let roof_y = c.y + s.y * 0.5 + 0.4;
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
            window(&mut v, vec3(x, y, c.z + hz + 0.12), vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), is_lit(i, j, seed));
            window(&mut v, vec3(x, y, c.z - hz - 0.12), vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, -1.0), is_lit(i, j, seed + 1));
        }
        for k in 0..nz {
            let z = c.z - hz + 2.0 + k as f32 * 4.0;
            window(&mut v, vec3(c.x + hx + 0.12, y, z), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0), is_lit(k, j, seed + 2));
            window(&mut v, vec3(c.x - hx - 0.12, y, z), vec3(0.0, 0.0, 1.0), vec3(-1.0, 0.0, 0.0), is_lit(k, j, seed + 3));
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
        let sidewalk = [0.6, 0.6, 0.6];
        let park = [0.25, 0.56, 0.25];

        batch.push_box(vec3(0.0, -0.05, 0.0), vec3(4000.0, 0.1, 4000.0), grass);

        let len = 2.0 * 480.0;
        let mut road_verts = Vec::new();
        let lines: Vec<f32> = (-GRID_N..=GRID_N).map(|k| k as f32 * BLOCK).collect();
        crate::road::grid_roads(&mut road_verts, &lines, len, ROAD);

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

        let mut roads = batch.finish();
        roads.extend(chunk_meshes(road_verts));
        City {
            roads,
            buildings,
        }
    }

    /// Điểm p (mặt phẳng xz), bán kính r, độ cao y có chạm tòa nhà không
    /// Đỉnh cao nhất của các toà nhà bao quanh điểm p (bán kính r), hoặc None nếu không có toà nào
    pub fn roof_top(&self, p: Vec2, r: f32) -> Option<f32> {
        let mut best: Option<f32> = None;
        for b in &self.buildings {
            if p.x + r > b.min.x && p.x - r < b.max.x && p.y + r > b.min.y && p.y - r < b.max.y {
                best = Some(best.map_or(b.top, |t| t.max(b.top)));
            }
        }
        best
    }

    pub fn hit(&self, p: Vec2, r: f32, y: f32) -> bool {
        self.buildings.iter().any(|b| {
            // Lối vào: nhà thường có một hành lang rộng 6 m từ mặt đường vào trong, không chặn người
            if b.hp < 1.0e5 && (p.x - (b.min.x + b.max.x) * 0.5).abs() + r < DOOR_HALF {
                return false;
            }
            y < b.top && p.x + r > b.min.x && p.x - r < b.max.x && p.y + r > b.min.y && p.y - r < b.max.y
        })
    }

    /// Độ cao nóc của nhà chứa điểm p (nếu có), dùng để đứng trên nóc
    pub fn roof_at(&self, p: Vec2) -> Option<f32> {
        let mut best: Option<f32> = None;
        for b in &self.buildings {
            if p.x >= b.min.x && p.x <= b.max.x && p.y >= b.min.y && p.y <= b.max.y {
                best = Some(best.map_or(b.top, |t| t.max(b.top)));
            }
        }
        best
    }

    /// Gây sát thương tòa nhà gần điểm nổ. Trả về tâm và đỉnh các tòa đã sập.
    pub fn damage(&mut self, p: Vec2) -> Vec<(Vec3, f32)> {
        let mut dead = Vec::new();
        for (i, b) in self.buildings.iter_mut().enumerate() {
            if b.hp >= 1.0e5 {
                continue;
            }
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

impl City {
    /// Công trình tùy chỉnh có cửa sổ suốt chiều cao
    pub fn add_custom(&mut self, c: Vec2, hw: f32, hd: f32, h: f32, rgb: [f32; 3], hp: f32) {
        self.add_band(c, hw, hd, 0.0, h, rgb, hp, 0.0);
    }

    /// Khối trơn không cửa sổ
    pub fn add_plain(&mut self, c: Vec2, hw: f32, hd: f32, h: f32, rgb: [f32; 3], hp: f32) {
        self.add_band(c, hw, hd, 0.0, h, rgb, hp, f32::MAX);
    }

    /// Khối từ độ cao y0 đến y1. Cửa sổ chỉ vẽ ở các tầng có độ cao từ win_from trở lên.
    pub fn add_band(
        &mut self,
        c: Vec2,
        hw: f32,
        hd: f32,
        y0: f32,
        y1: f32,
        rgb: [f32; 3],
        hp: f32,
        win_from: f32,
    ) {
        let center = vec3(c.x, (y0 + y1) * 0.5, c.y);
        let size = vec3(hw * 2.0, y1 - y0, hd * 2.0);
        let min = c - vec2(hw, hd);
        let max = c + vec2(hw, hd);
        let mut shadow = Vec::new();
        push_shadow(&mut shadow, box_corners(vec3(min.x, 0.0, min.y), vec3(max.x, y1, max.y)));
        let mut v = Vec::new();
        push_box(&mut v, center, size, rgb);
        band_windows(&mut v, c, hw, hd, y0, y1, win_from);
        self.buildings.push(Building {
            min,
            max,
            top: y1,
            rgb,
            hp,
            meshes: chunk_meshes(v),
            shadow,
            center,
            size,
        });
    }

    /// Khối với mesh tự cung cấp (dùng cho tường OBJ). Va chạm theo khung min-max.
    pub fn add_verts(&mut self, min: Vec2, max: Vec2, top: f32, rgb: [f32; 3], hp: f32, verts: Vec<Vertex>) {
        let center = vec3((min.x + max.x) * 0.5, top * 0.5, (min.y + max.y) * 0.5);
        let size = vec3(max.x - min.x, top, max.y - min.y);
        let mut shadow = Vec::new();
        push_shadow(&mut shadow, box_corners(vec3(min.x, 0.0, min.y), vec3(max.x, top, max.y)));
        self.buildings.push(Building {
            min,
            max,
            top,
            rgb,
            hp,
            meshes: chunk_meshes(verts),
            shadow,
            center,
            size,
        });
    }

    /// Xóa các công trình có tâm nằm trong ô vuông quanh c (nửa cạnh half)
    pub fn clear_block(&mut self, c: Vec2, half: f32) {
        self.buildings.retain(|b| {
            let bc = (b.min + b.max) * 0.5;
            (bc.x - c.x).abs() > half || (bc.y - c.y).abs() > half
        });
    }
}

/// Cửa sổ cho một khối, chỉ ở các tầng có độ cao >= win_from
fn band_windows(out: &mut Vec<Vertex>, c: Vec2, hw: f32, hd: f32, y0: f32, y1: f32, win_from: f32) {
    let seed = ((c.x * 7.0) as i32).wrapping_add((c.y * 13.0) as i32);
    let nx = (((2.0 * hw - 2.0) / 4.0).floor() as i32 + 1).max(1);
    let nz = (((2.0 * hd - 2.0) / 4.0).floor() as i32 + 1).max(1);
    let mut y = y0 + 3.0;
    let mut j: i32 = 0;
    while y <= y1 - 2.0 {
        if y >= win_from {
            for i in 0..nx {
                let x = c.x - hw + 2.0 + i as f32 * 4.0;
                window(out, vec3(x, y, c.y + hd + 0.12), vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), is_lit(i, j, seed));
                window(out, vec3(x, y, c.y - hd - 0.12), vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, -1.0), is_lit(i, j, seed + 1));
            }
            for k in 0..nz {
                let z = c.y - hd + 2.0 + k as f32 * 4.0;
                window(out, vec3(c.x + hw + 0.12, y, z), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0), is_lit(k, j, seed + 2));
                window(out, vec3(c.x - hw - 0.12, y, z), vec3(0.0, 0.0, 1.0), vec3(-1.0, 0.0, 0.0), is_lit(k, j, seed + 3));
            }
        }
        y += 4.0;
        j += 1;
    }
}
