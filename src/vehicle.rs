use std::f32::consts::{FRAC_PI_2, TAU};

use macroquad::prelude::*;

use crate::city::City;
use crate::config::{rnd, WORLD_LIMIT};
use crate::input::Controls;

/// Đổi m/s sang km/h (chỉ dùng để đổi đơn vị, không làm tỉ lệ nào khác)
const KMH_PER_MPS: f32 = 1.5;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Car,
    Bike,
    Plane,
    Tank,
}

/// Tốc độ tối đa của từng số (m/s)
fn gear_tops(kind: Kind) -> [f32; 4] {
    match kind {
        Kind::Car => [26.6667, 53.3333, 85.3333, 120.0],
        Kind::Bike => [12.0, 24.0, 38.0, 55.0],
        Kind::Tank => [6.0, 11.0, 17.0, 22.0],
        Kind::Plane => [120.0; 4],
    }
}

/// Hệ số gia tốc theo số: số thấp tăng tốc nhanh, số cao tăng tốc chậm
const GEAR_ACCEL: [f32; 4] = [1.6, 1.1, 0.8, 0.6];

struct Tuning {
    max_rev: f32,
    accel: f32,
    brake: f32,
    drag: f32,
    turn: f32,
    radius: f32,
}

fn tuning(kind: Kind) -> Tuning {
    match kind {
        Kind::Car => Tuning { max_rev: 12.0, accel: 22.0, brake: 35.0, drag: 8.0, turn: 1.9, radius: 1.4 },
        Kind::Bike => Tuning { max_rev: 8.0, accel: 28.0, brake: 30.0, drag: 7.0, turn: 2.6, radius: 0.9 },
        Kind::Tank => Tuning { max_rev: 6.0, accel: 12.0, brake: 20.0, drag: 5.0, turn: 1.2, radius: 2.0 },
        Kind::Plane => Tuning { max_rev: 0.0, accel: 0.0, brake: 0.0, drag: 0.0, turn: 0.0, radius: 2.0 },
    }
}

fn sgn(x: f32) -> f32 {
    if x < 0.0 { -1.0 } else { 1.0 }
}

fn b(x: bool) -> f32 {
    if x { 1.0 } else { 0.0 }
}

pub struct Vehicle {
    pub kind: Kind,
    pub name: &'static str,
    pub pos: Vec3,
    pub heading: f32,
    /// Tốc độ theo m/s (suy ra từ kmh cho xe mặt đất)
    pub speed: f32,
    /// Tốc độ hiển thị trên UI (km/h). Đây là nguồn duy nhất của chuyển động xe mặt đất.
    pub kmh: f32,
    pub pitch: f32,
    pub roll: f32,
    pub throttle: f32,
    pub cool: f32,
    pub broken: bool,
    /// Số hiện tại: 0 đến 3 tương ứng số 1 đến số 4
    pub gear: usize,
    /// Chế độ số tự động
    pub auto_gear: bool,
}

impl Vehicle {
    pub fn new(kind: Kind, pos: Vec3, heading: f32) -> Self {
        let name = match kind {
            Kind::Car => "Ô tô",
            Kind::Bike => "Xe máy",
            Kind::Plane => "Máy bay",
            Kind::Tank => "Xe tăng",
        };
        Self {
            kind,
            name,
            pos,
            heading,
            speed: 0.0,
            kmh: 0.0,
            pitch: 0.0,
            roll: 0.0,
            throttle: 0.0,
            cool: 0.0,
            broken: false,
            gear: 0,
            auto_gear: true,
        }
    }

    /// Hồi phục: đưa xe ra vị trí trống gần nhất và quay mũi ra xa công trình gần nhất
    pub fn reset(&mut self, city: &City) {
        self.pitch = 0.0;
        self.roll = 0.0;
        self.speed = 0.0;
        self.kmh = 0.0;
        self.throttle = 0.0;
        self.broken = false;

        let r = if self.kind == Kind::Plane { 2.0 } else { tuning(self.kind).radius };
        let origin = self.pos;
        'search: for ring in 0..60 {
            let d = ring as f32 * 4.0;
            let steps = (8 * ring).max(1);
            for s in 0..steps {
                let a = s as f32 * TAU / steps as f32;
                let p = vec2(origin.x + a.cos() * d, origin.z + a.sin() * d);
                if !city.hit(p, r, 0.0) {
                    self.pos.x = p.x;
                    self.pos.z = p.y;
                    break 'search;
                }
            }
        }

        // Quay mũi ra xa công trình gần nhất trong vòng 80 m
        let mut best: Option<(f32, Vec2)> = None;
        for b in &city.buildings {
            let c = (b.min + b.max) * 0.5;
            let dist = vec2(self.pos.x - c.x, self.pos.z - c.y).length();
            if dist < 80.0 && best.map_or(true, |(bd, _)| dist < bd) {
                best = Some((dist, c));
            }
        }
        if let Some((_, c)) = best {
            let away = vec2(self.pos.x - c.x, self.pos.z - c.y);
            if away.length() > 1e-3 {
                self.heading = away.x.atan2(away.y);
            }
        }

        // Máy bay giữ độ cao (không thấp hơn 5 m), xe mặt đất đặt trên mặt đất
        self.pos.y = if self.kind == Kind::Plane { origin.y.max(5.0) } else { 0.0 };
    }

    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.pos)
            * Mat4::from_rotation_y(self.heading)
            * Mat4::from_rotation_x(-self.pitch)
            * Mat4::from_rotation_z(self.roll)
    }

    /// Trả về true nếu máy bay vừa rơi
    pub fn update(&mut self, dt: f32, c: &Controls, city: &City) -> bool {
        if self.kind == Kind::Plane {
            let crashed = self.update_plane(dt, c, city);
            self.kmh = self.speed * KMH_PER_MPS;
            crashed
        } else {
            self.update_ground(dt, c, city);
            false
        }
    }

    fn crash(&mut self) {
        self.broken = true;
        self.speed = 0.0;
        self.kmh = 0.0;
        self.throttle = 0.0;
    }

    fn clamp_world(&mut self) {
        self.pos.x = self.pos.x.clamp(-WORLD_LIMIT, WORLD_LIMIT);
        self.pos.z = self.pos.z.clamp(-WORLD_LIMIT, WORLD_LIMIT);
    }

    /// Xử lý chuyển số: thủ công bằng X/Z, tự động theo tốc độ
    fn shift_gears(&mut self, c: &Controls) {
        let tops = gear_tops(self.kind);
        if c.gear_up && self.gear < 3 {
            self.gear += 1;
            self.auto_gear = false;
        }
        if c.gear_down && self.gear > 0 {
            self.gear -= 1;
            self.auto_gear = false;
        }
        if c.auto_toggle {
            self.auto_gear = !self.auto_gear;
        }
        if self.auto_gear {
            if self.gear < 3 && self.speed > tops[self.gear] * 0.92 {
                self.gear += 1;
            } else if self.gear > 0 && self.speed < tops[self.gear - 1] * 0.5 {
                self.gear -= 1;
            }
        }
    }

    fn update_ground(&mut self, dt: f32, c: &Controls, city: &City) {
        let t = tuning(self.kind);
        let tops = gear_tops(self.kind);
        let steer = b(c.left) - b(c.right);
        let k = KMH_PER_MPS;

        self.shift_gears(c);
        let cap = tops[self.gear] * k;

        // Tốc độ được tính trên UI (km/h): giữ W thì tăng dần theo thời gian
        if c.up {
            self.kmh += t.accel * GEAR_ACCEL[self.gear] * k * dt;
            if self.kmh > cap {
                self.kmh = cap;
            }
        } else if c.down {
            if self.kmh > 1.8 * k {
                self.kmh -= t.brake * k * dt;
            } else {
                self.kmh = (self.kmh - t.accel * 0.5 * k * dt).max(-t.max_rev * k);
            }
        } else {
            self.kmh -= sgn(self.kmh) * (t.drag * k * dt).min(self.kmh.abs());
        }
        if c.brake {
            self.kmh -= sgn(self.kmh) * (t.brake * 1.5 * k * dt).min(self.kmh.abs());
        }
        self.kmh = self.kmh.clamp(-t.max_rev * k, tops[3] * k);
        self.speed = self.kmh / k;

        let rate = t.turn * (self.speed.abs() / 8.0).min(1.0) * sgn(self.speed);
        self.heading += steer * rate * dt;

        // Di chuyển theo đúng tốc độ trên UI
        let dir = vec3(self.heading.sin(), 0.0, self.heading.cos());
        let next = self.pos + dir * self.speed * dt;
        if city.hit(next.xz(), t.radius, 0.0) {
            self.kmh *= -0.25;
            if self.kmh.abs() < k {
                self.kmh = 0.0;
            }
            self.speed = self.kmh / k;
        } else {
            self.pos = next;
        }
        self.clamp_world();
        self.pos.y = 0.0;
        self.roll = if self.kind == Kind::Bike {
            -steer * 0.2 * (self.speed.abs() / 20.0).min(1.0)
        } else {
            0.0
        };
    }

    fn update_plane(&mut self, dt: f32, c: &Controls, city: &City) -> bool {
        if self.broken {
            return false;
        }
        let support_now = city.roof_top(self.pos.xz(), 2.0).unwrap_or(0.0).max(0.0);
        let on_ground = self.pos.y <= support_now + 0.01;

        if c.thr_up {
            self.throttle = (self.throttle + 0.5 * dt).min(1.0);
        }
        if c.thr_dn {
            self.throttle = (self.throttle - 0.5 * dt).max(0.0);
        }
        self.speed = (self.speed + (self.throttle * 110.0 - self.speed) * dt * 0.5).max(0.0);

        let steer = b(c.left) - b(c.right);

        let (pitch_up, pitch_down) = if crate::input::invert_plane() { (c.down, c.up) } else { (c.up, c.down) };
        let mut pitch_t = 0.0;
        if pitch_up {
            pitch_t = 0.4;
        }
        if pitch_down {
            pitch_t = -0.4;
        }
        if on_ground && self.speed < 30.0 {
            pitch_t = 0.0;
        }
        self.pitch += (pitch_t - self.pitch) * dt * 1.5;

        if on_ground {
            self.heading += steer * 0.9 * (self.speed / 10.0).min(1.0) * dt;
            self.roll += (0.0 - self.roll) * dt * 5.0;
        } else {
            let roll_t = -steer * 0.7;
            self.roll += (roll_t - self.roll) * dt * 2.5;
            self.heading += -self.roll * 0.8 * (self.speed / 40.0).min(1.0) * dt;
        }

        let (sp, cp) = (self.pitch.sin(), self.pitch.cos());
        let vy = sp * self.speed - 3.0 * (1.0 - (self.speed / 60.0).min(1.0));
        let mut next = self.pos
            + vec3(self.heading.sin() * cp, 0.0, self.heading.cos() * cp) * self.speed * dt;
        next.y = self.pos.y + vy * dt;

        // Đâm vào thân toà nhà: đang ở dưới nóc mà bay vào trong toà
        let roof_next = city.roof_top(next.xz(), 2.0).unwrap_or(0.0);
        if roof_next > 0.0 && next.y < roof_next - 0.5 && self.pos.y < roof_next - 0.5 {
            self.crash();
            return true;
        }
        // Mặt đỡ: mặt đất hoặc nóc toà nhà bên dưới
        let support_next = roof_next.max(0.0);
        if next.y <= support_next {
            // Va cham manh: roi nhanh, hoac chuc mui xuong khi dang chay nhanh
            if self.pos.y > support_now + 0.5 && (vy < -6.0 || (self.pitch < -0.15 && self.speed > 15.0)) {
                self.crash();
                return true;
            }
            next.y = support_next;
        }

        next.x = next.x.clamp(-WORLD_LIMIT, WORLD_LIMIT);
        next.z = next.z.clamp(-WORLD_LIMIT, WORLD_LIMIT);
        next.y = next.y.min(900.0);
        self.pos = next;
        false
    }
}

pub fn default_fleet() -> Vec<Vehicle> {
    vec![
        // Ô tô: trung tâm thành phố
        Vehicle::new(Kind::Car, vec3(4.0, 0.0, 20.0), 0.0),
        // Xe máy: bãi xe phía nam
        Vehicle::new(Kind::Bike, vec3(0.0, 0.0, -650.0), 0.0),
        // Máy bay: sân bay phía tây, mũi hướng đông (vào thành phố)
        Vehicle::new(Kind::Plane, vec3(-700.0, 0.0, 0.0), FRAC_PI_2),
        // Xe tăng: bãi tăng phía đông, mũi hướng tây (vào thành phố)
        Vehicle::new(Kind::Tank, vec3(650.0, 0.0, 0.0), -FRAC_PI_2),
    ]
}
#[allow(dead_code)]
pub fn random_unused() -> f32 {
    rnd(0.0, 1.0)
}