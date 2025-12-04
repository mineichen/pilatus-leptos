use std::num::NonZeroU32;

use anyhow::{Context, anyhow};
use leptos::logging::{debug_log, log};

pub fn parse(input: &[u8]) -> anyhow::Result<Option<egui_pixels::RgbImageInterleaved<u8>>> {
    if input.len() < 8 {
        return Err(anyhow!("Header is {} bytes long", input.len()).into());
    }

    const MISSING_FRAME_ERROR: u8 = 16;

    match input[0] {
        0 => {}
        MISSING_FRAME_ERROR => return Ok(None),
        x => {
            return Err(anyhow!("Stream item with error: {}", x).into());
        }
    }
    let meta_len = u32::from_le_bytes(array(&input[4..8]));

    let meta_bytes = input.get(8..meta_len as usize + 8).ok_or_else(|| {
        anyhow!(
            "Metadata out of bounds. expected: {}, remaining-input: {}",
            meta_len,
            input.len(),
        )
    })?;
    debug_log!("Meta: {:?}", String::from_utf8_lossy(meta_bytes));
    let size_without_image_data: usize = 4 + 4 + meta_len as usize + 4;
    //let metadata = dbg!(&input.get(ok_header_len..ok_header_len + 10));
    if input.len() < size_without_image_data + 12 {
        return Err(anyhow!("Before image is not long enouth: {}", input.len()).into());
    }

    let size = u32::from_le_bytes(array(
        &input[size_without_image_data - 4..size_without_image_data],
    ));

    // align to 8byte
    let image_buf_start = (size_without_image_data + 7) & !7;
    let align_bytes = (image_buf_start - size_without_image_data) as u32;

    // +0 is reserved
    let (kind, width, image_buf_len, height) =
        read_raw(&input, size, image_buf_start, align_bytes)?;

    let pixels = &input[image_buf_start + 8..image_buf_start + 8 + image_buf_len as usize];

    const SCALE: f32 = 0.01;
    let width_offset = (width.get() / 2) as isize;
    let height_offset = (height.get() / 2) as isize;
    match kind {
        0 => {
            log!("Encode u8");

            Ok(Some(egui_pixels::RgbImageInterleaved::new_arc(
                pixels.iter().map(|&c| [c; 3]).collect(),
                width,
                height,
            )))
        }
        1 => {
            // pixels //16bit
            //     .chunks_exact(2)
            //     .enumerate()
            //     .map(|(pos, x)| InstanceData {
            //         position: Vec3::new(
            //             (pos as isize / width.get() as isize - height_offset) as f32 * SCALE,
            //             u16::from_le_bytes(array(x)) as f32 * SCALE * 0.01,
            //             (pos as isize % width.get() as isize - width_offset) as f32 * SCALE,
            //         ),
            //         scale: settings.scale,
            //         color: [0.5, 0.5, 0.5, 0.5],
            //     })
            //     .collect();
            log!("Encode u16");
            Ok(None)
        }
        x => Err(anyhow::anyhow!("Unkonwn image format {x}").into()),
    }
}

fn array<T: Copy, const N: usize>(slice: &[T]) -> [T; N] {
    slice.try_into().expect("incorrect_length")
}
fn read_raw(
    input: &[u8],
    size: u32,
    image_buf_start: usize,
    align_bytes: u32,
) -> anyhow::Result<(u8, std::num::NonZero<u32>, u32, std::num::NonZero<u32>)> {
    let kind = input[image_buf_start + 1];
    let channels = u16::from_le_bytes(array(&input[image_buf_start + 2..image_buf_start + 4]));
    let width: NonZeroU32 =
        u32::from_le_bytes(array(&input[image_buf_start + 4..image_buf_start + 8]))
            .try_into()
            .context("width")?;
    let image_buf_len = size - 8 - align_bytes;
    let pixel_count = match kind {
        0 => image_buf_len,
        1 => image_buf_len / 2,
        _ => return Err(anyhow!("Unknown kind {kind}").into()),
    };
    let height: NonZeroU32 = (pixel_count / width).try_into()?;
    if pixel_count % width != 0 {
        return Err(anyhow!("Expected remainer of 0 {pixel_count}h {width}w").into());
    }
    Ok((kind, width, image_buf_len, height))
}
