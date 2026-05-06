use std::fs::File;
use std::io::BufReader;

use crate::app::error::Error;

pub fn load_png_texture(ctx: &egui::Context, path: &str) -> Result<egui::TextureHandle, Error> {
    
    let file = File::open(path)
        .map_err(|e| Error::StdIo(e.to_string()))?;

    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|e| Error::App(e.to_string()))?;

    let buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| Error::App("Failed to determine PNG buffer size".into()))?;

    let mut buf = vec![0; buffer_size];

    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| Error::App(e.to_string()))?;

    let bytes = &buf[..info.buffer_size()];

    let rgba = match info.color_type {
        png::ColorType::Rgba => bytes.to_vec(),

        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity((info.width * info.height * 4) as usize);
            for chunk in bytes.chunks_exact(3) {
                out.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            out
        }

        other => {
            return Err(Error::App(format!(
                "Unsupported PNG color type: {:?}",
                other
            )));
        }
    };

    let image = egui::ColorImage::from_rgba_unmultiplied(
        [info.width as usize, info.height as usize],
        &rgba,
    );

    Ok(ctx.load_texture(
        path,
        image,
        egui::TextureOptions::NEAREST,
    ))
}