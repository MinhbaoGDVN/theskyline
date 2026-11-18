use macroquad::prelude::*;

use crate::obj::ObjPart;

/// Số đỉnh tối đa cho một lần vẽ (chia hết cho 3, dưới giới hạn macroquad)
pub const MAX_VERTS: usize = 4998;

pub fn light_dir() -> Vec3 {
    vec3(0.4, 0.8, 0.3).normalize()
}

/// Đổ bóng đơn giản theo hướng pháp tuyến
pub fn shade(rgb: [f32; 3], n: Vec3) -> Color {
    let lit = 0.45 + 0.55 * n.dot(light_dir()).max(0.0);
    Color::new(
        (rgb[0] * lit).min(1.0),
        (rgb[1] * lit).min(1.0),
        (rgb[2] * lit).min(1.0),
        1.0,
    )
}

fn vtx(p: Vec3, c: Color) -> Vertex {
    Vertex::new(p.x, p.y, p.z, 0.0, 0.0, c)
}

/// Thêm một tứ giác (2 tam giác) với màu đã đổ bóng
pub fn quad(out: &mut Vec<Vertex>, q: [Vec3; 4], n: Vec3, rgb: [f32; 3]) {
    let c = shade(rgb, n);
    for &i in &[0usize, 1, 2, 0, 2, 3] {
        out.push(vtx(q[i], c));
    }
}

/// Thêm một hình hộp (36 đỉnh)
pub fn push_box(out: &mut Vec<Vertex>, c: Vec3, s: Vec3, rgb: [f32; 3]) {
    let h = s * 0.5;
    let (x0, x1) = (c.x - h.x, c.x + h.x);
    let (y0, y1) = (c.y - h.y, c.y + h.y);
    let (z0, z1) = (c.z - h.z, c.z + h.z);
    quad(out, [vec3(x1, y0, z0), vec3(x1, y1, z0), vec3(x1, y1, z1), vec3(x1, y0, z1)], vec3(1.0, 0.0, 0.0), rgb);
    quad(out, [vec3(x0, y0, z1), vec3(x0, y1, z1), vec3(x0, y1, z0), vec3(x0, y0, z0)], vec3(-1.0, 0.0, 0.0), rgb);
    quad(out, [vec3(x0, y1, z0), vec3(x0, y1, z1), vec3(x1, y1, z1), vec3(x1, y1, z0)], vec3(0.0, 1.0, 0.0), rgb);
    quad(out, [vec3(x0, y0, z0), vec3(x1, y0, z0), vec3(x1, y0, z1), vec3(x0, y0, z1)], vec3(0.0, -1.0, 0.0), rgb);
    quad(out, [vec3(x0, y0, z1), vec3(x1, y0, z1), vec3(x1, y1, z1), vec3(x0, y1, z1)], vec3(0.0, 0.0, 1.0), rgb);
    quad(out, [vec3(x1, y0, z0), vec3(x0, y0, z0), vec3(x0, y1, z0), vec3(x1, y1, z0)], vec3(0.0, 0.0, -1.0), rgb);
}

/// Thêm một model OBJ đã xoay/tịnh tiến
pub fn push_model(out: &mut Vec<Vertex>, parts: &[ObjPart], m: Mat4) {
    for part in parts {
        for (p, n) in part.pos.iter().zip(part.nor.iter()) {
            let wp = m.transform_point3(*p);
            let wn = m.transform_vector3(*n).normalize_or_zero();
            out.push(vtx(wp, shade(part.color, wn)));
        }
    }
}

pub fn mesh_from(vertices: Vec<Vertex>) -> Mesh {
    let indices = (0..vertices.len() as u16).collect();
    Mesh {
        vertices,
        indices,
        texture: None,
    }
}

/// Chia danh sách đỉnh thành các mesh không vượt MAX_VERTS
pub fn chunk_meshes(verts: Vec<Vertex>) -> Vec<Mesh> {
    verts.chunks(MAX_VERTS).map(|c| mesh_from(c.to_vec())).collect()
}

pub fn draw_verts(verts: Vec<Vertex>) {
    for m in chunk_meshes(verts) {
        draw_mesh(&m);
    }
}

/// Gom hình hộp tĩnh (đường, vỉa hè) thành các mesh
pub struct Batch {
    meshes: Vec<Mesh>,
    cur: Vec<Vertex>,
}

impl Batch {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            cur: Vec::new(),
        }
    }

    pub fn push_box(&mut self, c: Vec3, s: Vec3, rgb: [f32; 3]) {
        if self.cur.len() + 36 > MAX_VERTS {
            self.flush();
        }
        push_box(&mut self.cur, c, s, rgb);
    }

    fn flush(&mut self) {
        if !self.cur.is_empty() {
            let v = std::mem::take(&mut self.cur);
            self.meshes.push(mesh_from(v));
        }
    }

    pub fn finish(mut self) -> Vec<Mesh> {
        self.flush();
        self.meshes
    }
}