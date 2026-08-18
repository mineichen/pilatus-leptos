use std::{collections::BTreeSet, ops::Range};

use anyhow::Context;
use imanot::PixelArea;
use imask::{ImageDimension, ImaskSet, SortedRanges};
use imbuf::{DynamicImage, DynamicImageChannel, ImageChannel};
use pilatus_engineering::image::{AnyMultiMap, ImageWithMeta, MetaImageDecoder, StreamImageError};

type RgbImage = imbuf::Image<[u8; 3], 1>;

/// Returns `Ok(None)` for MissingFrame error
/// Inefficient image buffer copy is tolerated, as imanot is expected to change to DynamicImage for it's original image soon
pub fn parse(
    input: &[u8],
    decoder: &MetaImageDecoder,
) -> anyhow::Result<Result<ImageWithMeta<DynamicImage>, StreamImageError<DynamicImage>>> {
    let time_before = chrono::Utc::now().timestamp_millis();
    leptos::logging::log!("before pilatus-engineering::decode {time_before}");
    let decoded = decoder.decode(input);
    leptos::logging::log!(
        "pilatus-engineering::decode took {:?}ms",
        chrono::Utc::now().timestamp_millis() - time_before
    );
    decoded
}

pub(crate) fn into_rgb(
    decoded: Result<ImageWithMeta<imbuf::DynamicImage>, StreamImageError<imbuf::DynamicImage>>,
) -> Result<Result<ImageWithMeta<RgbImage>, StreamImageError<RgbImage>>, anyhow::Error> {
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
        x => return Err(anyhow::anyhow!("Unkonwn image format {x:?}")),
    };
    Ok(image)
}
pub fn extract_from_extensions(
    extensions: &mut AnyMultiMap,
    rgba: [u8; 4],
) -> impl Iterator<Item = (usize, PixelArea)> {
    let mut layers = extensions
        .iter_extract::<LayerOverlay>()
        .map(|x| (x.layer, x.pixel_area))
        .fuse();

    let mut rest = extensions
        .iter_extract::<SortedRanges<u32>>()
        .map(move |ranges| PixelArea::from_ranges(ranges, rgba))
        .chain(extensions.iter_extract::<PixelArea>());
    let mut seen_layers = BTreeSet::<usize>::new();
    std::iter::from_fn(move || match layers.next() {
        Some((k, v)) => {
            seen_layers.insert(k);
            Some((k, v))
        }
        None => rest.next().map(|ranges| {
            let mut iter = seen_layers.iter().copied();
            let next_layer = iter
                .next()
                .map(|mut last| {
                    while let Some(x) = iter.next()
                        && x - 1 == last
                    {
                        last = x;
                    }
                    last + 1
                })
                .unwrap_or(0);
            leptos::logging::log!("Next layer: {next_layer}");
            seen_layers.insert(next_layer);
            (next_layer, ranges)
        }),
    })
}

/// An overlay to be placed on an explicit layer index.
///
/// Unlike the dense overlays extracted by [`extract_from_extensions`], these
/// keep their layer index, so layers in between may stay empty.
#[derive(Debug, Clone)]
pub struct LayerOverlay {
    pub layer: usize,
    pub pixel_area: PixelArea,
}
