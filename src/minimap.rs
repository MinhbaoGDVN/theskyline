use macroquad::prelude::*;

use crate::city::City;
use crate::config::{BLOCK, EXT, GRID_N, ROAD};
use crate::traffic::NpcCar;
use crate::vehicle::Vehicle;

/// Kích thước minimap (px) và khoảng cách tới mép màn hình
pub const SIZE: f32 = 320.0;
pub const MARGIN: f32 = 14.0;
/// Minimap phủ toàn bản đồ: từ -HALF đến +HALF (m), đủ cho sân bay, nhà tù, cửa hàng
pub const HALF: f32 = 800.0;

/// Minimap góc trên bên phải: nhà, đường, xe NPC (xám), xe khác (cam), xe đang lái (đỏ, theo hướng)
pub fn draw(city: &City, vehicles: &[Vehicle], active: usize, traffic: &[NpcCar]) {
    let x0 = screen_width() - SIZE - MARGIN;
    let y0 = MARGIN;
    let s = SIZE / (2.0 * HALF);
    let to_map = |x: f32, z: f32| {
        vec2(
            x0 + (x.clamp(-HALF, HALF) + HALF) * s,
            y0 + (z.clamp(-HALF, HALF) + HALF) * s,
        )
    };

    draw_rectangle(x0 - 4.0, y0 - 4.0, SIZE + 8.0, SIZE + 8.0, Color::new(0.0, 0.0, 0.0, 0.55));
    draw_rectangle(x0, y0, SIZE, SIZE, Color::new(0.42, 0.61, 0.29, 1.0));

    let road_w = (ROAD * s).max(1.5);
    let road = Color::new(0.25, 0.25, 0.28, 1.0);
    // Đường lưới chỉ vẽ trong phạm vi thành phố (±EXT), không kéo dài ra ngoài
    for k in -GRID_N..=GRID_N {
        let p = k as f32 * BLOCK;
        let a = to_map(p, 0.0);
        let top = to_map(p, -EXT).y;
        let bot = to_map(p, EXT).y;
        draw_rectangle(a.x - road_w * 0.5, top, road_w, bot - top, road);
        let b = to_map(0.0, p);
        let left = to_map(-EXT, p).x;
        let right = to_map(EXT, p).x;
        draw_rectangle(left, b.y - road_w * 0.5, right - left, road_w, road);
    }

    for b in &city.buildings {
        let a = to_map(b.min.x, b.min.y);
        let sz = (b.max - b.min) * s;
        let c = b.rgb;
        draw_rectangle(a.x, a.y, sz.x.max(1.0), sz.y.max(1.0), Color::new(c[0], c[1], c[2], 1.0));
    }

    for n in traffic {
        let p = n.position();
        let q = to_map(p.x, p.z);
        draw_circle(q.x, q.y, 1.8, Color::new(0.85, 0.85, 0.85, 1.0));
    }

    for (i, v) in vehicles.iter().enumerate() {
        if i == active {
            continue;
        }
        let q = to_map(v.pos.x, v.pos.z);
        draw_circle(q.x, q.y, 3.0, Color::new(1.0, 0.6, 0.1, 1.0));
    }

    let v = &vehicles[active];
    let p = to_map(v.pos.x, v.pos.z);
    let f = vec2(v.heading.sin(), v.heading.cos());
    let side = vec2(-f.y, f.x);
    draw_triangle(p + f * 9.0, p - f * 5.0 + side * 5.0, p - f * 5.0 - side * 5.0, RED);

    draw_rectangle_lines(x0, y0, SIZE, SIZE, 2.0, WHITE);
}