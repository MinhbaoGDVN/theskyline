mod camera;
mod city;
mod combat;
#[path = ".config"]
mod config;
mod discord;
mod game;
mod hud;
mod map;
mod icon;
mod input;
mod menu;
mod minimap;
mod models;
mod obj;
mod render;
mod road;
mod house;
mod shadow;
mod traffic;
mod vehicle;
mod zones;

use macroquad::audio::{load_sound_from_bytes, play_sound, play_sound_once, stop_sound, PlaySoundParams};
use macroquad::prelude::*;

use game::Game;
use menu::{Menu, MenuResult, Screen};
use models::Models;

/// Buoc logic co dinh: 60 lan moi giay, khong phu thuoc FPS
const STEP: f32 = 1.0 / 60.0;
/// Gioi han thoi gian mot frame, tranh chay qua nhieu buoc khi may qua cham
const MAX_FRAME: f32 = 0.25;

fn window_conf() -> Conf {
    Conf {
        window_title: "The Skyline".to_owned(),
        window_width: 1280,
        window_height: 720,
        icon: icon::window_icon(),
        ..Default::default()
    }
}

fn load_font() -> Option<Font> {
    load_ttf_font_from_bytes(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/ui.ttf"))).ok()
}

#[macroquad::main(window_conf)]
async fn main() {
    macroquad::rand::srand(get_time() as u64);

    let font = load_font();
    if font.is_none() {
        eprintln!("Khong nap duoc assets/fonts/ui.ttf, dung font mac dinh");
    }
    let menu_font = load_font();

    let honk = load_sound_from_bytes(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sounds/honk.ogg")))
        .await
        .expect("honk.ogg");
    let ambient = load_sound_from_bytes(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sounds/ambient.ogg")))
        .await
        .expect("ambient.ogg");
    let mut ambient_on = false;

    let mut game = Game::new(Models::load(), font);
    let mut rpc = discord::Rpc::start();
    let mut menu = Menu::new();
    let mut acc = 0.0f32;
    loop {
        match menu.screen {
            Screen::Playing => {
                if is_key_pressed(KeyCode::H) {
                    play_sound_once(&honk);
                }
                let want = game.active_is_plane();
                if want != ambient_on {
                    if want {
                        play_sound(&ambient, PlaySoundParams { looped: true, volume: 0.5 });
                    } else {
                        stop_sound(&ambient);
                    }
                    ambient_on = want;
                }

                acc += get_frame_time().min(MAX_FRAME);
                game.begin_frame();
                while acc >= STEP {
                    game.update(STEP);
                    acc -= STEP;
                }
                if is_key_pressed(KeyCode::Escape) {
                    menu.screen = Screen::Menu;
                    acc = 0.0;
                }
                game.draw();
                let (details, state) = game.status();
                rpc.tick(details, state);
            }
            _ => {
                if ambient_on {
                    stop_sound(&ambient);
                    ambient_on = false;
                }
                acc = 0.0;
                match menu.frame(menu_font.as_ref()) {
                    MenuResult::Play => menu.screen = Screen::Playing,
                    MenuResult::Quit => break,
                    MenuResult::None => {}
                }
                rpc.tick("Đang ở menu".to_string(), "The Skyline".to_string());
            }
        }
        next_frame().await;
    }
}