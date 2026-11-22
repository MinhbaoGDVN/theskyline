use macroquad::prelude::*;

use crate::menu::{Menu, Screen};

/// Thời gian mờ vào / mờ ra (giây)
const FADE_TIME: f32 = 0.5;
/// Thời gian giữ logo khi mở game (giây)
const HOLD_TIME: f32 = 2.0;
const CREDIT: &str = "Made by The Skyline Team - Power by Rust";

/// Lớp đen mờ dần kèm logo game và dòng credit.
/// Mở game: mờ vào 0.25s, giữ 1.5s, mờ ra 0.25s rồi vào menu.
/// Chuyển menu <-> game: mờ đen 0.25s, đổi màn hình, mờ lại 0.25s.
pub struct Fade {
    alpha: f32,
    pending: Option<Screen>,
    splash: Option<f32>,
}

impl Fade {
    pub fn new() -> Self {
        Self { alpha: 0.0, pending: None, splash: Some(0.0) }
    }

    /// Yêu cầu chuyển sang màn hình khác
    pub fn request(&mut self, s: Screen) {
        if self.pending.is_none() {
            self.pending = Some(s);
        }
    }

    pub fn update(&mut self, dt: f32, menu: &mut Menu) {
        if let Some(t) = self.splash {
            let t = t + dt;
            let total = 2.0 * FADE_TIME + HOLD_TIME;
            if t >= total {
                self.splash = None;
                self.alpha = 0.0;
            } else {
                self.splash = Some(t);
                self.alpha = if t < FADE_TIME {
                    t / FADE_TIME
                } else if t < FADE_TIME + HOLD_TIME {
                    1.0
                } else {
                    1.0 - (t - FADE_TIME - HOLD_TIME) / FADE_TIME
                }
                .clamp(0.0, 1.0);
            }
            return;
        }
        if self.pending.is_some() {
            self.alpha = (self.alpha + dt / FADE_TIME).min(1.0);
            if self.alpha >= 1.0 {
                if let Some(s) = self.pending.take() {
                    menu.screen = s;
                }
            }
        } else {
            self.alpha = (self.alpha - dt / FADE_TIME).max(0.0);
        }
    }

    /// Vẽ lớp đen, logo (icon.png) đập nhẹ ở giữa, và dòng credit bên dưới
    pub fn draw(&self, tex: &Texture2D, font: Option<&Font>) {
        if self.alpha <= 0.0 {
            return;
        }
        let w = screen_width();
        let h = screen_height();
        draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, self.alpha));

        let pulse = 1.0 + 0.04 * (get_time() as f32 * 4.0).sin();
        let dw = w * 0.18 * pulse;
        let dh = dw * tex.height() / tex.width();
        let cy = h * 0.46;
        draw_texture_ex(
            tex,
            (w - dw) / 2.0,
            cy - dh / 2.0,
            Color::new(1.0, 1.0, 1.0, self.alpha),
            DrawTextureParams {
                dest_size: Some(vec2(dw, dh)),
                ..Default::default()
            },
        );

        let size: u16 = 22;
        let d = measure_text(CREDIT, font, size, 1.0);
        draw_text_ex(
            CREDIT,
            (w - d.width) / 2.0,
            cy + dh / 2.0 + 44.0,
            TextParams {
                font,
                font_size: size,
                color: Color::new(0.85, 0.87, 0.9, self.alpha),
                ..Default::default()
            },
        );
    }
}
