use macroquad::prelude::*;
use std::f32::consts::FRAC_PI_2;

use crate::obj::parse_obj;
use crate::render::push_model;

const ROAD_OBJ: &str = include_str!("../assets/models/road.obj");
const ROAD_MTL: &str = include_str!("../assets/models/road.mtl");

/// Ve cac duong luoi bang model road.obj. Moi gia tri trong `lines` la mot duong:
/// duong doc theo z tai x = p, va duong ngang theo x tai z = p. Dai `len`, rong `width`.
pub fn grid_roads(out: &mut Vec<Vertex>, lines: &[f32], len: f32, width: f32) {
    let parts = parse_obj(ROAD_OBJ, ROAD_MTL);
    for &p in lines {
        let along_z = Mat4::from_translation(vec3(p, 0.0, 0.0)) * Mat4::from_scale(vec3(width, 1.0, len));
        push_model(out, &parts, along_z);

        let along_x = Mat4::from_translation(vec3(0.0, 0.0, p))
            * Mat4::from_rotation_y(FRAC_PI_2)
            * Mat4::from_scale(vec3(width, 1.0, len));
        push_model(out, &parts, along_x);
    }
}
