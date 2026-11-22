use std::collections::HashMap;

use macroquad::prelude::Vec3;

/// Một bộ phận của model. Đỉnh không dùng chỉ số, viết theo từng tam giác.
pub struct ObjPart {
    pub color: [f32; 3],
    pub pos: Vec<Vec3>,
    pub nor: Vec<Vec3>,
}

#[derive(Default)]
struct Current {
    color: [f32; 3],
    tris: Vec<[Vec3; 3]>,
}

/// Đọc model OBJ từ chuỗi; màu lấy từ bảng MTL (chuỗi)
pub fn parse_obj(text: &str, mtl_text: &str) -> Vec<ObjPart> {
    let mtl = load_mtl(mtl_text);
    let mut verts: Vec<Vec3> = Vec::new();
    let mut parts = Vec::new();
    let mut cur = Current {
        color: [0.8, 0.8, 0.8],
        ..Default::default()
    };

    for line in text.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("v") => {
                let p: Vec<f32> = it.filter_map(|s| s.parse().ok()).collect();
                if p.len() >= 3 {
                    verts.push(Vec3::new(p[0], p[1], p[2]));
                }
            }
            Some("f") => {
                let n = verts.len() as i64;
                let idx: Vec<usize> = it
                    .filter_map(|tok| {
                        let i: i64 = tok.split('/').next()?.parse().ok()?;
                        let k = if i < 0 { n + i } else { i - 1 };
                        (k >= 0 && k < n).then_some(k as usize)
                    })
                    .collect();
                for k in 1..idx.len().saturating_sub(1) {
                    cur.tris
                        .push([verts[idx[0]], verts[idx[k]], verts[idx[k + 1]]]);
                }
            }
            Some("o") | Some("g") => flush(&mut cur, &mut parts),
            Some("usemtl") => {
                if let Some(name) = it.next() {
                    // Ket thuc mot mang mau truoc khi doi mau, de cac mat khong bi nhuom sai mau
                    flush(&mut cur, &mut parts);
                    cur.color = *mtl.get(name).unwrap_or(&[0.8, 0.8, 0.8]);
                }
            }
            _ => {}
        }
    }
    flush(&mut cur, &mut parts);
    parts
}

fn flush(cur: &mut Current, parts: &mut Vec<ObjPart>) {
    if cur.tris.is_empty() {
        return;
    }
    let sum: Vec3 = cur.tris.iter().flatten().copied().sum();
    let center = sum / (cur.tris.len() * 3) as f32;

    let mut pos = Vec::with_capacity(cur.tris.len() * 3);
    let mut nor = Vec::with_capacity(cur.tris.len() * 3);
    for t in &cur.tris {
        let mut n = (t[1] - t[0]).cross(t[2] - t[0]).normalize_or_zero();
        let tc = (t[0] + t[1] + t[2]) / 3.0;
        if n.dot(tc - center) < 0.0 {
            n = -n;
        }
        for p in t {
            pos.push(*p);
            nor.push(n);
        }
    }
    parts.push(ObjPart {
        color: cur.color,
        pos,
        nor,
    });
    cur.tris.clear();
}

fn load_mtl(text: &str) -> HashMap<String, [f32; 3]> {
    let mut map = HashMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("newmtl") => current = it.next().map(str::to_string),
            Some("Kd") => {
                let c: Vec<f32> = it.filter_map(|s| s.parse().ok()).collect();
                if let (Some(name), 3) = (&current, c.len()) {
                    map.insert(name.clone(), [c[0], c[1], c[2]]);
                }
            }
            _ => {}
        }
    }
    map
}