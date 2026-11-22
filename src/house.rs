use macroquad::prelude::*;

use crate::obj::parse_obj;
use crate::render::push_model;

const BOX_OBJ: &str = include_str!("../assets/models/house/house_box.obj");
const MTL: &str = include_str!("../assets/models/house/house.mtl");

/// Than nha (ke ca toa cao) la mot khoi hop duy nhat, mai bang, tang tren bang tang duoi.
/// Mau cua model duoc nhan voi `rgb`, nen van giu duoc mau va do toi khi bi pha.
pub fn body(out: &mut Vec<Vertex>, c: Vec3, s: Vec3, rgb: [f32; 3]) {
    let parts = parse_obj(BOX_OBJ, MTL);
    let m = Mat4::from_translation(vec3(c.x, c.y - s.y * 0.5, c.z)) * Mat4::from_scale(s);
    let start = out.len();
    push_model(out, &parts, m);
    for v in &mut out[start..] {
        for ch in 0..3 {
            v.color[ch] = ((v.color[ch] as f32) * rgb[ch]).min(255.0) as u8;
        }
    }
}
