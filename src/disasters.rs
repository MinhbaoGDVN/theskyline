use macroquad::prelude::*;

use crate::config::rnd;

/// Cứ mỗi phút kiểm tra một lần
pub const CHECK_EVERY: f32 = 60.0;
/// Xác suất có thiên tai mới trong mỗi lần kiểm tra
pub const CHANCE: f32 = 0.01;

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Fire,
    War,
    Storm,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Fire => "Hoả hoạn",
            Kind::War => "Chiến tranh",
            Kind::Storm => "Giông sét",
        }
    }

    /// Thời gian kéo dài (giây)
    pub fn duration(self) -> f32 {
        match self {
            Kind::Fire => 240.0,
            Kind::War => 1200.0,
            Kind::Storm => 900.0,
        }
    }
}

/// Một thiên tai đang diễn ra
pub struct Event {
    pub kind: Kind,
    pub left: f32,
    pub spot: Vec3,
    pub tick: f32,
    pub dmg_t: f32,
    pub flash: f32,
    /// Bom đang rơi: (vị trí, thời gian còn lại trước khi nổ)
    pub bombs: Vec<(Vec3, f32)>,
}

pub struct Disasters {
    pub event: Option<Event>,
    pub check_t: f32,
}

impl Disasters {
    pub fn new() -> Self {
        Self {
            event: None,
            check_t: CHECK_EVERY,
        }
    }

    /// Mỗi phút có 1% cơ hội có một thiên tai mới (nếu chưa có cái nào)
    pub fn roll(&mut self, dt: f32) -> Option<Kind> {
        if self.event.is_some() {
            return None;
        }
        self.check_t -= dt;
        if self.check_t > 0.0 {
            return None;
        }
        self.check_t = CHECK_EVERY;
        if rnd(0.0, 1.0) < CHANCE {
            Some(match rnd(0.0, 3.0).floor() as u32 {
                0 => Kind::Fire,
                1 => Kind::War,
                _ => Kind::Storm,
            })
        } else {
            None
        }
    }

    pub fn start(&mut self, kind: Kind, spot: Vec3) {
        self.event = Some(Event {
            kind,
            left: kind.duration(),
            spot,
            tick: 0.0,
            dmg_t: 0.0,
            flash: 0.0,
            bombs: Vec::new(),
        });
    }
}
