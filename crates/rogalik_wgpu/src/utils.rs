use rogalik_common::Color;

pub fn color_to_wgpu(color: Color) -> wgpu::Color {
    let col = color.as_srgb();
    wgpu::Color {
        r: col[0] as f64,
        g: col[1] as f64,
        b: col[2] as f64,
        a: col[3] as f64,
    }
}
