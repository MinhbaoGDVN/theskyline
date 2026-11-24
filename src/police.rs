use std::f32::consts::{PI, TAU};

use macroquad::prelude::*;

use crate::city::City;
use crate::config::{rnd, BLOCK, WORLD_LIMIT};
use crate::models::Models;
use crate::render::{push_box, push_model};

/// Trạng thái bắt giữ của người chơi
#[derive(Clone, Copy, PartialEq)]
pub enum Arrest {
    Free,
    Carried,
    Jailed,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Foot,
    Car,
}

/// Công an: đi bộ hoặc lái xe cảnh sát; xe đang chở người thì `carrying = true`
pub struct Cop {
    pub pos: Vec3,
    pub heading: f32,
    pub kind: Kind,
    pub carrying: bool,
    want: f32,
}

impl Cop {
    /// Sinh công an cách người chơi 70-110 m
    pub fn spawn(center: Vec3, kind: Kind) -> Self {
        let a = rnd(0.0, TAU);
        let d = rnd(70.0, 110.0);
        let x = (center.x + a.cos() * d).clamp(-WORLD_LIMIT, WORLD_LIMIT);
        let z = (center.z + a.sin() * d).clamp(-WORLD_LIMIT, WORLD_LIMIT);
        Self {
            pos: vec3(x, 0.0, z),
            heading: rnd(0.0, TAU),
            kind,
            carrying: false,
            want: 0.0,
        }
    }

    /// Đuổi theo `target` với tốc độ `speed`; gặp nhà thì quay đầu né
    pub fn update(&mut self, dt: f32, target: Vec3, speed: f32, city: &City) {
        let to = vec2(target.x - self.pos.x, target.z - self.pos.z);
        if to.length() > 1.0 {
            let want = to.x.atan2(to.y);
            let mut d = want - self.heading;
            d = (d + PI).rem_euclid(TAU) - PI;
            self.heading += d.clamp(-2.5 * dt, 2.5 * dt);
        }
        let f = vec3(self.heading.sin(), 0.0, self.heading.cos());
        let next = self.pos + f * speed * dt;
        if city.hit(vec2(next.x, next.z), 0.8, 1.0) {
            self.heading += 1.2;
        } else {
            self.pos = vec3(
                next.x.clamp(-WORLD_LIMIT, WORLD_LIMIT),
                0.0,
                next.z.clamp(-WORLD_LIMIT, WORLD_LIMIT),
            );
        }
    }

    /// Xe cảnh sát chạy theo làn đường lưới: quay mượt, chỉ rẽ ở giao lộ, không đâm qua nhà
    pub fn drive(&mut self, dt: f32, target: Vec3, speed: f32, city: &City) {
        if self.kind != Kind::Car {
            self.update(dt, target, speed, city);
            return;
        }
        let dist = vec2(target.x - self.pos.x, target.z - self.pos.z).length();
        if dist < 4.0 {
            return;
        }
        let k = BLOCK;
        let moving_z = self.heading.cos().abs() >= self.heading.sin().abs();
        // Kéo nhẹ về đúng vạch đường (không dịch đột ngột)
        let blend = (dt * 8.0).min(1.0);
        if moving_z {
            let line = (self.pos.x / k).round() * k;
            self.pos.x += (line - self.pos.x) * blend;
        } else {
            let line = (self.pos.z / k).round() * k;
            self.pos.z += (line - self.pos.z) * blend;
        }
        // Tại giao lộ thì chọn hướng mới về phía mục tiêu
        let along = if moving_z { self.pos.z } else { self.pos.x };
        let at_intersection = ((along / k).round() * k - along).abs() < 2.5;
        if at_intersection {
            let dx = target.x - self.pos.x;
            let dz = target.z - self.pos.z;
            if moving_z && dx.abs() > 8.0 && dx.abs() > dz.abs() {
                self.want = if dx > 0.0 { PI * 0.5 } else { -PI * 0.5 };
            } else if !moving_z && dz.abs() > 8.0 && dz.abs() > dx.abs() {
                self.want = if dz > 0.0 { 0.0 } else { PI };
            }
        }
        // Quay từ từ về hướng mong muốn (tối đa 2.5 rad/giây)
        let mut d = self.want - self.heading;
        d = (d + PI).rem_euclid(TAU) - PI;
        self.heading += d.clamp(-2.5 * dt, 2.5 * dt);
        // Chỉ chạy khi đã gần đúng hướng
        let aligned = 1.0 - (d.abs() / PI).min(1.0);
        let f = vec3(self.heading.sin(), 0.0, self.heading.cos());
        let next = self.pos + f * speed * aligned * dt;
        if city.hit(vec2(next.x, next.z), 0.8, 1.0) {
            // Gặp nhà: quay sang bên để tránh
            self.want = self.heading + PI * 0.5;
        } else {
            self.pos = vec3(
                next.x.clamp(-WORLD_LIMIT, WORLD_LIMIT),
                0.0,
                next.z.clamp(-WORLD_LIMIT, WORLD_LIMIT),
            );
        }
    }

    pub fn draw(&self, verts: &mut Vec<Vertex>, models: &Models) {
        match self.kind {
            Kind::Car => {
                let m = Mat4::from_translation(self.pos) * Mat4::from_rotation_y(self.heading);
                push_model(verts, &models.car().parts, m);
                push_box(verts, self.pos + vec3(0.0, 2.1, 0.0), vec3(0.8, 0.25, 0.4), [0.9, 0.1, 0.1]);
            }
            Kind::Foot => {
                let p = self.pos;
                push_box(verts, p + vec3(-0.2, 0.42, 0.0), vec3(0.24, 0.84, 0.26), [0.1, 0.1, 0.18]);
                push_box(verts, p + vec3(0.2, 0.42, 0.0), vec3(0.24, 0.84, 0.26), [0.1, 0.1, 0.18]);
                push_box(verts, p + vec3(0.0, 1.19, 0.0), vec3(0.56, 0.70, 0.32), [0.12, 0.25, 0.6]);
                push_box(verts, p + vec3(0.0, 1.76, 0.0), vec3(0.40, 0.42, 0.40), [0.95, 0.8, 0.65]);
                push_box(verts, p + vec3(0.0, 1.98, 0.0), vec3(0.44, 0.12, 0.44), [0.1, 0.1, 0.12]);
            }
        }
    }
}
