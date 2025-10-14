#[derive(Default)]
pub(crate) struct Recorder {
    buffer: Option<wgpu::Buffer>,
    frames: Vec<Vec<u8>>,
    is_recording: bool,
    width: u32,
    height: u32,
}
impl Recorder {
    pub(crate) fn toggle_recording(&mut self) {
        if self.is_recording {
            self.is_recording = false;
            log::info!("Recording disabled");
            self.save_gif(&format!(
                "{}.gif",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ));
            return;
        }

        self.frames.clear();
        self.is_recording = true;
        log::info!("Recording enabled");
    }
    pub(crate) fn handle_queue(
        &mut self,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output: &wgpu::SurfaceTexture,
    ) {
        if !self.is_recording {
            return;
        };

        if self.buffer.is_none() || width != self.width || height != self.height {
            self.create_buffer(width, height, device);
            self.frames.clear();
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Recording encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &output.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: self.buffer.as_ref().unwrap(),
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(Self::get_bytes_per_row(width)),
                    rows_per_image: Some(height),
                },
            },
            output.texture.size(),
        );

        queue.submit(std::iter::once(encoder.finish()));

        {
            let buffer_slice = self.buffer.as_ref().unwrap().slice(..);
            buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

            device.poll(wgpu::Maintain::Wait);
            let data = buffer_slice.get_mapped_range();
            self.frames.push(data.iter().copied().collect());
        }

        self.buffer.as_ref().unwrap().unmap();
    }
    fn create_buffer(&mut self, width: u32, height: u32, device: &wgpu::Device) {
        let buffer_desc = wgpu::BufferDescriptor {
            size: Self::get_buffer_size(width, height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: Some("Recording buffer"),
            mapped_at_creation: false,
        };
        self.buffer = Some(device.create_buffer(&buffer_desc));
        self.width = width;
        self.height = height;
    }
    fn get_buffer_size(width: u32, height: u32) -> wgpu::BufferAddress {
        (Self::get_bytes_per_row(width) * height) as wgpu::BufferAddress
    }
    fn get_bytes_per_row(width: u32) -> u32 {
        let pixel_size = std::mem::size_of::<[u8; 4]>() as u32;
        let bytes_per_row = pixel_size * width;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padding = (align - bytes_per_row % align) % align;
        bytes_per_row + padding
    }
    fn save_gif(&mut self, path: &str) {
        let mut frames = self.frames.clone();
        let path = path.to_string();
        let width = self.width;
        let height = self.height;

        let _ = std::thread::spawn(move || {
            let mut image = std::fs::File::create(path).unwrap();
            let mut encoder =
                gif::Encoder::new(&mut image, width as u16, height as u16, &[]).unwrap();
            let padded_bytes_per_row = Self::get_bytes_per_row(width);
            let bytes_per_row = std::mem::size_of::<[u8; 4]>() as u32 * width;

            let mut buf = Vec::with_capacity((bytes_per_row * height) as usize);

            for frame_data in frames.iter_mut() {
                // Remove padding bytes
                buf.clear();
                frame_data
                    .chunks(padded_bytes_per_row as usize)
                    .map(|a| &a[..bytes_per_row as usize])
                    .for_each(|a| buf.extend(a));

                let mut frame =
                    gif::Frame::from_rgba_speed(width as u16, height as u16, &mut buf, 20);
                frame.delay = 1;
                let _ = encoder.write_frame(&frame);
            }
            log::info!("Gif file saved");
        });
    }
}
