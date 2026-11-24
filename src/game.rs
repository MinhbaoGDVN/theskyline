use macroquad::prelude::*;

use crate::camera::ChaseCam;
use crate::civilians::{Civilian, CIV_TARGET, CORPSE_TIME};
use crate::police::{Arrest, Cop, Kind as CopKind};
use crate::city::{Building, City};
use crate::combat::{self, Particle, Shell};
use crate::config::{rnd, BLAST_RADIUS, BLOCK, EXT, GRID_N, TRAFFIC_COUNT};
use crate::hud;
use crate::input::{self, Controls, Edges};
use crate::minimap;
use crate::models::Models;
use crate::player::{Player, MAX_HP, MAX_STAMINA};
use crate::render::{draw_verts, push_box, push_model};
use crate::shadow::{box_corners, draw_shadows, push_shadow};
use crate::traffic::NpcCar;
use crate::vehicle::{default_fleet, Kind, Vehicle};

/// Bóng đổ có thể vươn xa hơn vật thể, nên phải kiểm tra thêm khoảng này
const SHADOW_MARGIN: f32 = 120.0;
/// Băng đạn súng lục
const MAG: u32 = 20;
/// Tiền phạt khi bị bắt
const FINE: u32 = 500;
/// Thời gian để trả nợ phạt (giây)
const DEBT_TIME: f32 = 900.0;
/// Thời gian bị giam nếu không trả nợ (giây)
const JAIL_TIME: f32 = 300.0;
/// Vị trí đồn công an (tâm khối nhà)
fn station() -> Vec3 {
    let c = crate::map::block(2, -3);
    vec3(c.x, 0.0, c.y)
}

/// Cửa đồn: điểm trên đường lưới gần đồn nhất (xe dừng ở đây, không lao vào nhà)
fn station_door() -> Vec3 {
    let s = station();
    vec3((s.x / BLOCK).round() * BLOCK, 0.0, s.z)
}

/// Số đồng tiền tối đa trên đường
const COIN_TARGET: usize = 40;
/// Giá trị mỗi đồng tiền
const COIN_VALUE: u32 = 10;

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

/// Một đồng tiền ngẫu nhiên nằm trên một đường lưới
fn random_coin() -> Vec3 {
    let k = (rnd(0.0, (2 * GRID_N + 1) as f32).floor() as i32 - GRID_N) as f32 * BLOCK;
    let t = rnd(-EXT, EXT);
    if rnd(0.0, 1.0) < 0.5 {
        vec3(k, 0.8, t)
    } else {
        vec3(t, 0.8, k)
    }
}

pub struct Game {
    models: Models,
    city: City,
    vehicles: Vec<Vehicle>,
    active: usize,
    /// true: đang lái phương tiện `active`; false: đang đi bộ
    riding: bool,
    player: Player,
    coins: Vec<Vec3>,
    coin_timer: f32,
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
    weather: crate::weather::Weather,
    alpha: f32,
    civs: Vec<Civilian>,
    civ_timer: f32,
    ammo: u32,
    reload_t: f32,
    fire_cd: f32,
    stars: u32,
    star_t: f32,
    gun_equipped: bool,
    inv_open: bool,
    tracers: Vec<(Vec3, Vec3, f32)>,
    cops: Vec<Cop>,
    cop_timer: f32,
    arrest: Arrest,
    debt: u32,
    debt_t: f32,
    jail_t: f32,
}

impl Game {
    pub fn new(models: Models, font: Option<Font>) -> Self {
        let vehicles = default_fleet();
        let win_start = vehicles[0].pos;
        let player = Player::new(vehicles[0].pos + vec3(3.0, 0.0, 0.0));
        Self {
            models,
            city: {
                let mut c = City::generate();
                crate::zones::add_zones(&mut c);
                crate::map::specials(&mut c);
                c
            },
            vehicles,
            active: 0,
            riding: false,
            player,
            coins: (0..COIN_TARGET).map(|_| random_coin()).collect(),
            coin_timer: 0.0,
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
            weather: crate::weather::Weather::new(),
            alpha: 1.0,
            civs: (0..CIV_TARGET).map(|_| Civilian::random_spawn()).collect(),
            civ_timer: 0.0,
            ammo: MAG,
            reload_t: 0.0,
            fire_cd: 0.0,
            stars: 0,
            star_t: 0.0,
            gun_equipped: false,
            inv_open: false,
            tracers: Vec::new(),
            cops: Vec::new(),
            cop_timer: 0.0,
            arrest: Arrest::Free,
            debt: 0,
            debt_t: 0.0,
            jail_t: 0.0,
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
        self.weather.update(dt);
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

        let mut blasts: Vec<Vec3> = Vec::new();
        if self.arrest != Arrest::Free {
            self.update_arrest(dt);
        } else if self.riding {
            blasts = self.update_riding(dt, &e, &c);
        } else {
            self.update_on_foot(dt, &e);
            self.player.update(dt, c.up, c.down, c.left, c.right, c.run, &self.city);
            self.check_npc_hit_player();
        }

        self.update_civilians(dt);
        self.update_police(dt);
        blasts.extend(combat::step_shells(&mut self.shells, dt, &self.city));
        for n in &mut self.traffic {
            n.update(dt);
        }
        self.refill_traffic(dt);
        combat::step_particles(&mut self.particles, dt);
        for p in blasts {
            self.explode(p);
        }

        self.update_coins(dt);
        if self.player.hp <= 0.0 {
            self.respawn();
        }

        if self.riding {
            self.cam.update(dt, &self.vehicles[self.active]);
        }
        self.notice_t -= dt;
        self.update_gun_timers(dt);
        for tr in &mut self.tracers {
            tr.2 -= dt;
        }
        self.tracers.retain(|t| t.2 > 0.0);
    }

    /// Logic khi đang lái: phương tiện, bắn, va chạm. Trả về các điểm nổ cần xử lý.
    fn update_riding(&mut self, dt: f32, e: &Edges, c: &Controls) -> Vec<Vec3> {
        if e.enter {
            self.try_exit();
        }
        if e.reset {
            let i = self.active;
            self.vehicles[i].reset(&self.city);
            let name = self.vehicles[i].name;
            self.notify(&format!("Đã hồi phục {name}"), 2.0);
        }

        for v in &mut self.vehicles {
            v.cool = (v.cool - dt).max(0.0);
        }

        let active = self.active;
        let before = self.vehicles[active].pos;
        let crashed = self.vehicles[active].update(dt, c, &self.city);
        if crashed {
            let crash_pos = self.vehicles[active].pos + vec3(0.0, 1.0, 0.0);
            combat::burst(&mut self.particles, crash_pos, 30, 18.0);
            self.damage_player(30.0);
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

        self.try_fire(c);
        self.npc_collide();
        Vec::new()
    }

    /// Logic khi đi bộ: lên xe khi bấm E gần phương tiện
    fn update_on_foot(&mut self, _dt: f32, e: &Edges) {
        if e.enter {
            self.try_enter();
        }
        if e.inv {
            self.inv_open = !self.inv_open;
        }
        if e.equip {
            self.gun_equipped = !self.gun_equipped;
            let msg = if self.gun_equipped { "Đã trang bị súng lục" } else { "Đã cất súng lục" };
            self.notify(msg, 1.5);
        }
        if e.reset && self.gun_equipped && self.ammo < MAG && self.reload_t <= 0.0 {
            self.reload_t = 1.5;
        }
        if self.gun_equipped && e.fire {
            self.player_shoot();
        }
    }

    /// Công an: xuất hiện theo số sao, đuổi theo người chơi; bắt được thì chở về đồn
    fn update_police(&mut self, dt: f32) {
        if self.stars == 0 {
            self.cops.clear();
        } else {
            self.cop_timer -= dt;
            if self.cop_timer <= 0.0 {
                self.cop_timer = 2.0;
                let foot = self.cops.iter().filter(|c| c.kind == CopKind::Foot).count() as u32;
                let car = self.cops.iter().filter(|c| c.kind == CopKind::Car).count() as u32;
                let p = self.player.pos;
                if foot < self.stars * 2 {
                    self.cops.push(Cop::spawn(p, CopKind::Foot));
                } else if car < self.stars {
                    self.cops.push(Cop::spawn(p, CopKind::Car));
                }
            }
            let p = self.player.pos;
            for c in &mut self.cops {
                let speed = if c.kind == CopKind::Car { 14.0 } else { 6.0 };
                c.drive(dt, p, speed, &self.city);
            }
            if !self.riding && self.arrest == Arrest::Free {
                let caught = self
                    .cops
                    .iter()
                    .any(|c| vec2(c.pos.x - p.x, c.pos.z - p.z).length() < 3.0);
                if caught {
                    self.start_arrest();
                }
            }
        }
        if self.debt > 0 && self.arrest == Arrest::Free {
            self.debt_t -= dt;
            if self.debt_t <= 0.0 {
                self.jail();
            }
        }
    }

    /// Bị bắt: xe cảnh sát đưa về đồn, người chơi nằm trong xe (không dịch chuyển)
    fn start_arrest(&mut self) {
        self.arrest = Arrest::Carried;
        self.cops.clear();
        let p = self.player.pos;
        let mut car = Cop::spawn(p, CopKind::Car);
        car.pos = p + vec3(3.0, 0.0, 0.0);
        car.carrying = true;
        self.cops.push(car);
        self.notify("Bị bắt! Đang chở về đồn", 3.0);
    }

    fn update_arrest(&mut self, dt: f32) {
        match self.arrest {
            Arrest::Carried => {
                let door = station_door();
                let mut arrived = false;
                let mut at = self.player.pos;
                if let Some(car) = self.cops.iter_mut().find(|c| c.carrying) {
                    car.drive(dt, door, 14.0, &self.city);
                    at = car.pos;
                    arrived = vec2(car.pos.x - door.x, car.pos.z - door.z).length() < 6.0;
                } else {
                    arrived = true;
                }
                self.player.teleport(at);
                if arrived {
                    self.release_at_station();
                }
            }
            Arrest::Jailed => {
                self.jail_t -= dt;
                if self.jail_t <= 0.0 {
                    self.free_from_jail();
                }
            }
            Arrest::Free => {}
        }
    }

    /// Đến đồn: trừ phạt (không âm), phần còn thiếu thành nợ có thời hạn
    fn release_at_station(&mut self) {
        let pay = self.player.money.min(FINE);
        self.player.money -= pay;
        self.debt = FINE - pay;
        self.cops.clear();
        self.stars = 0;
        self.star_t = 0.0;
        self.arrest = Arrest::Free;
        self.riding = false;
        let door = station_door();
        self.player.teleport(door);
        if self.debt > 0 {
            self.debt_t = DEBT_TIME;
            let msg = format!("Nợ phạt {} $: nhặt tiền trong 15 phút", self.debt);
            self.notify(&msg, 4.0);
        } else {
            self.notify("Đã nộp phạt, được thả", 3.0);
        }
    }

    /// Hết thời hạn trả nợ: bị giam 5 phút
    fn jail(&mut self) {
        self.arrest = Arrest::Jailed;
        self.jail_t = JAIL_TIME;
        self.cops.clear();
        self.stars = 0;
        self.star_t = 0.0;
        let st = station();
        self.player.teleport(vec3(st.x, 0.0, st.z));
        self.notify("Hết hạn trả nợ, bị giam 5 phút", 4.0);
    }

    fn free_from_jail(&mut self) {
        self.arrest = Arrest::Free;
        self.debt = 0;
        self.debt_t = 0.0;
        let st = station();
        self.player.teleport(vec3(st.x, 0.0, st.z + 14.0));
        self.notify("Đã ra tù", 3.0);
    }

    /// Dân thường đi lại quanh người chơi; xác nằm lại rồi biến mất; xa quá thì bỏ
    fn update_civilians(&mut self, dt: f32) {
        for c in &mut self.civs {
            c.update(dt, &self.city);
        }
        let p = self.player.pos;
        self.civs.retain(|c| {
            if c.dead {
                c.dead_t < CORPSE_TIME
            } else {
                (c.pos - p).length() < 230.0
            }
        });
        self.civ_timer -= dt;
        if self.civ_timer <= 0.0 {
            self.civ_timer = 0.5;
            let alive = self.civs.iter().filter(|c| !c.dead).count();
            if alive < CIV_TARGET {
                for _ in 0..5 {
                    let c = Civilian::spawn_near(p);
                    let d = (c.pos - p).length();
                    if d > 25.0 && d < 160.0 && !self.city.hit(vec2(c.pos.x, c.pos.z), 0.5, 1.0) {
                        self.civs.push(c);
                        break;
                    }
                }
            }
        }
    }

    fn update_gun_timers(&mut self, dt: f32) {
        self.fire_cd = (self.fire_cd - dt).max(0.0);
        if self.reload_t > 0.0 {
            self.reload_t -= dt;
            if self.reload_t <= 0.0 {
                self.reload_t = 0.0;
                self.ammo = MAG;
            }
        }
        if self.star_t > 0.0 {
            self.star_t -= dt;
            if self.star_t <= 0.0 && self.stars > 0 {
                self.stars -= 1;
                self.star_t = if self.stars > 0 { 40.0 } else { 0.0 };
            }
        }
    }

    /// Tăng sao truy nã (tối đa 5), mỗi lần đều làm mới thời gian giảm sao
    fn add_wanted(&mut self, n: u32) {
        self.stars = (self.stars + n).min(5);
        self.star_t = 40.0;
    }

    /// Khoảng cách tới tường đầu tiên dọc tia bắn (nếu có)
    fn ray_wall(&self, o: Vec3, d: Vec3, range: f32) -> Option<f32> {
        let mut t = 0.5;
        while t < range {
            let p = o + d * t;
            if self.city.hit(vec2(p.x, p.z), 0.05, p.y) {
                return Some(t);
            }
            t += 0.5;
        }
        None
    }

    /// Tia bắn từ camera qua con trỏ chuột: (điểm gốc, hướng chuẩn hoá)
    fn aim_ray(&self) -> (Vec3, Vec3) {
        let cam = self.cam.camera();
        let inv = cam.matrix().inverse();
        let (mx, my) = mouse_position();
        let x = mx / screen_width() * 2.0 - 1.0;
        let y = 1.0 - my / screen_height() * 2.0;
        let far = inv * vec4(x, y, 1.0, 1.0);
        let far = far.truncate() / far.w;
        let dir = (far - cam.position).normalize_or_zero();
        (cam.position, dir)
    }

    /// Bắn một viên về phía con trỏ chuột: trúng dân thường đầu tiên trên tia (nếu không bị tường chắn)
    fn player_shoot(&mut self) {
        if !self.gun_equipped || self.ammo == 0 || self.reload_t > 0.0 || self.fire_cd > 0.0 {
            return;
        }
        self.ammo -= 1;
        self.fire_cd = 0.25;
        let h = self.player.heading;
        let f = vec3(h.sin(), 0.0, h.cos());
        let r = vec3(h.cos(), 0.0, -h.sin());
        let muzzle = self.player.pos + vec3(0.0, 1.25, 0.0) + f * 0.5 + r * 0.36;
        let (_, d) = self.aim_ray();
        let range = 400.0;
        let wall = self.ray_wall(muzzle, d, range);
        let mut best: Option<(usize, f32)> = None;
        for (i, c) in self.civs.iter().enumerate() {
            if let Some(t) = c.ray_hit(muzzle, d, range) {
                if best.map_or(true, |(_, bt)| t < bt) {
                    best = Some((i, t));
                }
            }
        }
        let mut end = match wall {
            Some(w) => muzzle + d * w,
            None => muzzle + d * range,
        };
        if let Some((i, t)) = best {
            if wall.map_or(true, |w| t < w) {
                end = muzzle + d * t;
                self.civs[i].dead = true;
                self.civs[i].dead_t = 0.0;
                combat::burst(&mut self.particles, end, 15, 8.0);
                self.add_wanted(1);
                self.notify("Bạn vừa bắn dân thường! Bị truy nã", 2.5);
            }
        }
        self.tracers.push((muzzle, end, 0.08));
        combat::burst(&mut self.particles, muzzle, 4, 3.0);
    }


    /// Lên xe gần nhất trong bán kính 6 m (máy bay phải đang nằm trên đất)
    fn try_enter(&mut self) {
        let p = self.player.pos;
        let mut best: Option<(usize, f32)> = None;
        for (i, v) in self.vehicles.iter().enumerate() {
            if v.pos.y > 1.5 {
                continue;
            }
            let d = vec2(v.pos.x - p.x, v.pos.z - p.z).length();
            if d < 6.0 && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        match best {
            Some((i, _)) => {
                self.active = i;
                self.riding = true;
                self.win_start = self.vehicles[i].pos;
                self.win_t = 0.0;
                self.path_m = 0.0;
                self.bumps = 0;
                self.bumps_last = 0;
                let name = self.vehicles[i].name;
                self.notify(&format!("Đã lên: {name} (E để xuống)"), 2.0);
            }
            None => self.notify("Lại gần xe để lên", 1.5),
        }
    }

    /// Xuống xe: phải dừng hẳn và đang ở trên đất
    fn try_exit(&mut self) {
        let i = self.active;
        if self.vehicles[i].pos.y > 1.5 {
            self.notify("Hạ cánh trước khi xuống", 1.5);
            return;
        }
        if self.vehicles[i].speed.abs() > 2.0 {
            self.notify("Dừng xe trước khi xuống", 1.5);
            return;
        }
        let v = &mut self.vehicles[i];
        v.kmh = 0.0;
        v.speed = 0.0;
        let right = vec3(v.heading.cos(), 0.0, -v.heading.sin());
        let spot = v.pos + right * 2.5;
        let heading = v.heading;
        self.player.pos = spot;
        self.player.heading = heading;
        self.riding = false;
        self.notify("Đã xuống xe", 1.5);
    }

    /// Người chơi bị trừ máu
    fn damage_player(&mut self, amount: f32) {
        self.player.hp -= amount;
    }

    /// Xe NPC đâm người đi bộ
    fn check_npc_hit_player(&mut self) {
        if self.player.hurt_cd > 0.0 {
            return;
        }
        let p = self.player.pos;
        let hit = self
            .traffic
            .iter()
            .any(|n| {
                let q = n.position();
                vec2(q.x - p.x, q.z - p.z).length() < 2.5
            });
        if hit {
            self.damage_player(20.0);
            self.player.hurt_cd = 1.0;
            self.notify("Bị xe đâm!", 1.2);
        }
    }

    /// Hồi sinh ở trạm xá gần nhất (tìm chỗ trống quanh trạm)
    fn respawn(&mut self) {
        let c = crate::map::block(-3, -2);
        let candidates = [
            vec2(0.0, 22.0),
            vec2(0.0, -22.0),
            vec2(22.0, 0.0),
            vec2(-22.0, 0.0),
            vec2(0.0, 40.0),
        ];
        let mut spot = c;
        for o in candidates {
            let q = c + o;
            if !self.city.hit(q, 1.0, 1.0) {
                spot = q;
                break;
            }
        }
        self.player.pos = vec3(spot.x, 0.0, spot.y);
        self.player.hp = MAX_HP;
        self.player.stamina = MAX_STAMINA;
        self.player.hurt_cd = 0.0;
        self.riding = false;
        self.notify("Bạn được đưa đến trạm xá", 3.0);
    }

    /// Đồng tiền: tự xuất hiện trên đường, nhặt khi đi bộ đến gần
    fn update_coins(&mut self, dt: f32) {
        self.coin_timer -= dt;
        if self.coin_timer <= 0.0 {
            self.coin_timer = 2.0;
            if self.coins.len() < COIN_TARGET {
                self.coins.push(random_coin());
            }
        }
        if self.riding {
            return;
        }
        let p = self.player.pos;
        let mut got: u32 = 0;
        self.coins.retain(|c| {
            let d = vec2(c.x - p.x, c.z - p.z).length();
            if d < 2.2 {
                got += 1;
                false
            } else {
                true
            }
        });
        if got > 0 {
            let gain = got * COIN_VALUE;
            self.player.money += gain;
            if self.debt > 0 {
                let pay = self.debt.min(self.player.money);
                self.player.money -= pay;
                self.debt -= pay;
                if self.debt == 0 {
                    self.debt_t = 0.0;
                    self.notify("Đã trả hết nợ phạt", 2.5);
                }
            }
            self.notify(&format!("+{gain} $"), 1.0);
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

    /// Va chạm xe mặt đất với NPC: xe bật lại, NPC bị hất văng kèm hiệu ứng
    fn npc_collide(&mut self) {
        let active = self.active;
        if self.vehicles[active].kind == Kind::Plane {
            return;
        }
        let vpos = self.vehicles[active].pos;
        const HIT_DIST: f32 = 3.6;
        let mut hit: Option<(usize, Vec3, f32)> = None;
        for (i, n) in self.traffic.iter().enumerate() {
            let np = n.position();
            let d = vec2(np.x - vpos.x, np.z - vpos.z);
            let dist = d.length();
            if dist < HIT_DIST {
                hit = Some((i, np, dist));
                break;
            }
        }
        if let Some((i, np, dist)) = hit {
            let d = vec2(np.x - vpos.x, np.z - vpos.z);
            let dir = if dist > 1e-3 { d / dist } else { vec2(1.0, 0.0) };
            let v = &mut self.vehicles[active];
            // Đẩy xe ra khỏi NPC và bật ngược lại
            v.pos.x -= dir.x * (HIT_DIST - dist);
            v.pos.z -= dir.y * (HIT_DIST - dist);
            v.kmh *= -0.25;
            self.traffic.swap_remove(i);
            combat::burst(&mut self.particles, np + vec3(0.0, 1.0, 0.0), 20, 10.0);
            self.damage_player(15.0);
            self.notify("Tông xe!", 1.0);
        }
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

    pub fn draw(&mut self) {
        if !self.riding {
            let (p, h) = self.player.render_state(self.alpha);
            self.cam.follow(get_frame_time().min(0.05), p, h);
        }
        self.weather.draw_sky(&self.cam.camera());
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
        draw_shadows(shadows);

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
        for c in &self.cops {
            if self.cam.visible(c.pos, 3.0) {
                c.draw(&mut verts, &self.models);
            }
        }
        if !self.riding && self.arrest == Arrest::Free {
            let (_, aim) = self.aim_ray();
            let pitch = aim.y.clamp(-1.0, 1.0).asin();
            self.player.draw(&mut verts, self.alpha, self.gun_equipped, pitch);
        }
        for (a, b, _) in &self.tracers {
            draw_line_3d(*a, *b, Color::new(1.0, 0.92, 0.5, 1.0));
        }
        for civ in &self.civs {
            if self.cam.visible(civ.pos, 2.0) {
                civ.draw(&mut verts);
            }
        }
        for c in &self.coins {
            if self.cam.visible(*c, 2.0) {
                push_box(&mut verts, *c, vec3(0.9, 0.9, 0.25), [1.0, 0.8, 0.2]);
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
        self.weather.draw_overlay();
        self.draw_hud();
        self.weather.draw_notice(self.font.as_ref());
        minimap::draw(&self.city, &self.vehicles, self.active, &self.traffic);
        if !self.riding {
            let (ms, mm) = (200.0_f32, 14.0_f32);
            let x0 = screen_width() - ms - mm;
            let y0 = mm;
            let sc = ms / (2.0 * crate::config::EXT);
            let p = self.player.pos;
            let px = x0 + (p.x.clamp(-crate::config::EXT, crate::config::EXT) + crate::config::EXT) * sc;
            let py = y0 + (p.z.clamp(-crate::config::EXT, crate::config::EXT) + crate::config::EXT) * sc;
            draw_circle(px, py, 4.0, GREEN);
            draw_circle_lines(px, py, 6.0, 1.5, WHITE);
        }
        if self.inv_open && !self.riding {
            self.draw_inventory();
        }
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

    /// Dang dieu khien phuong tien (dung cho am thanh buong lai va con)
    pub fn active_can_honk(&self) -> bool {
        self.riding && matches!(self.vehicles[self.active].kind, Kind::Car | Kind::Bike)
    }

    /// Không còn đổi xe bằng số; giữ hàm để main không lỗi
    pub fn select_vehicle(&mut self, _i: usize) {}

    pub fn active_is_plane(&self) -> bool {
        self.riding && self.vehicles[self.active].kind == Kind::Plane
    }

    pub fn status(&self) -> (String, String) {
        if self.riding {
            let v = &self.vehicles[self.active];
            (format!("Đang lái {}", v.name), format!("Tốc độ {:.0} km/h", v.kmh.abs()))
        } else {
            ("Đang đi bộ".to_string(), format!("Tiền: {} $", self.player.money))
        }
    }

    /// Alpha nội suy của frame hiện tại (0..1), main gọi trước khi vẽ
    pub fn set_alpha(&mut self, a: f32) {
        self.alpha = a;
    }

    /// Báo cho main biết nhân vật vừa bước một bước (để phát tiếng bước chân)
    pub fn take_step(&mut self) -> bool {
        self.player.take_step()
    }

    fn draw_hud(&self) {
        let font = self.font.as_ref();
        let mut lines: Vec<String> = Vec::new();

        if self.dev {
            lines.push("[DEV MODE]".to_string());
            lines.push(format!("The Skyline v{}", env!("CARGO_PKG_VERSION")));
            lines.push(format!("FPS: {}   TPS: {}   Frame: {:.1} ms", get_fps(), self.tps, get_frame_time() * 1000.0));
            lines.push(String::new());
        }
        if self.riding {
            let v = &self.vehicles[self.active];
            lines.extend(self.vehicle_lines(v));
            if self.dev {
                lines.push(format!("Đường đi thực: {:.0} km/h", self.path_kmh));
                lines.push(format!("Đo thẳng: {:.0} km/h", self.straight_kmh));
                lines.push(format!("Va chạm/giây: {}", self.bumps_last));
            }
        } else {
            lines.push("Đi bộ".to_string());
            let gun = if self.reload_t > 0.0 { "Đang nạp...".to_string() } else { format!("{}/{}", self.ammo, MAG) };
            if self.gun_equipped {
                lines.push(format!("Súng: {gun}"));
            }
        }
        lines.push(format!("Tiền: {} $", self.player.money));
        match self.arrest {
            Arrest::Carried => lines.push("Đang bị đưa về đồn".to_string()),
            Arrest::Jailed => lines.push(format!("Đang bị giam: {:.0} s", self.jail_t)),
            Arrest::Free => {}
        }
        if self.debt > 0 {
            lines.push(format!("Nợ phạt: {} $ ({:.0} s)", self.debt, self.debt_t));
        }
        if self.stars > 0 {
            lines.push(format!("Truy nã: {}/5 sao", self.stars));
        }

        hud::draw_panel(font, 12.0, 12.0, &lines, 18);
        hud::draw_help(font);
        self.draw_bars(font);
        if self.notice_t > 0.0 {
            hud::draw_notice(font, &self.notice);
        }
    }

    /// Kho đồ (I): súng lục là một món riêng, nhấn 1 để trang bị hoặc cất
    fn draw_inventory(&self) {
        let font = self.font.as_ref();
        let w = 440.0;
        let h = 200.0;
        let x = (screen_width() - w) / 2.0;
        let y = (screen_height() - h) / 2.0;
        draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.7));
        draw_rectangle(x, y, w, 3.0, Color::new(0.97, 0.72, 0.16, 1.0));
        draw_text_ex("KHO ĐỒ (I để đóng)", x + 16.0, y + 34.0, TextParams { font, font_size: 24, color: WHITE, ..Default::default() });
        let status = if self.gun_equipped { "ĐANG TRANG BỊ" } else { "đang cất" };
        let row = format!("Súng lục: {}/{} viên  [{}]", self.ammo, MAG, status);
        draw_text_ex(&row, x + 16.0, y + 84.0, TextParams { font, font_size: 22, color: WHITE, ..Default::default() });
        draw_text_ex("Nhấn 1 để trang bị / cất súng", x + 16.0, y + 124.0, TextParams { font, font_size: 18, color: Color::new(0.8, 0.85, 0.9, 1.0), ..Default::default() });
        draw_text_ex(&format!("Tiền: {} $", self.player.money), x + 16.0, y + 164.0, TextParams { font, font_size: 20, color: Color::new(1.0, 0.85, 0.3, 1.0), ..Default::default() });
    }

    /// Thanh máu và thể lực ở góc dưới trái (có khung, có số)
    fn draw_bars(&self, font: Option<&Font>) {
        let w = 260.0;
        let h = 22.0;
        let x = 12.0;
        let y = screen_height() - 26.0 - 74.0;
        draw_rectangle(x - 8.0, y - 8.0, w + 16.0, 2.0 * h + 30.0 + 8.0, Color::new(0.0, 0.0, 0.0, 0.45));
        draw_rectangle(x - 8.0, y - 8.0, w + 16.0, 3.0, Color::new(0.97, 0.72, 0.16, 1.0));
        draw_bar(font, x, y, w, h, self.player.hp / MAX_HP, Color::new(0.85, 0.2, 0.2, 1.0), "Máu", &format!("{:.0}/{:.0}", self.player.hp.max(0.0), MAX_HP));
        draw_bar(font, x, y + h + 14.0, w, h, self.player.stamina / MAX_STAMINA, Color::new(0.25, 0.8, 0.35, 1.0), "Thể lực", &format!("{:.0}/{:.0}", self.player.stamina, MAX_STAMINA));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_bar(font: Option<&Font>, x: f32, y: f32, w: f32, h: f32, frac: f32, color: Color, label: &str, value: &str) {
    draw_text_ex(
        label,
        x,
        y + h * 0.5 + 6.0,
        TextParams {
            font,
            font_size: 18,
            color: WHITE,
            ..Default::default()
        },
    );
    let bx = x + 84.0;
    let bw = w - 84.0;
    draw_rectangle(bx, y, bw, h, Color::new(0.0, 0.0, 0.0, 0.6));
    draw_rectangle(bx, y, bw * frac.clamp(0.0, 1.0), h, color);
    draw_rectangle(bx, y, bw * frac.clamp(0.0, 1.0), h * 0.45, Color::new(1.0, 1.0, 1.0, 0.18));
    draw_rectangle_lines(bx, y, bw, h, 2.0, Color::new(0.6, 0.65, 0.75, 1.0));
    let d = measure_text(value, font, 16, 1.0);
    draw_text_ex(
        value,
        bx + (bw - d.width) / 2.0,
        y + h * 0.5 + 5.0,
        TextParams {
            font,
            font_size: 16,
            color: WHITE,
            ..Default::default()
        },
    );
}

/// Dòng hiển thị số hiện tại
fn gear_text(v: &Vehicle) -> String {
    let mode = if v.auto_gear { "tự động" } else { "thủ công" };
    format!("Số: {} ({})", v.gear + 1, mode)
}
