use macroquad::prelude::*;

use crate::city::City;
use crate::obj::parse_obj;
use crate::render::{push_model, Batch};

/// Máu rất lớn: công trình đặc biệt không bị phá hủy
const HP: f32 = 1.0e6;
const WHITE: [f32; 3] = [0.92, 0.92, 0.92];
const RED: [f32; 3] = [0.85, 0.12, 0.12];
const BLUE: [f32; 3] = [0.15, 0.3, 0.75];
const GREEN: [f32; 3] = [0.2, 0.6, 0.3];
const STONE: [f32; 3] = [0.82, 0.78, 0.65];
const DARK: [f32; 3] = [0.15, 0.15, 0.17];

/// Tâm ô (i, j) trong lưới thành phố
pub fn block(i: i32, j: i32) -> Vec2 {
    vec2((i as f32 + 0.5) * 80.0, (j as f32 + 0.5) * 80.0)
}

/// Tháp kiểm soát không lưu: thân trơn, chỉ buồng lái phía trên có cửa sổ
pub fn tower(city: &mut City, c: Vec2) {
    city.add_band(c, 2.5, 2.5, 0.0, 30.0, WHITE, HP, f32::MAX);
    city.add_band(c, 4.5, 4.5, 30.0, 40.0, [0.7, 0.78, 0.85], HP, 30.0);
}

/// Tường bao vẽ bằng OBJ: một hình hộp (x0,z0)-(x1,z1), cao `height`
pub fn wall(city: &mut City, x0: f32, z0: f32, x1: f32, z1: f32, height: f32, rgb: [f32; 3]) {
    let obj = box_obj(x0, 0.0, z0, x1, height, z1);
    let mtl = format!("newmtl wall\nKd {} {} {}\n", rgb[0], rgb[1], rgb[2]);
    let parts = parse_obj(&obj, &mtl);
    let mut verts = Vec::new();
    push_model(&mut verts, &parts, Mat4::IDENTITY);
    city.add_verts(vec2(x0, z0), vec2(x1, z1), height, rgb, HP, verts);
}

/// Chuỗi OBJ cho một hình hộp
fn box_obj(x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32) -> String {
    let v = [
        (x0, y0, z0),
        (x1, y0, z0),
        (x1, y1, z0),
        (x0, y1, z0),
        (x0, y0, z1),
        (x1, y0, z1),
        (x1, y1, z1),
        (x0, y1, z1),
    ];
    let mut s = String::from("usemtl wall\n");
    for (x, y, z) in v {
        s.push_str(&format!("v {x} {y} {z}\n"));
    }
    for f in ["1 4 3 2", "5 6 7 8", "1 2 6 5", "4 8 7 3", "1 5 8 4", "2 3 7 6"] {
        s.push_str(&format!("f {f}\n"));
    }
    s
}

/// Năm công trình đặc biệt, thay nhà thường trong các khối đã chọn
pub fn specials(city: &mut City) {
    let mut flat = Batch::new();

    // Trạm xá: thân trắng, dấu thập đỏ trên mái
    let c = block(-3, -2);
    city.clear_block(c, 40.0);
    city.add_custom(c, 14.0, 12.0, 16.0, WHITE, HP);
    flat.push_box(vec3(c.x, 16.2, c.y), vec3(6.0, 0.4, 1.8), RED);
    flat.push_box(vec3(c.x, 16.2, c.y), vec3(1.8, 0.4, 6.0), RED);

    // Đồn công an: thân xanh xám, biển xanh và cờ
    let c = block(2, -3);
    city.clear_block(c, 40.0);
    city.add_custom(c, 16.0, 12.0, 18.0, [0.35, 0.4, 0.5], HP);
    flat.push_box(vec3(c.x, 15.0, c.y + 12.2), vec3(10.0, 1.2, 0.2), BLUE);
    flat.push_box(vec3(c.x + 10.0, 21.0, c.y), vec3(0.2, 6.0, 0.2), [0.7, 0.7, 0.72]);
    flat.push_box(vec3(c.x + 10.4, 23.5, c.y), vec3(0.1, 1.6, 2.4), BLUE);

    // Ngân hàng: đá, có cột và mái bằng phẳng phía trước
    let c = block(-1, 2);
    city.clear_block(c, 40.0);
    city.add_custom(c, 20.0, 14.0, 14.0, STONE, HP);
    for x in [-12.0_f32, -4.0, 4.0, 12.0] {
        flat.push_box(vec3(c.x + x, 7.0, c.y + 14.6), vec3(1.2, 14.0, 1.2), WHITE);
    }
    flat.push_box(vec3(c.x, 14.6, c.y), vec3(42.0, 1.0, 30.0), STONE);

    // Trạm cứu hỏa: thân đỏ, tháp luyện tập và cửa garage
    let c = block(3, 1);
    city.clear_block(c, 40.0);
    city.add_custom(c, 20.0, 14.0, 12.0, [0.7, 0.15, 0.12], HP);
    city.add_custom(c + vec2(-14.0, -9.0), 4.0, 4.0, 24.0, [0.65, 0.12, 0.1], HP);
    for x in [-12.0_f32, 0.0, 12.0] {
        flat.push_box(vec3(c.x + x, 4.0, c.y + 14.2), vec3(6.0, 8.0, 0.3), DARK);
    }

    // Tạp hóa: nhà nhỏ vàng, mái che xanh lá
    let c = block(-4, 4);
    city.clear_block(c, 40.0);
    city.add_custom(c, 8.0, 6.0, 7.0, [0.95, 0.85, 0.35], HP);
    flat.push_box(vec3(c.x, 6.4, c.y + 6.8), vec3(14.0, 0.4, 2.0), GREEN);

    city.roads.extend(flat.finish());
}