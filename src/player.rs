use macroquad::prelude::*;

use crate::city::City;
use crate::config::WORLD_LIMIT;
use crate::obj::parse_obj;
use crate::render::push_model;

pub const MAX_HP: f32 = 100.0;
pub const MAX_STAMINA: f32 = 100.0;
const WALK_SPEED: f32 = 9.5;
const RUN_SPEED: f32 = 17.0;
const TURN_SPEED: f32 = 4.0;

/// Khối lập phương đơn vị (cạnh 1, tâm tại gốc): dùng để vẽ mọi bộ phận cơ thể
const CUBE: &str = "usemtl c
v -0.5 -0.5 -0.5
v 0.5 -0.5 -0.5
v 0.5 0.5 -0.5
v -0.5 0.5 -0.5
v -0.5 -0.5 0.5
v 0.5 -0.5 0.5
v 0.5 0.5 0.5
v -0.5 0.5 0.5
f 1 4 3 2
f 5 6 7 8
f 1 5 8 4
f 2 3 7 6
f 4 8 7 3
f 1 2 6 5
";

/// Vẽ một khối có màu, theo ma trận biến đổi (xoay được)
fn part(out: &mut Vec<Vertex>, m: Mat4, rgb: [f32; 3]) {
    let mtl = format!("newmtl c\nKd {} {} {}\n", rgb[0], rgb[1], rgb[2]);
    let parts = parse_obj(CUBE, &mtl);
    push_model(out, &parts, m);
}

/// Nhân vật đi bộ: cơ thể làm từ khối xoay được, tay chân đung đưa theo nhịp bước,
/// vị trí và hướng được nội suy giữa các bước logic để chuyển động mượt
pub struct Player {
    pub pos: Vec3,
    pub heading: f32,
    prev_pos: Vec3,
    prev_heading: f32,
    pub hp: f32,
    pub stamina: f32,
    pub money: u32,
    pub hurt_cd: f32,
    walk_t: f32,
    step_dist: f32,
    step_ready: bool,
    moving: bool,
}

impl Player {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            heading: 0.0,
            prev_pos: pos,
            prev_heading: 0.0,
            hp: MAX_HP,
            stamina: MAX_STAMINA,
            money: 0,
            hurt_cd: 0.0,
            walk_t: 0.0,
            step_dist: 0.0,
            step_ready: false,
            moving: false,
        }
    }

    /// Đi bộ: W/S tiến lùi, A/D quay (trái là quay trái), Ctrl để chạy (tốn thể lực)
    pub fn update(&mut self, dt: f32, up: bool, down: bool, left: bool, right: bool, run: bool, city: &City) {
        self.prev_pos = self.pos;
        self.prev_heading = self.heading;
        self.hurt_cd -= dt;
        let turn = (if left { 1.0 } else { 0.0 }) - (if right { 1.0 } else { 0.0 });
        self.heading += turn * TURN_SPEED * dt;
        let dir = (if up { 1.0 } else { 0.0 }) - (if down { 1.0 } else { 0.0 });
        self.moving = dir != 0.0;
        let running = run && self.moving && self.stamina > 0.0;
        if self.moving {
            let speed = if running { RUN_SPEED } else { WALK_SPEED };
            let f = vec3(self.heading.sin(), 0.0, self.heading.cos());
            let next = self.pos + f * dir * speed * dt;
            let next = vec3(next.x.clamp(-WORLD_LIMIT, WORLD_LIMIT), 0.0, next.z.clamp(-WORLD_LIMIT, WORLD_LIMIT));
            if !city.hit(vec2(next.x, next.z), 0.6, 1.0) {
                self.pos = next;
            }
            self.walk_t += dt * speed * 1.6;
            self.step_dist += speed * dt;
            let stride = if running { 2.6 } else { 2.0 };
            if self.step_dist >= stride {
                self.step_dist -= stride;
                self.step_ready = true;
            }
        }
        if running {
            self.stamina -= 25.0 * dt;
        } else {
            self.stamina += 15.0 * dt;
        }
        self.stamina = self.stamina.clamp(0.0, MAX_STAMINA);
    }

    /// Trả về true đúng một lần mỗi khi nhân vật bước một bước
    pub fn take_step(&mut self) -> bool {
        let r = self.step_ready;
        self.step_ready = false;
        r
    }

    /// Vị trí và hướng để vẽ, nội suy theo alpha (0..1) giữa bước logic trước và sau
    pub fn render_state(&self, alpha: f32) -> (Vec3, f32) {
        let a = alpha.clamp(0.0, 1.0);
        let pos = self.prev_pos.lerp(self.pos, a);
        let mut d = self.heading - self.prev_heading;
        d = (d + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
        (pos, self.prev_heading + d * a)
    }

    /// Vẽ nhân vật. Hệ tọa độ cục bộ: z là phía trước, x là bên phải, y là lên.
    pub fn draw(&self, verts: &mut Vec<Vertex>, alpha: f32) {
        let (pos, h) = self.render_state(alpha);
        let t = get_time() as f32;
        let swing = if self.moving { self.walk_t.sin() * 0.55 } else { 0.0 };
        let bob = if self.moving {
            (self.walk_t * 2.0).sin().abs() * 0.05
        } else {
            (t * 2.0).sin() * 0.01
        };
        let base = Mat4::from_translation(pos) * Mat4::from_rotation_y(h) * Mat4::from_translation(vec3(0.0, bob, 0.0));

        let shirt = [0.20, 0.45, 0.85];
        let pants = [0.18, 0.18, 0.25];
        let shoe = [0.10, 0.08, 0.08];
        let skin = [0.95, 0.80, 0.65];
        let hair = [0.15, 0.10, 0.07];
        let dark = [0.05, 0.05, 0.05];
        let belt = [0.10, 0.10, 0.10];

        // Hai chân: xoay quanh hông, bước đối nhau
        for (side, phase) in [(-1.0_f32, 1.0_f32), (1.0, -1.0)] {
            let leg = base
                * Mat4::from_translation(vec3(side * 0.17, 0.84, 0.0))
                * Mat4::from_rotation_x(swing * phase);
            part(verts, leg * Mat4::from_translation(vec3(0.0, -0.42, 0.0)) * Mat4::from_scale(vec3(0.24, 0.84, 0.26)), pants);
            part(
                verts,
                leg * Mat4::from_translation(vec3(0.0, -0.84 + 0.07, 0.05)) * Mat4::from_scale(vec3(0.26, 0.14, 0.36)),
                shoe,
            );
        }

        // Thân, thắt lưng, cổ
        part(verts, base * Mat4::from_translation(vec3(0.0, 1.19, 0.0)) * Mat4::from_scale(vec3(0.56, 0.70, 0.32)), shirt);
        part(verts, base * Mat4::from_translation(vec3(0.0, 0.90, 0.0)) * Mat4::from_scale(vec3(0.58, 0.14, 0.34)), belt);
        part(verts, base * Mat4::from_translation(vec3(0.0, 1.55, 0.02)) * Mat4::from_scale(vec3(0.2, 0.08, 0.2)), skin);

        // Hai tay: xoay quanh vai, đung đưa ngược chiều chân
        for (side, phase) in [(-1.0_f32, -1.0_f32), (1.0, 1.0)] {
            let arm = base
                * Mat4::from_translation(vec3(side * 0.37, 1.46, 0.0))
                * Mat4::from_rotation_x(swing * phase * 0.8);
            part(verts, arm * Mat4::from_translation(vec3(0.0, -0.31, 0.0)) * Mat4::from_scale(vec3(0.18, 0.62, 0.2)), shirt);
            part(verts, arm * Mat4::from_translation(vec3(0.0, -0.69, 0.0)) * Mat4::from_scale(vec3(0.16, 0.14, 0.18)), skin);
        }

        // Đầu, tóc, mắt
        part(verts, base * Mat4::from_translation(vec3(0.0, 1.76, 0.0)) * Mat4::from_scale(vec3(0.40, 0.42, 0.40)), skin);
        part(verts, base * Mat4::from_translation(vec3(0.0, 1.95, -0.01)) * Mat4::from_scale(vec3(0.42, 0.10, 0.42)), hair);
        part(verts, base * Mat4::from_translation(vec3(-0.09, 1.78, 0.2)) * Mat4::from_scale(vec3(0.07, 0.06, 0.02)), dark);
        part(verts, base * Mat4::from_translation(vec3(0.09, 1.78, 0.2)) * Mat4::from_scale(vec3(0.07, 0.06, 0.02)), dark);
    }
}
