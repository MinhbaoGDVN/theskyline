use macroquad::prelude::*;

const HELP: &str = "W/S: ga/phanh (may bay: ga) | A/D: lai | Up/Down: mui may bay | E: len/xuong xe | Ctrl: chay bo | Space: phanh tay | X/Z: so | T: so tu dong | F: ban | R: hoi phuc | H: bip bip";

/// Bảng thông tin ở góc trên trái, mỗi dòng một đoạn
pub fn draw_panel(font: Option<&Font>, x: f32, y: f32, lines: &[String], size: u16) {
    let lh = size as f32 * 1.4;
    let w = lines
        .iter()
        .map(|l| measure_text(l, font, size, 1.0).width)
        .fold(0.0, f32::max)
        + 20.0;
    let h = lines.len() as f32 * lh + 12.0;
    draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.45));
    for (i, line) in lines.iter().enumerate() {
        draw_text_ex(
            line,
            x + 10.0,
            y + 8.0 + (i as f32 + 1.0) * lh - 4.0,
            TextParams {
                font,
                font_size: size,
                color: WHITE,
                ..Default::default()
            },
        );
    }
}

/// Cài đặt: có hiện hướng dẫn điều khiển ở dưới cùng không
pub static SHOW_HELP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub fn show_help() -> bool {
    SHOW_HELP.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn set_show_help(v: bool) {
    SHOW_HELP.store(v, std::sync::atomic::Ordering::Relaxed);
}

pub fn draw_help(font: Option<&Font>) {
    if !show_help() {
        return;
    }
    let h = 26.0;
    let y = screen_height() - h;
    draw_rectangle(0.0, y, screen_width(), h, Color::new(0.0, 0.0, 0.0, 0.45));
    draw_text_ex(
        HELP,
        10.0,
        y + 18.0,
        TextParams {
            font,
            font_size: 15,
            color: WHITE,
            ..Default::default()
        },
    );
}

pub fn draw_notice(font: Option<&Font>, text: &str) {
    let size = 30;
    let w = measure_text(text, font, size, 1.0).width;
    draw_text_ex(
        text,
        (screen_width() - w) / 2.0,
        screen_height() * 0.35,
        TextParams {
            font,
            font_size: size,
            color: WHITE,
            ..Default::default()
        },
    );
}