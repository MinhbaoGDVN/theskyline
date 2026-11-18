use macroquad::prelude::*;

use crate::render::light_dir;

/// Độ cao của mặt bóng, cao hơn vỉa hè và vạch kẻ đường
pub const SHADOW_Y: f32 = 0.3;

/// 8 góc của hình hộp bao quanh
pub fn box_corners(mn: Vec3, mx: Vec3) -> [Vec3; 8] {
    [
        vec3(mn.x, mn.y, mn.z),
        vec3(mx.x, mn.y, mn.z),
        vec3(mx.x, mn.y, mx.z),
        vec3(mn.x, mn.y, mx.z),
        vec3(mn.x, mx.y, mn.z),
        vec3(mx.x, mx.y, mn.z),
        vec3(mx.x, mx.y, mx.z),
        vec3(mn.x, mx.y, mx.z),
    ]
}

/// Chiếu một điểm xuống mặt đất theo hướng ánh sáng
fn project(p: Vec3) -> Vec2 {
    let l = light_dir();
    let t = p.y / l.y;
    let q = p - l * t;
    vec2(q.x, q.z)
}

fn cross(o: Vec2, a: Vec2, b: Vec2) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// Bao lồi (monotone chain)
fn hull(mut pts: Vec<Vec2>) -> Vec<Vec2> {
    pts.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap()
            .then(a.y.partial_cmp(&b.y).unwrap())
    });
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let mut lower: Vec<Vec2> = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper: Vec<Vec2> = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Thêm bóng đổ của một hộp (từ 8 góc) vào danh sách đỉnh, màu đen trong suốt
pub fn push_shadow(out: &mut Vec<Vertex>, corners: [Vec3; 8]) {
    let pts: Vec<Vec2> = corners.iter().map(|c| project(*c)).collect();
    let h = hull(pts);
    if h.len() < 3 {
        return;
    }
    let c = Color::new(0.0, 0.0, 0.0, 0.3);
    for i in 1..h.len() - 1 {
        for p in [h[0], h[i], h[i + 1]] {
            out.push(Vertex::new(p.x, SHADOW_Y, p.y, 0.0, 0.0, c));
        }
    }
}