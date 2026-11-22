use std::sync::atomic::{AtomicU32, Ordering};

use macroquad::prelude::*;

use crate::input;

/// Man hinh dang hien thi
pub enum Screen {
    Menu,
    Settings,
    Playing,
}

/// Ket qua mot khung hinh cua menu
pub enum MenuResult {
    None,
    Play,
    Quit,
}

/// Am luong tong (0.0 - 1.0), luu duoi dang bit cua f32; mac dinh 0.7
static MASTER_VOL: AtomicU32 = AtomicU32::new(0x3F33_3333);

pub fn master_volume() -> f32 {
    f32::from_bits(MASTER_VOL.load(Ordering::Relaxed))
}

pub fn set_master_volume(v: f32) {
    MASTER_VOL.store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

pub struct Menu {
    pub screen: Screen,
    sel: usize,
    bg: Texture2D,
}

const MAIN_ITEMS: [&str; 3] = ["Chơi", "Cài đặt", "Thoát"];
const BTN_W: f32 = 420.0;
const BTN_H: f32 = 76.0;
const BTN_GAP: f32 = 104.0;
const SET_W: f32 = 620.0;
/// Vi tri cum nut, theo ti le man hinh
const COL_X: f32 = 0.07;
const COL_Y: f32 = 0.42;

fn gold() -> Color {
    Color::new(0.97, 0.72, 0.16, 1.0)
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
            draw_button(font, label, x, by, BTN_W, selected);
            if clicked {
                match i {
                    0 => result = MenuResult::Play,
                    1 => {
                        self.screen = Screen::Settings;
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

    fn settings_frame(&mut self, font: Option<&Font>) -> MenuResult {
        let w = screen_width();
        let h = screen_height();
        let x = w * COL_X;
        let y0 = h * COL_Y;
        draw_panel(x - 28.0, y0 - 28.0, SET_W + 56.0, BTN_GAP * 2.0 + BTN_H + 56.0);

        let invert = input::invert_plane();
        let vol = master_volume();
        let labels = [
            format!("Đảo ngược điều khiển máy bay: {}", if invert { "BẬT" } else { "TẮT" }),
            format!("Âm lượng: {}%", (vol * 100.0).round() as i32),
            "Quay lại".to_string(),
        ];
        let (mx, my) = mouse_position();
        let mut toggle = false;
        let mut back = false;
        for i in 0..3 {
            let bw = if i == 2 { BTN_W } else { SET_W };
            let by = y0 + i as f32 * BTN_GAP;
            let hover = inside(mx, my, x, by, bw, BTN_H);
            if hover {
                self.sel = i;
            }
            let selected = i == self.sel;
            let clicked = (hover && is_mouse_button_pressed(MouseButton::Left))
                || (selected && is_key_pressed(KeyCode::Enter));
            let label: &str = &labels[i];
            draw_button(font, label, x, by, bw, selected);

            if i == 1 {
                // Thanh am luong nam trong nut
                let bx = x + 16.0;
                let bwid = bw - 32.0;
                let byb = by + BTN_H - 16.0;
                draw_rectangle(bx, byb, bwid, 8.0, Color::new(0.0, 0.0, 0.0, 0.55));
                draw_rectangle(bx, byb, bwid * vol, 8.0, gold());
                if hover && is_mouse_button_down(MouseButton::Left) {
                    set_master_volume((mx - bx) / bwid);
                }
                if selected {
                    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                        set_master_volume(vol - 0.05);
                    }
                    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                        set_master_volume(vol + 0.05);
                    }
                }
            } else if clicked {
                if i == 0 {
                    toggle = true;
                } else {
                    back = true;
                }
            }
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.sel = (self.sel + 2) % 3;
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.sel = (self.sel + 1) % 3;
        }
        if toggle {
            input::set_invert_plane(!invert);
        }
        if back || is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Menu;
            self.sel = 1;
        }
        MenuResult::None
    }
}

fn inside(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

/// Khung nen mo phia sau cum nut, co vach vang o tren
fn draw_panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.40));
    draw_rectangle(x, y, w, 4.0, gold());
}

/// Nut lon: bong do, nen gradient gia, vien vang khi duoc chon, mui ten hai ben
fn draw_button(font: Option<&Font>, label: &str, x: f32, y: f32, w: f32, selected: bool) {
    let h = BTN_H;
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
        draw_triangle(vec2(x + 22.0, cy - 12.0), vec2(x + 22.0, cy + 12.0), vec2(x + 40.0, cy), gold());
        draw_triangle(vec2(x + w - 22.0, cy - 12.0), vec2(x + w - 22.0, cy + 12.0), vec2(x + w - 40.0, cy), gold());
    }
    let size: u16 = 34;
    let d = measure_text(label, font, size, 1.0);
    let tx = x + (w - d.width) / 2.0;
    let ty = y + h / 2.0 + size as f32 * 0.35 - 6.0;
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