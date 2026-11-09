use crate::ShaderKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum BindGroupLayoutKind {
    Shader {
        kind: ShaderKind,
        // Has custom uniform
        uniform: bool,
        textures: usize,
    },
    Uniform,
}

pub(crate) fn get_bind_group_layout(
    device: &wgpu::Device,
    kind: BindGroupLayoutKind,
) -> wgpu::BindGroupLayout {
    match kind {
        BindGroupLayoutKind::Uniform => get_uniform_bind_group_layout(device),
        BindGroupLayoutKind::Shader {
            kind: ShaderKind::Sprite,
            uniform,
            textures,
        } => get_sprite_bind_group_layout(device, uniform, textures),
        BindGroupLayoutKind::Shader {
            kind: ShaderKind::PostProcess,
            uniform,
            textures,
        } => get_post_process_bind_group_layout(device, uniform, textures > 0),
    }
}

fn get_uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Uniform bind group layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn get_sprite_bind_group_layout(
    device: &wgpu::Device,
    uniform: bool,
    textures: usize,
) -> wgpu::BindGroupLayout {
    let mut entries = vec![];
    if textures > 0 {
        // Shared sampler
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        for i in 0..textures {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32 + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });
        }
    }

    if uniform {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: entries.len() as u32,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    }

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &entries,
        label: Some("Sprite Bind Group Layout"),
    })
}

fn get_post_process_bind_group_layout(
    device: &wgpu::Device,
    uniform: bool,
    texture: bool,
) -> wgpu::BindGroupLayout {
    let mut entries = vec![
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        // input view
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
    ];

    if texture {
        // TODO reuse sampler?
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        entries.push(
            // texture
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
        );
    }

    if uniform {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: entries.len() as u32,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    }

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &entries,
        label: Some("PostProcess Bind Group Layout"),
    })
}
