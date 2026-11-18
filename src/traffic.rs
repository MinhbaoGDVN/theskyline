use std::f32::consts::{FRAC_PI_2, PI};

use macroquad::prelude::*;

use crate::config::{rnd, BLOCK, EXT, GRID_N};

/// Khoảng cách từ tâm đường tới làn xe (làn phải, đường rộng 8 m)
const LANE: f32 = 5.0;

/// Xe NPC chạy thẳng trên một đường, quay đầu khi tới biên
pub struct NpcCar {
    along_z: bool,
    road: f32,
    dir: f32,
    t: f32,
    speed: f32,
}

impl NpcCar {
    pub fn random() -> Self {
        let k = rnd(0.0, (2 * GRID_N + 1) as f32).floor() - GRID_N as f32;
        Self {
            along_z: rnd(0.0, 1.0) < 0.5,
            road: k * BLOCK,
            dir: if rnd(0.0, 1.0) < 0.5 { 1.0 } else { -1.0 },
            t: rnd(-EXT, EXT),
            speed: rnd(12.0, 22.0),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.t += self.dir * self.speed * dt;
        if self.t > EXT {
            self.t -= 2.0 * EXT;
        }
        if self.t < -EXT {
            self.t += 2.0 * EXT;
        }
    }

    pub fn position(&self) -> Vec3 {
        if self.along_z {
            let side = if self.dir > 0.0 { -LANE } else { LANE };
            vec3(self.road + side, 0.0, self.t)
        } else {
            let side = if self.dir > 0.0 { LANE } else { -LANE };
            vec3(self.t, 0.0, self.road + side)
        }
    }

    fn angle(&self) -> f32 {
        match (self.along_z, self.dir > 0.0) {
            (true, true) => 0.0,
            (true, false) => PI,
            (false, true) => FRAC_PI_2,
            (false, false) => -FRAC_PI_2,
        }
    }

    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.position()) * Mat4::from_rotation_y(self.angle())
    }
}