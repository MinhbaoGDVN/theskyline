use macroquad::miniquad::conf::Icon;

const ICON_PNG: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/icon.png"));

fn resized<const N: usize>(img: &image::RgbaImage, size: u32) -> [u8; N] {
    let r = image::imageops::resize(img, size, size, image::imageops::FilterType::Lanczos3);
    let mut out = [0u8; N];
    out.copy_from_slice(r.as_raw());
    out
}

/// Icon cửa sổ game (16, 32, 64 px), lấy từ file PNG
pub fn window_icon() -> Option<Icon> {
    let img = image::load_from_memory(ICON_PNG).ok()?.to_rgba8();
    Some(Icon {
        small: resized::<1024>(&img, 16),
        medium: resized::<4096>(&img, 32),
        big: resized::<16384>(&img, 64),
    })
}