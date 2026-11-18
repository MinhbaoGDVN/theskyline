use macroquad::prelude::*;

use crate::city::City;
use crate::config::rnd;

pub struct Shell {
    pub pos: Vec3,
    vel: Vec3,
    life: f32,
}

impl Shell {
    pub fn new(pos: Vec3, vel: Vec3) -> Self {
        Self { pos, vel, life: 6.0 }
    }
}

pub struct Particle {
    pub pos: Vec3,
    vel: Vec3,
    life: f32,
    pub rgb: [f32; 3],
}

const PALETTE: [[f32; 3]; 3] = [[1.0, 0.55, 0.1], [1.0, 0.85, 0.3], [0.35, 0.35, 0.35]];

/// Cập nhật đạn. Trả về các điểm nổ khi đạn trúng đất hoặc tòa nhà.
pub fn step_shells(shells: &mut Vec<Shell>, dt: f32, city: &City) -> Vec<Vec3> {
    let mut blasts = Vec::new();
    shells.retain_mut(|s| {
        s.vel.y -= 25.0 * dt;
        s.pos += s.vel * dt;
        s.life -= dt;
        let dead = s.pos.y <= 0.3 || city.hit(s.pos.xz(), 0.4, s.pos.y) || s.life <= 0.0;
        if dead && s.life > 0.0 {
            blasts.push(vec3(s.pos.x, s.pos.y.max(0.5), s.pos.z));
        }
        !dead
    });
    blasts
}

pub fn burst(out: &mut Vec<Particle>, pos: Vec3, n: usize, speed: f32) {
    for i in 0..n {
        let dir = vec3(rnd(-0.5, 0.5), rnd(0.2, 0.8), rnd(-0.5, 0.5)).normalize_or_zero();
        out.push(Particle {
            pos,
            vel: dir * speed * rnd(0.5, 1.0),
            life: rnd(0.6, 1.2),
            rgb: PALETTE[i % PALETTE.len()],
        });
    }
}

pub fn step_particles(particles: &mut Vec<Particle>, dt: f32) {
    particles.retain_mut(|p| {
        p.vel.y -= 25.0 * dt;
        p.pos += p.vel * dt;
        p.life -= dt;
        p.life > 0.0 && p.pos.y >= 0.0
    });
}