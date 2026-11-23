use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::config::DRAW_DIST;
use crate::vehicle::{Kind, Vehicle};

/// Nửa góc nón nhìn, rộng hơn khung hình để vật thể không bị nhấp nháy ở mép
const HALF_CONE: f32 = 75.0 * PI / 180.0;

/// Camera bám sau phương tiện, di chuyển mượt
pub struct ChaseCam {
    pos: Vec3,
    look: Vec3,
    started: bool,
    fh: f32,
}

impl ChaseCam {
    pub fn new() -> Self {
        Self {
            pos: Vec3::ZERO,
            look: Vec3::ZERO,
            started: false,
            fh: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, v: &Vehicle) {
        // Khoảng cách (lùi lại phía sau, độ cao)
        let (dist, height) = match v.kind {
            Kind::Plane => (22.0, 6.0),
            Kind::Tank => (16.0, 5.0),
            _ => (12.0, 4.5),
        };
        let f = vec3(v.heading.sin(), 0.0, v.heading.cos());
        let mut desired = v.pos - f * dist;
        desired.y = (v.pos.y + height).max(1.5);

        if !self.started {
            self.pos = desired;
            self.started = true;
        } else {
            let k = 1.0 - (-6.0 * dt).exp();
            self.pos = self.pos.lerp(desired, k);
        }

        let mut look = v.pos + f * 6.0 + vec3(0.0, 1.5, 0.0);
        if v.kind == Kind::Plane {
            look.y += v.pitch.sin() * 8.0;
        }
        self.look = look;
    }

    /// Camera đi theo người khi đi bộ
    pub fn follow(&mut self, dt: f32, pos: Vec3, heading: f32) {
        if !self.started {
            self.fh = heading;
        }
        // Góc quay được làm mượt, không bám ngay theo nhân vật
        let mut d = heading - self.fh;
        d = (d + PI).rem_euclid(2.0 * PI) - PI;
        self.fh += d * (1.0 - (-10.0 * dt).exp());
        let f = vec3(self.fh.sin(), 0.0, self.fh.cos());
        let mut desired = pos - f * 9.0;
        desired.y = (pos.y + 4.0).max(2.0);
        if !self.started {
            self.pos = desired;
            self.started = true;
        } else {
            let k = 1.0 - (-12.0 * dt).exp();
            self.pos = self.pos.lerp(desired, k);
        }
        self.look = pos + f * 6.0 + vec3(0.0, 1.8, 0.0);
    }

    /// Vật thể (tâm center, bán kính radius) có nằm trong tầm nhìn không
    pub fn visible(&self, center: Vec3, radius: f32) -> bool {
        let to = center - self.pos;
        let d = to.length();
        if d <= radius {
            return true;
        }
        if d > DRAW_DIST + radius {
            return false;
        }
        let fwd = (self.look - self.pos).normalize_or_zero();
        let cos_ang = to.dot(fwd) / d;
        let limit = HALF_CONE + (radius / d).min(1.0).asin();
        cos_ang >= limit.cos()
    }

    pub fn camera(&self) -> Camera3D {
        Camera3D {
            position: self.pos,
            up: vec3(0.0, 1.0, 0.0),
            target: self.look,
            fovy: 60.0 * PI / 180.0,
            z_near: 2.0,
            z_far: 2200.0,
            ..Default::default()
        }
    }
}