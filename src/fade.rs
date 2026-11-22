use macroquad::prelude::*;

use crate::menu::{Menu, Screen};

/// Tốc độ mờ dần (độ mờ mỗi giây)
const FADE_SPEED: f32 = 2.5;

/// Lớp đen mờ dần kèm logo nhỏ đập nhẹ: mở đầu game, và khi chuyển menu <-> game
pub struct Fade {
    alpha: f32,
    pending: Option<Screen>,
}

impl Fade {
    /// Bắt đầu từ màn đen rồi mờ dần lên menu
    pub fn new() -> Self {
        Self { alpha: 1.0, pending: None }
    }

    /// Yêu cầu chuyển sang màn hình khác: mờ đen, đổi màn hình, rồi mờ dần lên
    pub fn request(&mut self, s: Screen) {
        if self.pending.is_none() {
            self.pending = Some(s);
        }
    }

    pub fn update(&mut self, dt: f32, menu: &mut Menu) {
        if self.pending.is_some() {
            self.alpha = (self.alpha + FADE_SPEED * dt).min(1.0);
            if self.alpha >= 1.0 {
                if let Some(s) = self.pending.take() {
                    menu.screen = s;
                }
            }
        } else {
            self.alpha = (self.alpha - FADE_SPEED * dt).max(0.0);
        }
    }

    /// Vẽ lớp đen và logo nhỏ (cắt từ ảnh menu) ở giữa màn hình, đập nhẹ theo thời gian
    pub fn draw(&self, tex: &Texture2D) {
        if self.alpha <= 0.0 {
            return;
        }
        let w = screen_width();
        let h = screen_height();
        draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, self.alpha));

        let tw = tex.width();
        let th = tex.height();
        let src = Rect::new(tw * 0.45, th * 0.12, tw * 0.53, th * 0.6);
        let pulse = 1.0 + 0.04 * (get_time() as f32 * 4.0).sin();
        let dw = w * 0.32 * pulse;
        let dh = dw * src.h / src.w;
        draw_texture_ex(
            tex,
            (w - dw) / 2.0,
            (h - dh) / 2.0,
            Color::new(1.0, 1.0, 1.0, self.alpha),
            DrawTextureParams {
                dest_size: Some(vec2(dw, dh)),
                source: Some(src),
                ..Default::default()
            },
        );
    }
}
