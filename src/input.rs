use macroquad::prelude::*;

/// Phím giữ (đọc mỗi bước logic)
pub struct Controls {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub thr_up: bool,
    pub thr_dn: bool,
    pub brake: bool,
    pub fire: bool,
    pub gear_up: bool,
    pub gear_down: bool,
    pub auto_toggle: bool,
}

pub fn read() -> Controls {
    Controls {
        up: is_key_down(KeyCode::W) || is_key_down(KeyCode::Up),
        down: is_key_down(KeyCode::S) || is_key_down(KeyCode::Down),
        left: is_key_down(KeyCode::A) || is_key_down(KeyCode::Left),
        right: is_key_down(KeyCode::D) || is_key_down(KeyCode::Right),
        thr_up: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
        thr_dn: is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl),
        brake: is_key_down(KeyCode::Space),
        fire: is_key_down(KeyCode::F),
        gear_up: false,
        gear_down: false,
        auto_toggle: false,
    }
}

/// Phím bấm (chỉ ghi nhận một lần mỗi frame, dùng trong bước logic đầu tiên)
#[derive(Default)]
pub struct Edges {
    pub switch: Option<usize>,
    pub next: bool,
    pub prev: bool,
    pub reset: bool,
    pub gear_up: bool,
    pub gear_down: bool,
    pub auto_toggle: bool,
    pub dev_toggle: bool,
}

impl Edges {
    pub fn merge(&mut self, o: &Edges) {
        if o.switch.is_some() {
            self.switch = o.switch;
        }
        self.next |= o.next;
        self.prev |= o.prev;
        self.reset |= o.reset;
        self.gear_up |= o.gear_up;
        self.gear_down |= o.gear_down;
        self.auto_toggle |= o.auto_toggle;
        self.dev_toggle |= o.dev_toggle;
    }
}

pub fn read_edges() -> Edges {
    let digits = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4];
    let mut e = Edges::default();
    for (i, k) in digits.iter().enumerate() {
        if is_key_pressed(*k) {
            e.switch = Some(i);
        }
    }
    e.next = is_key_pressed(KeyCode::E);
    e.prev = is_key_pressed(KeyCode::Q);
    e.reset = is_key_pressed(KeyCode::R);
    e.gear_up = is_key_pressed(KeyCode::X);
    e.gear_down = is_key_pressed(KeyCode::Z);
    e.auto_toggle = is_key_pressed(KeyCode::T);
    // Shift + D + F12: bật/tắt dev mode
    let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
    e.dev_toggle = shift && is_key_down(KeyCode::D) && is_key_pressed(KeyCode::F12);
    e
}

/// Cai dat: dao nguoc dieu khien len/xuong cua may bay
pub static INVERT_PLANE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn invert_plane() -> bool {
    INVERT_PLANE.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn set_invert_plane(v: bool) {
    INVERT_PLANE.store(v, std::sync::atomic::Ordering::Relaxed);
}