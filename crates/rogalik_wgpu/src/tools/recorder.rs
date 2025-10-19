use std::{
    io::{pipe, Write},
    process::Command,
};

#[derive(Default)]
pub(crate) struct Recorder {
    buffer: Option<wgpu::Buffer>,
    frames: Vec<u8>,
    is_recording: bool,
    width: u32,
    height: u32,
}
impl Recorder {
    pub(crate) fn toggle_recording(&mut self) {
        if self.is_recording {
            self.is_recording = false;
            log::info!("Recording disabled");

            self.save_video(&format!(
                "{}.mp4",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ));
            return;
        }

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

        if self.buffer.is_none() {
            self.create_buffer(width, height, device);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Recording encoder"),
        });

        let (_, padded_bytes_per_row) = Self::get_bytes_per_row(width);

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
                    bytes_per_row: Some(padded_bytes_per_row),
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
            self.frames.extend(data.iter().copied());
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
        let (_, padded_bytes_per_row) = Self::get_bytes_per_row(width);
        (padded_bytes_per_row * height) as wgpu::BufferAddress
    }
    fn get_bytes_per_row(width: u32) -> (u32, u32) {
        let pixel_size = std::mem::size_of::<[u8; 4]>() as u32;
        let bytes_per_row = pixel_size * width;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padding = (align - bytes_per_row % align) % align;
        (bytes_per_row, bytes_per_row + padding)
    }
    fn save_video(&mut self, path: &str) {
        let frames = self.frames.clone();
        let width = self.width;
        let height = self.height;
        let path = path.to_string();

        let (bytes_per_row, padded_bytes_per_row) = Self::get_bytes_per_row(width);

        let _ = std::thread::spawn(move || {
            let (pipe_reader, mut pipe_writer) = pipe().unwrap();
            let mut cmd = Command::new("ffmpeg")
                .args([
                    "-f",
                    "rawvideo",
                    "-video_size",
                    &format!("{width}x{height}"),
                    "-pixel_format",
                    "rgb32",
                    "-r",
                    "60",
                    "-i",
                    "pipe:",
                    "-c:v",
                    "h264",
                    &path,
                ])
                .stdin(pipe_reader)
                .spawn()
                .unwrap();

            frames
                .chunks(padded_bytes_per_row as usize)
                .map(|a| &a[..bytes_per_row as usize])
                .for_each(|a| pipe_writer.write_all(a).unwrap());

            drop(pipe_writer);
            cmd.wait().unwrap();
            log::info!("Video file saved");
        });
    }
}
