use std::sync::atomic::{AtomicU32, Ordering};

use macroquad::prelude::*;

use crate::input;
use crate::weather;

/// Man hinh dang hien thi
pub enum Screen {
    Menu,
    Settings,
    Credits,
    Playing,
}

/// Ket qua mot khung hinh cua menu
pub enum MenuResult {
    None,
    Play,
    Quit,
}

// Am luong (0.0 - 1.0), luu duoi dang bit cua f32
static MASTER_VOL: AtomicU32 = AtomicU32::new(0x3F33_3333); // 0.7
static MUSIC_VOL: AtomicU32 = AtomicU32::new(0x3F00_0000); // 0.5
static SFX_VOL: AtomicU32 = AtomicU32::new(0x3F4C_CCCD); // 0.8

fn load_vol(a: &AtomicU32) -> f32 {
    f32::from_bits(a.load(Ordering::Relaxed))
}

fn store_vol(a: &AtomicU32, v: f32) {
    a.store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

pub fn master_volume() -> f32 {
    load_vol(&MASTER_VOL)
}

pub fn music_volume() -> f32 {
    load_vol(&MUSIC_VOL)
}

pub fn sfx_volume() -> f32 {
    load_vol(&SFX_VOL)
}

pub fn set_master_volume(v: f32) {
    store_vol(&MASTER_VOL, v);
}

pub fn set_music_volume(v: f32) {
    store_vol(&MUSIC_VOL, v);
}

pub fn set_sfx_volume(v: f32) {
    store_vol(&SFX_VOL, v);
}

pub struct Menu {
    pub screen: Screen,
    sel: usize,
    bg: Texture2D,
}

const MAIN_ITEMS: [&str; 4] = ["Chơi", "Cài đặt", "Credit", "Thoát"];
const BTN_W: f32 = 420.0;
const BTN_H: f32 = 76.0;
const BTN_GAP: f32 = 104.0;
/// Vi tri cum nut, theo ti le man hinh
const COL_X: f32 = 0.07;
const COL_Y: f32 = 0.42;

// Man hinh Cai dat: toa do theo man hinh 720 px (se nhan voi ti le thuc te)
const SET_ROWS: usize = 7;
const SET_W: f32 = 620.0;
const SET_H: f32 = 52.0;
/// Vi tri y cua tung hang: 0-2 am thanh, 3 dieu khien, 4 hien thi, 5 quay lai
const ROW_Y: [f32; SET_ROWS] = [140.0, 210.0, 280.0, 392.0, 504.0, 574.0, 644.0];
/// Tieu de tung nhom
const HEAD_Y: [(f32, &str); 3] = [(122.0, "ÂM THANH"), (374.0, "ĐIỀU KHIỂN"), (486.0, "HIỂN THỊ")];

fn gold() -> Color {
    Color::new(0.97, 0.72, 0.16, 1.0)
}

fn pct(v: f32) -> i32 {
    (v * 100.0).round() as i32
}

fn onoff(b: bool) -> &'static str {
    if b {
        "BẬT"
    } else {
        "TẮT"
    }
}

impl Menu {
    pub fn new() -> Self {
        Self {
            screen: Screen::Menu,
            sel: 0,
            bg: Texture2D::from_file_with_format(include_bytes!("../assets/images/menu-bg.png"), None),
        }
    }

    pub fn frame(&mut self, font: Option<&Font>) -> MenuResult {
        clear_background(BLACK);
        draw_texture_ex(
            &self.bg,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        match self.screen {
            Screen::Settings => self.settings_frame(font),
            Screen::Credits => self.credits_frame(font),
            _ => self.main_frame(font),
        }
    }

    fn main_frame(&mut self, font: Option<&Font>) -> MenuResult {
        let w = screen_width();
        let h = screen_height();
        let x = w * COL_X;
        let y0 = h * COL_Y;
        draw_panel(x - 28.0, y0 - 28.0, BTN_W + 56.0, BTN_GAP * (MAIN_ITEMS.len() as f32 - 1.0) + BTN_H + 56.0);

        let (mx, my) = mouse_position();
        let mut result = MenuResult::None;
        for (i, label) in MAIN_ITEMS.iter().enumerate() {
            let by = y0 + i as f32 * BTN_GAP;
            let hover = inside(mx, my, x, by, BTN_W, BTN_H);
            if hover {
                self.sel = i;
            }
            let selected = i == self.sel;
            let clicked = (hover && is_mouse_button_pressed(MouseButton::Left))
                || (selected && is_key_pressed(KeyCode::Enter));
            draw_button(font, label, x, by, BTN_W, BTN_H, 30, selected);
            if clicked {
                match i {
                    0 => result = MenuResult::Play,
                    1 => {
                        self.screen = Screen::Settings;
                        self.sel = 0;
                    }
                    2 => {
                        self.screen = Screen::Credits;
                        self.sel = 0;
                    }
                    _ => result = MenuResult::Quit,
                }
            }
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.sel = (self.sel + MAIN_ITEMS.len() - 1) % MAIN_ITEMS.len();
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.sel = (self.sel + 1) % MAIN_ITEMS.len();
        }
        result
    }

    fn credits_frame(&mut self, font: Option<&Font>) -> MenuResult {
        let w = screen_width();
        let h = screen_height();
        let s = h / 720.0;
        let x0 = (w - SET_W) / 2.0;
        draw_panel(x0 - 28.0, 90.0 * s, SET_W + 56.0, 520.0 * s);
        draw_centered(font, "CREDIT", w / 2.0, 140.0 * s, 40, gold());

        let people = [
            ("EzimerMC", "Music, Idea/Designer", "https://www.youtube.com/@EzimerMC"),
            ("k9os", "Tester", "https://www.youtube.com/@k9os.2"),
            ("MinhbaoGDVN", "Coder, Model", "https://www.youtube.com/@MinhbaoGDVN_real"),
        ];
        for (i, (name, role, link)) in people.iter().enumerate() {
            let y = (190.0 + i as f32 * 96.0) * s;
            draw_centered(font, name, w / 2.0, y, 30, WHITE);
            draw_centered(font, role, w / 2.0, y + 30.0 * s, 22, Color::new(0.75, 0.8, 0.9, 1.0));
            draw_centered(font, link, w / 2.0, y + 54.0 * s, 18, Color::new(0.45, 0.75, 1.0, 1.0));
        }

        let bw = BTN_W;
        let bh = SET_H * s;
        let bx = (w - bw) / 2.0;
        let by = 540.0 * s;
        let (mx, my) = mouse_position();
        let hover = inside(mx, my, bx, by, bw, bh);
        let clicked = (hover && is_mouse_button_pressed(MouseButton::Left)) || is_key_pressed(KeyCode::Enter);
        draw_button(font, "Quay lại", bx, by, bw, bh, 26, true);
        if clicked || is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Menu;
            self.sel = 2;
        }
        MenuResult::None
    }

    fn settings_frame(&mut self, font: Option<&Font>) -> MenuResult {
        let w = screen_width();
        let h = screen_height();
        let s = h / 720.0;
        let x0 = (w - SET_W) / 2.0;
        draw_panel(x0 - 28.0, 90.0 * s, SET_W + 56.0, 610.0 * s);
        for (hy, title) in HEAD_Y.iter() {
            draw_centered(font, title, w / 2.0, hy * s, 22, gold());
        }

        let master = master_volume();
        let music = music_volume();
        let sfx = sfx_volume();
        let invert = input::invert_plane();
        let notice = weather::show_notice();
        let help = crate::hud::show_help();
        let labels: [String; SET_ROWS] = [
            format!("Âm lượng tổng: {}%", pct(master)),
            format!("Nhạc nền: {}%", pct(music)),
            format!("Hiệu ứng: {}%", pct(sfx)),
            format!("Đảo ngược điều khiển máy bay: {}", onoff(invert)),
            format!("Thông báo thời tiết: {}", onoff(notice)),
            format!("Hướng dẫn điều khiển: {}", onoff(help)),
            "Quay lại".to_string(),
        ];

        let (mx, my) = mouse_position();
        let left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
        let right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);
        let mut toggle_invert = false;
        let mut toggle_notice = false;
        let mut toggle_help = false;
        let mut back = false;
        for i in 0..SET_ROWS {
            let bw = if i == SET_ROWS - 1 { BTN_W } else { SET_W };
            let bh = SET_H * s;
            let bx = (w - bw) / 2.0;
            let by = ROW_Y[i] * s;
            let hover = inside(mx, my, bx, by, bw, bh);
            if hover {
                self.sel = i;
            }
            let selected = i == self.sel;
            let clicked = (hover && is_mouse_button_pressed(MouseButton::Left))
                || (selected && is_key_pressed(KeyCode::Enter));
            draw_button(font, &labels[i], bx, by, bw, bh, 26, selected);

            if i < 3 {
                // Thanh truot nam ngay trong nut
                let cur = match i {
                    0 => master,
                    1 => music,
                    _ => sfx,
                };
                let px = bx + 16.0;
                let pw = bw - 32.0;
                let py = by + bh - 12.0;
                draw_rectangle(px, py, pw, 6.0, Color::new(0.0, 0.0, 0.0, 0.55));
                draw_rectangle(px, py, pw * cur, 6.0, gold());
                if hover && is_mouse_button_down(MouseButton::Left) {
                    set_slider(i, (mx - px) / pw);
                }
                if selected {
                    if left {
                        set_slider(i, cur - 0.05);
                    }
                    if right {
                        set_slider(i, cur + 0.05);
                    }
                }
            } else if clicked {
                match i {
                    3 => toggle_invert = true,
                    4 => toggle_notice = true,
                    5 => toggle_help = true,
                    _ => back = true,
                }
            }
        }

        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.sel = (self.sel + SET_ROWS - 1) % SET_ROWS;
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.sel = (self.sel + 1) % SET_ROWS;
        }
        if toggle_invert {
            input::set_invert_plane(!invert);
        }
        if toggle_notice {
            weather::set_show_notice(!notice);
        }
        if toggle_help {
            crate::hud::set_show_help(!help);
        }
        if back || is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Menu;
            self.sel = 1;
        }
        MenuResult::None
    }
}

fn set_slider(i: usize, v: f32) {
    match i {
        0 => set_master_volume(v),
        1 => set_music_volume(v),
        _ => set_sfx_volume(v),
    }
}

fn inside(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

fn draw_centered(font: Option<&Font>, text: &str, cx: f32, cy: f32, size: u16, color: Color) {
    let d = measure_text(text, font, size, 1.0);
    draw_text_ex(
        text,
        cx - d.width / 2.0,
        cy,
        TextParams {
            font,
            font_size: size,
            color,
            ..Default::default()
        },
    );
}

/// Khung nen mo phia sau cum nut, co vach vang o tren
fn draw_panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.40));
    draw_rectangle(x, y, w, 4.0, gold());
}

/// Nut lon: bong do, nen gradient gia, vien vang khi duoc chon, mui ten hai ben
fn draw_button(font: Option<&Font>, label: &str, x: f32, y: f32, w: f32, h: f32, size: u16, selected: bool) {
    draw_rectangle(x + 6.0, y + 8.0, w, h, Color::new(0.0, 0.0, 0.0, 0.45));
    let fill = if selected {
        Color::new(0.20, 0.45, 0.95, 1.0)
    } else {
        Color::new(0.11, 0.14, 0.20, 0.94)
    };
    draw_rectangle(x, y, w, h, fill);
    let shine = if selected { 0.16 } else { 0.05 };
    draw_rectangle(x, y, w, h * 0.45, Color::new(1.0, 1.0, 1.0, shine));
    let edge = if selected { gold() } else { Color::new(0.50, 0.55, 0.65, 1.0) };
    draw_rectangle_lines(x, y, w, h, 3.0, edge);
    draw_rectangle_lines(x + 6.0, y + 6.0, w - 12.0, h - 12.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.18));
    if selected {
        let cy = y + h / 2.0;
        draw_triangle(vec2(x + 22.0, cy - 10.0), vec2(x + 22.0, cy + 10.0), vec2(x + 36.0, cy), gold());
        draw_triangle(vec2(x + w - 22.0, cy - 10.0), vec2(x + w - 22.0, cy + 10.0), vec2(x + w - 36.0, cy), gold());
    }
    let d = measure_text(label, font, size, 1.0);
    let tx = x + (w - d.width) / 2.0;
    let ty = y + h / 2.0 + size as f32 * 0.35 - 4.0;
    draw_text_ex(
        label,
        tx + 2.0,
        ty + 2.0,
        TextParams {
            font,
            font_size: size,
            color: Color::new(0.0, 0.0, 0.0, 0.5),
            ..Default::default()
        },
    );
    draw_text_ex(
        label,
        tx,
        ty,
        TextParams {
            font,
            font_size: size,
            color: WHITE,
            ..Default::default()
        },
    );
}
