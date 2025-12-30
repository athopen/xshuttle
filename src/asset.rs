use image::{RgbaImage, load_from_memory};

pub fn load_icon() -> RgbaImage {
    load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("Failed to load icon")
        .into_rgba8()
}
