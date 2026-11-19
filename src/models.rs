use macroquad::prelude::*;

use crate::obj::{parse_obj, ObjPart};
use crate::vehicle::Kind;

const MTL: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/palette.mtl"));
const CAR: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/car.obj"));
const BIKE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/bike.obj"));
const PLANE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/plane.obj"));
const TANK: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/tank.obj"));

/// Một model OBJ kèm khung bao (dùng cho bóng đổ)
pub struct Model {
    pub parts: Vec<ObjPart>,
    mn: Vec3,
    mx: Vec3,
}

impl Model {
    fn load(obj: &str) -> Self {
        let parts = parse_obj(obj, MTL);
        let mut mn = Vec3::splat(f32::MAX);
        let mut mx = Vec3::splat(f32::MIN);
        for p in &parts {
            for v in &p.pos {
                mn = mn.min(*v);
                mx = mx.max(*v);
            }
        }
        Self { parts, mn, mx }
    }

    pub fn bounds(&self) -> (Vec3, Vec3) {
        (self.mn, self.mx)
    }
}

pub struct Models {
    car: Model,
    bike: Model,
    plane: Model,
    tank: Model,
}

impl Models {
    pub fn load() -> Self {
        Self {
            car: Model::load(CAR),
            bike: Model::load(BIKE),
            plane: Model::load(PLANE),
            tank: Model::load(TANK),
        }
    }

    pub fn of(&self, kind: Kind) -> &Model {
        match kind {
            Kind::Car => &self.car,
            Kind::Bike => &self.bike,
            Kind::Plane => &self.plane,
            Kind::Tank => &self.tank,
        }
    }

    /// Xe NPC dùng chung model ô tô
    pub fn car(&self) -> &Model {
        &self.car
    }
}