use macroquad::prelude::*;

use crate::obj::{load_obj, ObjPart};
use crate::vehicle::Kind;

const MODEL_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models");

/// Một model OBJ kèm khung bao quanh (dùng cho bóng đổ)
pub struct Model {
    pub parts: Vec<ObjPart>,
    mn: Vec3,
    mx: Vec3,
}

impl Model {
    fn load(name: &str) -> Self {
        let path = format!("{MODEL_DIR}/{name}.obj");
        let parts = load_obj(&path).unwrap_or_else(|e| panic!("Không đọc được {path}: {e}"));
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
            car: Model::load("car"),
            bike: Model::load("bike"),
            plane: Model::load("plane"),
            tank: Model::load("tank"),
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