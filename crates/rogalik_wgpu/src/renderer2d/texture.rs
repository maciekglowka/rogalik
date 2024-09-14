use crate::assets::texture::TextureData;

pub fn get_texture_bind_group(
    texture_data: &TextureData,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    let texture = wgpu_texture_from_bytes(texture_data, device, queue);
    let diff_tex_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let diff_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diff_tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diff_sampler),
            },
        ],
        label: Some("Sprite Diffuse Bind Group"),
    })
}

fn wgpu_texture_from_bytes(
    texture_data: &TextureData,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width: texture_data.dim.0,
        height: texture_data.dim.1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("Texture"),
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &texture_data.buffer,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * texture_data.dim.0),
            rows_per_image: Some(texture_data.dim.1),
        },
        size,
    );
    texture
}
