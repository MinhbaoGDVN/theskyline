use macroquad::prelude::*;
use macroquad::rand::gen_range;

use std::sync::atomic::{AtomicBool, Ordering};

use crate::render::light_dir;

/// Cài đặt: có hiện thông báo thời tiết ở góc trái màn hình không (luôn hiện khi bật)
pub static SHOW_NOTICE: AtomicBool = AtomicBool::new(true);

pub fn show_notice() -> bool {
    SHOW_NOTICE.load(Ordering::Relaxed)
}

pub fn set_show_notice(v: bool) {
    SHOW_NOTICE.store(v, Ordering::Relaxed);
}
use std::f32::consts::TAU;

/// Một ngày đêm (giây thực)
const DAY_LEN: f32 = 240.0;
/// Mỗi kiểu thời tiết kéo dài (giây thực)
const WEATHER_LEN: f32 = 90.0;
/// Thời gian hiện thông báo thời tiết
const NOTICE_TIME: f32 = 4.0;
const DROPS: usize = 180;

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Clear,
    Cloudy,
    Rain,
    Fog,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Clear => "Trời quang",
            Kind::Cloudy => "Nhiều mây",
            Kind::Rain => "Mưa",
            Kind::Fog => "Sương mù",
        }
    }

    /// Mức xám hoá của bầu trời (0 = trong xanh)
    fn grey(self) -> f32 {
        match self {
            Kind::Clear => 0.0,
            Kind::Cloudy => 0.35,
            Kind::Rain => 0.55,
            Kind::Fog => 0.8,
        }
    }
}

pub struct Weather {
    /// Thời điểm trong ngày, 0..DAY_LEN (0 = nửa đêm, DAY_LEN/2 = trưa)
    time: f32,
    kind: Kind,
    kind_t: f32,
    notice_t: f32,
    /// Hạt mưa: (x, y, tốc độ), x và y theo tỉ lệ 0..1 của màn hình
    drops: Vec<[f32; 3]>,
}

impl Weather {
    pub fn new() -> Self {
        let drops = (0..DROPS)
            .map(|_| [gen_range(0.0, 1.0), gen_range(0.0, 1.0), gen_range(1.2, 1.8)])
            .collect();
        Self {
            time: DAY_LEN * 0.35,
            kind: Kind::Clear,
            kind_t: 0.0,
            notice_t: NOTICE_TIME,
            drops,
        }
    }

    fn pick_kind(&self) -> Kind {
        let r = gen_range(0.0, 1.0);
        let k = if r < 0.35 {
            Kind::Clear
        } else if r < 0.6 {
            Kind::Cloudy
        } else if r < 0.85 {
            Kind::Rain
        } else {
            Kind::Fog
        };
        k
    }

    pub fn update(&mut self, dt: f32) {
        self.time = (self.time + dt) % DAY_LEN;
        self.kind_t += dt;
        if self.kind_t >= WEATHER_LEN {
            self.kind_t = 0.0;
            let next = self.pick_kind();
            if next != self.kind {
                self.kind = next;
                self.notice_t = NOTICE_TIME;
            }
        }
        self.notice_t = (self.notice_t - dt).max(0.0);
        for d in &mut self.drops {
            d[1] += d[2] * dt;
            if d[1] > 1.0 {
                d[1] -= 1.0;
                d[0] = gen_range(0.0, 1.0);
            }
        }
    }

    /// Độ cao mặt trời: -1 (nửa đêm) đến 1 (trưa)
    fn elevation(&self) -> f32 {
        -(TAU * self.time / DAY_LEN).cos()
    }

    /// Xoá nền bằng màu trời (theo giờ và thời tiết) rồi vẽ mặt trời
    pub fn draw_sky(&self, cam: &Camera3D) {
        let e = self.elevation();
        let d = (e * 2.5 + 0.5).clamp(0.0, 1.0);

        let night = vec3(0.04, 0.06, 0.14);
        let day = vec3(0.55, 0.8, 1.0);
        let mut sky = night.lerp(day, d);
        let dusk = (1.0 - (e.abs() * 4.0).min(1.0)) * 0.6;
        sky = sky.lerp(vec3(0.95, 0.55, 0.35), dusk);
        let grey = vec3(0.62, 0.64, 0.68) * (0.4 + 0.6 * d);
        let g = self.kind.grey();
        sky = sky.lerp(grey, g);
        clear_background(Color::new(sky.x, sky.y, sky.z, 1.0));

        // Mặt trời đặt đối diện với hướng bóng đổ trên màn hình.
        // Nếu mặt trời thật nằm trong màn hình thì vẽ đúng vị trí đó.
        if e > -0.05 && self.kind != Kind::Fog {
            let a = (e * 6.0 + 0.3).clamp(0.0, 1.0) * (1.0 - self.kind.grey());
            let w = screen_width();
            let h = screen_height();
            let to_screen = |p: Vec3| -> Option<Vec2> {
                let c = cam.matrix() * vec4(p.x, p.y, p.z, 1.0);
                if c.w <= 0.0 {
                    None
                } else {
                    Some(vec2((c.x / c.w + 1.0) * 0.5 * w, (1.0 - c.y / c.w) * 0.5 * h))
                }
            };
            let margin = 60.0;

            // Đúng vị trí thật của mặt trời (nếu đang nằm trong màn hình)
            let far = cam.position + light_dir() * 5000.0;
            if let Some(p) = to_screen(far) {
                if p.x > margin && p.x < w - margin && p.y > margin && p.y < h - margin {
                    draw_sun(p.x, p.y, 1.0, a);
                    return;
                }
            }

            // Ngoài màn hình: hướng bóng đổ trên mặt đất quanh điểm nhìn, rồi lấy ngược lại
            let lxz = vec3(light_dir().x, 0.0, light_dir().z).normalize_or_zero();
            let g = vec3(cam.target.x, 0.0, cam.target.z);
            if let (Some(p0), Some(p1)) = (to_screen(g), to_screen(g - lxz * 50.0)) {
                let shadow_dir = (p1 - p0).normalize_or_zero();
                let sun_dir = -shadow_dir;
                if sun_dir.length_squared() > 0.0 {
                    let c = vec2(w * 0.5, h * 0.5);
                    let sx = ((w * 0.5 - margin) / sun_dir.x.abs().max(1e-4))
                        .min((h * 0.5 - margin) / sun_dir.y.abs().max(1e-4));
                    let p = c + sun_dir * sx;
                    draw_sun(p.x, p.y, 0.45, a * 0.9);
                }
            }
        }
    }

    /// Lớp phủ thời tiết vẽ lên trên cảnh 3D (mưa, sương)
    pub fn draw_overlay(&self) {
        let w = screen_width();
        let h = screen_height();
        match self.kind {
            Kind::Clear => {}
            Kind::Cloudy => draw_rectangle(0.0, 0.0, w, h, Color::new(0.5, 0.52, 0.56, 0.12)),
            Kind::Fog => draw_rectangle(0.0, 0.0, w, h, Color::new(0.8, 0.82, 0.85, 0.35)),
            Kind::Rain => {
                draw_rectangle(0.0, 0.0, w, h, Color::new(0.1, 0.14, 0.2, 0.12));
                for d in &self.drops {
                    let x = d[0] * w;
                    let y = d[1] * h;
                    draw_line(x, y, x - 5.0, y + 16.0, 1.5, Color::new(0.75, 0.85, 1.0, 0.5));
                }
            }
        }
    }

    /// Thông báo thời tiết ở góc trái màn hình, mờ dần ở giây cuối
    pub fn draw_notice(&self, font: Option<&Font>) {
        if !show_notice() {
            return;
        }
        let a = 1.0;
        let text = format!("Thời tiết: {}", self.kind.name());
        let size: u16 = 22;
        let tw = measure_text(&text, font, size, 1.0).width;
        let x = 12.0;
        let y = 185.0;
        draw_rectangle(x, y, tw + 20.0, 34.0, Color::new(0.0, 0.0, 0.0, 0.45 * a));
        draw_text_ex(
            &text,
            x + 10.0,
            y + 24.0,
            TextParams {
                font,
                font_size: size,
                color: Color::new(1.0, 1.0, 1.0, a),
                ..Default::default()
            },
        );
    }
}

/// Vẽ mặt trời tại (x, y) với tỉ lệ kích thước scale
fn draw_sun(x: f32, y: f32, scale: f32, a: f32) {
    draw_circle(x, y, 70.0 * scale, Color::new(1.0, 0.95, 0.7, 0.15 * a));
    draw_circle(x, y, 42.0 * scale, Color::new(1.0, 0.97, 0.75, 0.35 * a));
    draw_circle(x, y, 30.0 * scale, Color::new(1.0, 0.98, 0.85, a));
}
