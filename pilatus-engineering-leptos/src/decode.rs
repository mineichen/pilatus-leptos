use std::ops::Range;

use anyhow::Context;
use imanot::PixelArea;
use imask::{ImageDimension, ImaskSet, SortedRanges, SortedRangesMap};
use imbuf::{DynamicImageChannel, ImageChannel};
use pilatus_engineering::image::{AnyMultiMap, ImageWithMeta, StreamImageError};

/// Returns `Ok(None)` for MissingFrame error
/// Inefficient image buffer copy is tolerated, as imanot is expected to change to DynamicImage for it's original image soon
pub fn parse(
    input: &[u8],
) -> anyhow::Result<
    Result<ImageWithMeta<imbuf::Image<[u8; 3], 1>>, StreamImageError<imbuf::Image<[u8; 3], 1>>>,
> {
    let time_before = chrono::Utc::now().timestamp_millis();
    leptos::logging::log!("before pilatus-engineering::decode {time_before}");
    let decoded = pilatus_engineering::image::decode(input)?;
    leptos::logging::log!(
        "pilatus-engineering::decode took {:?}ms",
        chrono::Utc::now().timestamp_millis() - time_before
    );
    match decoded {
        Ok(img) => {
            let meta = img.meta;
            let ext = img.extensions;
            let image = extract_rgb(img.image)?;
            let mut image_with_meta = ImageWithMeta::with_meta(image, meta);
            image_with_meta.extensions = ext;
            Ok(Ok(image_with_meta))
        }
        #[expect(deprecated)]
        Err(StreamImageError::MissedItems(x)) => Ok(Err(StreamImageError::MissedItems(x))),
        Err(StreamImageError::ProcessingError { image, error }) => {
            Ok(Err(StreamImageError::ProcessingError {
                image: extract_rgb(image).context("Extract ProcessingError image")?,
                error,
            }))
        }
        Err(e) => Err(e.into()),
    }
}

fn extract_rgb(
    img: imbuf::DynamicImage,
) -> anyhow::Result<imbuf::ImageChannels<[ImageChannel<[u8; 3]>; 1]>> {
    let first = img.first();
    let image = match (first, first.pixel_elements().get(), img.len()) {
        (DynamicImageChannel::U8(ch), 1, 1) => imbuf::Image::<[u8; 3], 1>::new_arc(
            ch.buffer_flat().iter().map(|&c| [c; 3]).collect(),
            ch.width(),
            ch.height(),
        ),
        (DynamicImageChannel::U8(_), 3, 1) => {
            imbuf::Image::<[u8; 3], 1>::try_from(img).expect("Checked dimensions in match")
        }
        // (DynamicImageChannel::U16(_), 1, 1) => {
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
        //     log!("Encode u16");
        //     Ok(None)
        // }
        x => return Err(anyhow::anyhow!("Unkonwn image format {x:?}")),
    };
    Ok(image)
}
pub fn extract_from_extensions(
    extensions: &mut AnyMultiMap,
    opacity: u8,
    color: [u8; 3],
) -> Vec<PixelArea> {
    extensions
        .iter::<SortedRanges<u64, u64>>()
        .map(|x| {
            let width = x.bounds().len_x();
            let iter = x.iter_global_with::<Range<u32>>(width);
            let roi = iter.bounds();
            let ranges = SortedRangesMap::try_from_ordered_iter(
                iter.map(|x| (x, imanot::Meta::new(opacity))).with_roi(roi),
            )
            .expect("Always sorted and not empty");
            PixelArea::from_ranges(ranges, color)
        })
        .collect()
}
