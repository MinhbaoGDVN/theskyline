mod camera;
mod city;
mod combat;
#[path = ".config"]
mod config;
mod game;
mod hud;
mod input;
mod minimap;
mod models;
mod obj;
mod render;
mod shadow;
mod traffic;
mod vehicle;

use macroquad::prelude::*;

use game::Game;
use models::Models;

const FONT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/ui.ttf");
/// Bước logic cố định: 60 lần mỗi giây, không phụ thuộc FPS
const STEP: f32 = 1.0 / 60.0;
/// Giới hạn thời gian một frame, tránh chạy quá nhiều bước khi máy quá chậm
const MAX_FRAME: f32 = 0.25;

fn window_conf() -> Conf {
    Conf {
        window_title: "The Skyline".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    macroquad::rand::srand(get_time() as u64);

    let font = load_ttf_font(FONT).await.ok();
    if font.is_none() {
        eprintln!("Không nạp được assets/fonts/ui.ttf, dùng font mặc định (không có dấu tiếng Việt)");
    }

    let mut game = Game::new(Models::load(), font);
    let mut acc = 0.0f32;
    loop {
        acc += get_frame_time().min(MAX_FRAME);
        game.begin_frame();
        while acc >= STEP {
            game.update(STEP);
            acc -= STEP;
        }
        game.draw();
        next_frame().await;
    }
}