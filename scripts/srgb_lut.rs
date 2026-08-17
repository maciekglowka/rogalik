fn u8_to_srgb(i: u8) -> f32 {
    (((i as f32 / 255.0) + 0.055) / 1.055).powf(2.4)
}

fn main() {
    println!("[");
    (0..=255).for_each(|i| println!("  {},", u8_to_srgb(i)));
    println!("]");
}
