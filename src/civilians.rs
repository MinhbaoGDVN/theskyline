use std::f32::consts::{PI, TAU};

use macroquad::prelude::*;

use crate::city::City;
use crate::config::{rnd, BLOCK, EXT, GRID_N, WORLD_LIMIT};
use crate::render::push_box;

/// Số dân thường tối đa trên đường
pub const CIV_TARGET: usize = 30;
const CIV_SPEED: f32 = 1.6;
/// Thời gian xác nằm lại trước khi biến mất (giây)
pub const CORPSE_TIME: f32 = 20.0;

/// Dân thường: đi bộ trên vỉa hè, tránh nhà, bị bắn thì ngã và biến mất sau một lúc
pub struct Civilian {
    pub pos: Vec3,
    pub heading: f32,
    pub dead: bool,
    pub dead_t: f32,
    turn_t: f32,
    shirt: [f32; 3],
}

impl Civilian {
    /// Một dân thường mới, đứng trên vỉa hè (lệch 7 m so với đường lưới)
    pub fn random_spawn() -> Self {
        let k = (rnd(0.0, (2 * GRID_N + 1) as f32).floor() as i32 - GRID_N) as f32 * BLOCK;
        let t = rnd(-EXT, EXT);
        let off = if rnd(0.0, 1.0) < 0.5 { 7.0 } else { -7.0 };
        let pos = if rnd(0.0, 1.0) < 0.5 {
            vec3(k + off, 0.0, t)
        } else {
            vec3(t, 0.0, k + off)
        };
        Self {
            pos,
            heading: rnd(0.0, TAU),
            dead: false,
            dead_t: 0.0,
            turn_t: rnd(2.0, 6.0),
            shirt: [rnd(0.2, 0.9), rnd(0.2, 0.9), rnd(0.2, 0.9)],
        }
    }

    /// Dân thường sinh trên vỉa hè gần một điểm (trong khoảng 150 m)
    pub fn spawn_near(center: Vec3) -> Self {
        let off = if rnd(0.0, 1.0) < 0.5 { 7.0 } else { -7.0 };
        let along = rnd(-150.0, 150.0);
        let pos = if rnd(0.0, 1.0) < 0.5 {
            let k = ((center.x + along) / BLOCK).round() * BLOCK;
            vec3(k + off, 0.0, (center.z + rnd(-150.0, 150.0)).clamp(-EXT, EXT))
        } else {
            let k = ((center.z + along) / BLOCK).round() * BLOCK;
            vec3((center.x + rnd(-150.0, 150.0)).clamp(-EXT, EXT), 0.0, k + off)
        };
        Self {
            pos,
            heading: rnd(0.0, TAU),
            dead: false,
            dead_t: 0.0,
            turn_t: rnd(2.0, 6.0),
            shirt: [rnd(0.2, 0.9), rnd(0.2, 0.9), rnd(0.2, 0.9)],
        }
    }

    pub fn update(&mut self, dt: f32, city: &City) {
        if self.dead {
            self.dead_t += dt;
            return;
        }
        self.turn_t -= dt;
        if self.turn_t <= 0.0 {
            self.heading += rnd(-1.5, 1.5);
            self.turn_t = rnd(2.0, 6.0);
        }
        let f = vec3(self.heading.sin(), 0.0, self.heading.cos());
        let next = self.pos + f * CIV_SPEED * dt;
        let blocked = city.hit(vec2(next.x, next.z), 0.5, 1.0)
            || next.x.abs() > WORLD_LIMIT
            || next.z.abs() > WORLD_LIMIT;
        if blocked {
            self.heading += PI * 0.5 + rnd(-0.5, 0.5);
            self.turn_t = 3.0;
        } else {
            self.pos = next;
        }
    }

    /// Tia bắn 3D (d chuẩn hoá) trúng dân thường: coi người là một hình cầu quanh ngực
    pub fn ray_hit(&self, o: Vec3, d: Vec3, range: f32) -> Option<f32> {
        if self.dead {
            return None;
        }
        let center = vec3(self.pos.x, 0.95, self.pos.z);
        let to = center - o;
        let t = to.dot(d);
        if t < 0.0 || t > range {
            return None;
        }
        let perp = (to - d * t).length();
        if perp < 0.8 {
            Some(t)
        } else {
            None
        }
    }

    pub fn draw(&self, verts: &mut Vec<Vertex>) {
        let p = self.pos;
        let skin = [0.95, 0.80, 0.65];
        if self.dead {
            push_box(verts, p + vec3(0.0, 0.25, 0.0), vec3(1.7, 0.45, 0.5), self.shirt);
            return;
        }
        push_box(verts, p + vec3(-0.2, 0.42, 0.0), vec3(0.24, 0.84, 0.26), [0.18, 0.18, 0.25]);
        push_box(verts, p + vec3(0.2, 0.42, 0.0), vec3(0.24, 0.84, 0.26), [0.18, 0.18, 0.25]);
        push_box(verts, p + vec3(0.0, 1.19, 0.0), vec3(0.56, 0.70, 0.32), self.shirt);
        push_box(verts, p + vec3(0.0, 1.76, 0.0), vec3(0.40, 0.42, 0.40), skin);
    }
}
