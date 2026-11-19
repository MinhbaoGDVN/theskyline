use macroquad::prelude::*;

use crate::city::City;
use crate::map;
use crate::render::Batch;

/// Máu rất lớn: các công trình khu này không bị phá hủy
const HP: f32 = 1.0e6;
const ROAD_COLOR: [f32; 3] = [0.2, 0.2, 0.22];
const LINE_COLOR: [f32; 3] = [1.0, 0.82, 0.25];

/// Đường nối: mặt đường rộng 8 và vạch kẻ giữa, cùng kiểu với đường trong thành phố
fn road(flat: &mut Batch, center: Vec3, size: Vec3) {
    flat.push_box(vec3(center.x, 0.03, center.z), vec3(size.x, 0.06, size.z), ROAD_COLOR);
    if size.x > size.z {
        flat.push_box(vec3(center.x, 0.1, center.z), vec3(size.x, 0.02, 0.6), LINE_COLOR);
    } else {
        flat.push_box(vec3(center.x, 0.1, center.z), vec3(0.6, 0.02, size.z), LINE_COLOR);
    }
}

pub fn add_zones(city: &mut City) {
    let mut flat = Batch::new();

    // ===== Sân bay phía tây (máy bay xuất phát trên đường băng) =====
    flat.push_box(vec3(-700.0, 0.04, 0.0), vec3(400.0, 0.08, 30.0), ROAD_COLOR);
    for i in 0..10 {
        let x = -890.0 + i as f32 * 40.0;
        flat.push_box(vec3(x, 0.09, 0.0), vec3(18.0, 0.02, 1.0), [1.0, 1.0, 1.0]);
    }
    road(&mut flat, vec3(-545.0, 0.0, -70.0), vec3(130.0, 0.0, 8.0));
    city.add_custom(vec2(-640.0, -70.0), 30.0, 12.0, 14.0, [0.85, 0.85, 0.88], HP); // nhà ga
    city.add_custom(vec2(-740.0, 60.0), 20.0, 15.0, 14.0, [0.45, 0.5, 0.4], HP); // nhà chứa máy bay
    map::tower(city, vec2(-600.0, -40.0)); // tháp kiểm soát không lưu

    // ===== Căn cứ quân sự phía đông (xe tăng xuất phát trong căn cứ) =====
    flat.push_box(vec3(650.0, 0.04, 0.0), vec3(200.0, 0.08, 200.0), [0.38, 0.42, 0.25]);
    road(&mut flat, vec3(516.0, 0.0, 0.0), vec3(72.0, 0.0, 8.0));
    let wall = [0.55, 0.53, 0.45];
    map::wall(city, 550.0, 96.5, 750.0, 99.5, 6.0, wall); // tường bắc
    map::wall(city, 550.0, -99.5, 750.0, -96.5, 6.0, wall); // tường nam
    map::wall(city, 746.5, -100.0, 749.5, 100.0, 6.0, wall); // tường đông
    map::wall(city, 550.0, -100.0, 553.0, -12.0, 6.0, wall); // tường tây phía nam
    map::wall(city, 550.0, 12.0, 553.0, 100.0, 6.0, wall); // tường tây phía bắc (cổng z -12..12)
    city.add_custom(vec2(700.0, 50.0), 20.0, 15.0, 12.0, [0.4, 0.45, 0.35], HP); // nhà kho
    city.add_custom(vec2(580.0, 80.0), 4.0, 4.0, 14.0, [0.5, 0.48, 0.42], HP); // tháp canh

    // ===== Trạm xe đạp công cộng phía nam (xe máy xuất phát giữa quảng trường) =====
    road(&mut flat, vec3(0.0, 0.0, -550.0), vec3(8.0, 0.0, 140.0));
    flat.push_box(vec3(0.0, 0.04, -650.0), vec3(80.0, 0.08, 60.0), [0.55, 0.55, 0.57]);
    flat.push_box(vec3(0.0, 4.2, -650.0), vec3(60.0, 0.4, 14.0), [0.35, 0.37, 0.4]);
    for x in [-29.0_f32, 29.0] {
        for z in [-657.0_f32, -643.0] {
            flat.push_box(vec3(x, 2.0, z), vec3(0.5, 4.0, 0.5), [0.3, 0.32, 0.35]);
        }
    }
    let rack = [0.25, 0.27, 0.3];
    for i in -7..=7 {
        let x = i as f32 * 4.0;
        for z in [-661.0_f32, -639.0] {
            flat.push_box(vec3(x, 0.5, z), vec3(0.12, 1.0, 0.12), rack);
            flat.push_box(vec3(x, 0.95, z), vec3(0.12, 0.12, 2.0), rack);
        }
    }
    flat.push_box(vec3(-38.0, 2.2, -650.0), vec3(0.3, 4.4, 0.3), [0.3, 0.3, 0.32]);
    flat.push_box(vec3(-38.0, 4.2, -650.0), vec3(3.0, 1.4, 0.2), [0.2, 0.42, 0.45]);

    city.roads.extend(flat.finish());
}