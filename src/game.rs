use macroquad::prelude::*;

use crate::camera::ChaseCam;
use crate::city::{Building, City};
use crate::combat::{self, Particle, Shell};
use crate::config::{BLAST_RADIUS, TRAFFIC_COUNT};
use crate::hud;
use crate::input::{self, Controls, Edges};
use crate::minimap;
use crate::models::Models;
use crate::render::{draw_verts, push_box, push_model};
use crate::shadow::{box_corners, push_shadow};
use crate::traffic::NpcCar;
use crate::vehicle::{default_fleet, Kind, Vehicle};

/// Bóng đổ có thể vươn xa hơn vật thể, nên phải kiểm tra thêm khoảng này
const SHADOW_MARGIN: f32 = 120.0;

/// Tâm và bán kính bao của một tòa nhà
fn building_bounds(b: &Building) -> (Vec3, f32) {
    let c = vec3((b.min.x + b.max.x) * 0.5, b.top * 0.5, (b.min.y + b.max.y) * 0.5);
    let r = vec3(b.max.x - b.min.x, b.top, b.max.y - b.min.y).length() * 0.5;
    (c, r)
}

/// Tâm và bán kính bao của một phương tiện
fn vehicle_bounds(v: &Vehicle) -> (Vec3, f32) {
    let r = if v.kind == Kind::Plane { 7.0 } else { 3.5 };
    (v.pos + vec3(0.0, 1.5, 0.0), r)
}

pub struct Game {
    models: Models,
    city: City,
    vehicles: Vec<Vehicle>,
    active: usize,
    traffic: Vec<NpcCar>,
    traffic_timer: f32,
    shells: Vec<Shell>,
    particles: Vec<Particle>,
    cam: ChaseCam,
    notice: String,
    notice_t: f32,
    font: Option<Font>,
    edges: Edges,
    dev: bool,
    tick_count: u32,
    tick_timer: f32,
    tps: u32,
    // Đo tốc độ trong cửa sổ 1 giây (thời gian game), chỉ hiện ở dev mode
    win_t: f32,
    win_start: Vec3,
    path_m: f32,
    bumps: u32,
    straight_kmh: f32,
    path_kmh: f32,
    bumps_last: u32,
}

impl Game {
    pub fn new(models: Models, font: Option<Font>) -> Self {
        let vehicles = default_fleet();
        let win_start = vehicles[0].pos;
        Self {
            models,
            city: City::generate(),
            vehicles,
            active: 0,
            traffic: (0..TRAFFIC_COUNT).map(|_| NpcCar::random()).collect(),
            traffic_timer: 0.0,
            shells: Vec::new(),
            particles: Vec::new(),
            cam: ChaseCam::new(),
            notice: "Chào mừng đến The Skyline!".to_string(),
            notice_t: 3.0,
            font,
            edges: Edges::default(),
            dev: false,
            tick_count: 0,
            tick_timer: 0.0,
            tps: 0,
            win_t: 0.0,
            win_start,
            path_m: 0.0,
            bumps: 0,
            straight_kmh: 0.0,
            path_kmh: 0.0,
            bumps_last: 0,
        }
    }

    fn notify(&mut self, text: &str, secs: f32) {
        self.notice = text.to_string();
        self.notice_t = secs;
    }

    /// Gọi một lần mỗi frame, trước các bước logic
    pub fn begin_frame(&mut self) {
        let e = input::read_edges();
        self.edges.merge(&e);
    }

    /// Một bước logic cố định (dt luôn là 1/60 giây)
    pub fn update(&mut self, dt: f32) {
        self.tick_count += 1;
        self.tick_timer += dt;
        if self.tick_timer >= 1.0 {
            self.tps = self.tick_count;
            self.tick_count = 0;
            self.tick_timer -= 1.0;
        }

        // Phím bấm chỉ dùng cho bước đầu tiên của frame
        let e = std::mem::take(&mut self.edges);
        if e.dev_toggle {
            self.dev = !self.dev;
            let state = if self.dev { "BẬT" } else { "TẮT" };
            self.notify(&format!("Dev mode: {state}"), 2.0);
        }
        let mut c: Controls = input::read();
        c.gear_up = e.gear_up;
        c.gear_down = e.gear_down;
        c.auto_toggle = e.auto_toggle;
        self.handle_switch_and_reset(&e);

        for v in &mut self.vehicles {
            v.cool = (v.cool - dt).max(0.0);
        }

        let mut blasts: Vec<Vec3> = Vec::new();
        let active = self.active;
        let before = self.vehicles[active].pos;
        let crashed = self.vehicles[active].update(dt, &c, &self.city);
        if crashed {
            blasts.push(self.vehicles[active].pos + vec3(0.0, 1.0, 0.0));
            self.notify("Máy bay rơi! Nhấn R để hồi phục", 3.0);
        }

        // Đo quãng đường đi thực tế và phát hiện va chạm (cho dev mode)
        let v = &self.vehicles[active];
        let step_d = (v.pos - before).length();
        self.path_m += step_d;
        if v.kind != Kind::Plane && v.speed.abs() > 5.0 && step_d < 1e-4 {
            self.bumps += 1;
        }

        self.win_t += dt;
        if self.win_t >= 1.0 {
            let straight = (self.vehicles[active].pos - self.win_start).length();
            self.straight_kmh = straight / self.win_t * 3.6;
            self.path_kmh = self.path_m / self.win_t * 3.6;
            self.bumps_last = self.bumps;
            self.win_start = self.vehicles[active].pos;
            self.win_t = 0.0;
            self.path_m = 0.0;
            self.bumps = 0;
        }

        self.try_fire(&c);
        blasts.extend(combat::step_shells(&mut self.shells, dt, &self.city));

        for n in &mut self.traffic {
            n.update(dt);
        }
        self.refill_traffic(dt);
        combat::step_particles(&mut self.particles, dt);

        for p in blasts {
            self.explode(p);
        }

        self.cam.update(dt, &self.vehicles[self.active]);
        self.notice_t -= dt;
    }

    fn handle_switch_and_reset(&mut self, e: &Edges) {
        let n = self.vehicles.len();
        let mut target = e.switch;
        if e.next {
            target = Some((self.active + 1) % n);
        }
        if e.prev {
            target = Some((self.active + n - 1) % n);
        }
        if let Some(i) = target {
            if i != self.active {
                self.active = i;
                self.win_start = self.vehicles[i].pos;
                self.win_t = 0.0;
                self.path_m = 0.0;
                self.bumps = 0;
                self.bumps_last = 0;
                self.straight_kmh = 0.0;
                self.path_kmh = 0.0;
                let name = self.vehicles[i].name;
                self.notify(&format!("Đã lên: {name}"), 2.0);
            }
        }
        if e.reset {
            let i = self.active;
            self.vehicles[i].reset(&self.city);
            let name = self.vehicles[i].name;
            self.notify(&format!("Đã hồi phục {name}"), 2.0);
        }
    }

    fn try_fire(&mut self, c: &Controls) {
        let v = &mut self.vehicles[self.active];
        if v.kind != Kind::Tank || !c.fire || v.cool > 0.0 {
            return;
        }
        v.cool = 0.8;
        let dir = vec3(v.heading.sin(), 0.0, v.heading.cos());
        let muzzle = v.pos + dir * 3.6 + vec3(0.0, 1.5, 0.0);
        self.shells.push(Shell::new(muzzle, dir * 90.0 + vec3(0.0, 6.0, 0.0)));
        combat::burst(&mut self.particles, muzzle, 6, 4.0);
    }

    fn refill_traffic(&mut self, dt: f32) {
        self.traffic_timer -= dt;
        if self.traffic_timer > 0.0 {
            return;
        }
        self.traffic_timer = 1.5;
        if self.traffic.len() < TRAFFIC_COUNT {
            self.traffic.push(NpcCar::random());
        }
    }

    fn explode(&mut self, p: Vec3) {
        combat::burst(&mut self.particles, p, 30, 18.0);
        for (center, _) in self.city.damage(p.xz()) {
            combat::burst(&mut self.particles, center, 40, 14.0);
        }
        self.traffic
            .retain(|n| n.position().distance(p) >= BLAST_RADIUS);
    }

    pub fn draw(&self) {
        clear_background(Color::new(0.55, 0.8, 1.0, 1.0));
        set_camera(&self.cam.camera());

        for m in &self.city.roads {
            draw_mesh(m);
        }

        // Bóng đổ chỉ tính cho vật thể có thể nhìn thấy (hoặc bóng của nó nằm trong tầm nhìn)
        let mut shadows: Vec<Vertex> = Vec::new();
        for b in &self.city.buildings {
            let (c, r) = building_bounds(b);
            if self.cam.visible(c, r + SHADOW_MARGIN) {
                shadows.extend_from_slice(&b.shadow);
            }
        }
        for v in &self.vehicles {
            let model = self.models.of(v.kind);
            if model.parts.is_empty() {
                continue;
            }
            let (c, r) = vehicle_bounds(v);
            if !self.cam.visible(c, r + SHADOW_MARGIN) {
                continue;
            }
            let (mn, mx) = model.bounds();
            let m = v.transform();
            push_shadow(&mut shadows, box_corners(mn, mx).map(|p| m.transform_point3(p)));
        }
        let (cmn, cmx) = self.models.car().bounds();
        for n in &self.traffic {
            let center = n.position() + vec3(0.0, 1.0, 0.0);
            if !self.cam.visible(center, 4.0 + SHADOW_MARGIN) {
                continue;
            }
            let m = n.transform();
            push_shadow(&mut shadows, box_corners(cmn, cmx).map(|p| m.transform_point3(p)));
        }
        draw_verts(shadows);

        // Nhà: chỉ gửi lên GPU khi nằm trong tầm nhìn
        for b in &self.city.buildings {
            let (c, r) = building_bounds(b);
            if !self.cam.visible(c, r) {
                continue;
            }
            for m in &b.meshes {
                draw_mesh(m);
            }
        }

        let mut verts = Vec::new();
        for n in &self.traffic {
            if self.cam.visible(n.position() + vec3(0.0, 1.0, 0.0), 4.0) {
                push_model(&mut verts, &self.models.car().parts, n.transform());
            }
        }
        for v in &self.vehicles {
            let (c, r) = vehicle_bounds(v);
            if !self.cam.visible(c, r) {
                continue;
            }
            let model = self.models.of(v.kind);
            if model.parts.is_empty() {
                push_box(&mut verts, v.pos + vec3(0.0, 1.0, 0.0), vec3(2.2, 1.6, 4.2), [0.9, 0.2, 0.2]);
            } else {
                push_model(&mut verts, &model.parts, v.transform());
            }
        }
        for s in &self.shells {
            push_box(&mut verts, s.pos, Vec3::splat(0.7), [1.0, 0.8, 0.2]);
        }
        for p in &self.particles {
            push_box(&mut verts, p.pos, Vec3::splat(0.7), p.rgb);
        }
        draw_verts(verts);

        set_default_camera();
        self.draw_hud();
        minimap::draw(&self.city, &self.vehicles, self.active, &self.traffic);
    }

    /// Thông tin của phương tiện đang lái, dùng cho cả hai chế độ
    fn vehicle_lines(&self, v: &Vehicle) -> Vec<String> {
        let mut lines = vec![v.name.to_string(), format!("Tốc độ: {:.0} km/h", v.kmh.abs())];
        match v.kind {
            Kind::Plane => {
                lines.push(format!("Độ cao: {:.0} m", v.pos.y));
                lines.push(format!("Ga: {:.0}%", v.throttle * 100.0));
            }
            Kind::Tank => {
                let s = if v.cool > 0.0 { "Súng: nạp đạn..." } else { "Súng: sẵn sàng" };
                lines.push(s.to_string());
                lines.push(gear_text(v));
            }
            _ => lines.push(gear_text(v)),
        }
        lines
    }

    /// Thông tin hiện trên Discord Rich Presence
    pub fn status(&self) -> (String, String) {
        let v = &self.vehicles[self.active];
        (format!("Đang lái {}", v.name), format!("Tốc độ {:.0} km/h", v.kmh.abs()))
    }
    fn draw_hud(&self) {
        let font = self.font.as_ref();
        let v = &self.vehicles[self.active];
        let mut lines: Vec<String> = Vec::new();

        if self.dev {
            lines.push("[DEV MODE]".to_string());
            lines.push(format!("The Skyline v{}", env!("CARGO_PKG_VERSION")));
            lines.push(format!("FPS: {}   TPS: {}   Frame: {:.1} ms", get_fps(), self.tps, get_frame_time() * 1000.0));
            lines.push(String::new());
            lines.extend(self.vehicle_lines(v));
            lines.push(format!("Đường đi thực: {:.0} km/h", self.path_kmh));
            lines.push(format!("Đo thẳng: {:.0} km/h", self.straight_kmh));
            lines.push(format!("Va chạm/giây: {}", self.bumps_last));
        } else {
            lines.extend(self.vehicle_lines(v));
        }

        let switch: Vec<String> = self
            .vehicles
            .iter()
            .enumerate()
            .map(|(i, x)| {
                if i == self.active {
                    format!("[{}]>{}<", i + 1, x.name)
                } else {
                    format!("[{}] {}", i + 1, x.name)
                }
            })
            .collect();
        lines.push(String::new());
        lines.push(switch.join("  "));

        hud::draw_panel(font, 12.0, 12.0, &lines, 18);
        hud::draw_help(font);
        if self.notice_t > 0.0 {
            hud::draw_notice(font, &self.notice);
        }
    }
}

/// Dòng hiển thị số hiện tại
fn gear_text(v: &Vehicle) -> String {
    let mode = if v.auto_gear { "tự động" } else { "thủ công" };
    format!("Số: {} ({})", v.gear + 1, mode)
}