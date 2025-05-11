use rogalik_common::{Color, TextureFiltering, TextureRepeat};

pub fn color_to_wgpu(color: Color) -> wgpu::Color {
    let col = color.as_srgb();
    wgpu::Color {
        r: col[0] as f64,
        g: col[1] as f64,
        b: col[2] as f64,
        a: col[3] as f64,
    }
}

pub(crate) fn get_wgpu_address_mode(repeat: TextureRepeat) -> wgpu::AddressMode {
    match repeat {
        TextureRepeat::Clamp => wgpu::AddressMode::ClampToEdge,
        TextureRepeat::Repeat => wgpu::AddressMode::Repeat,
        TextureRepeat::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
    }
}

pub(crate) fn get_wgpu_filter_mode(filtering: TextureFiltering) -> wgpu::FilterMode {
    match filtering {
        TextureFiltering::Nearest => wgpu::FilterMode::Nearest,
        TextureFiltering::Linear => wgpu::FilterMode::Linear,
    }
}
