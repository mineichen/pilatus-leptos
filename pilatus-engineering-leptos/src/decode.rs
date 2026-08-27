use anyhow::Context;
use imanot::{PixelArea, PixelAreaStack};
use imask::SortedRanges;
use imbuf::{DynamicImage, DynamicImageChannel, Image, ImageChannel};
use pilatus_engineering::image::{AnyMultiMap, ImageWithMeta, MetaImageDecoder, StreamImageError};

type RgbImage = imbuf::Image<[u8; 3], 1>;

/// Callback that converts a streamed image into the rgb image displayed by
/// the viewer together with the [`PixelAreaStack`] it draws as overlays.
pub type ExtractImage = Box<
    dyn FnMut(
        Result<ImageWithMeta<DynamicImage>, StreamImageError<DynamicImage>>,
    ) -> anyhow::Result<Option<(RgbImage, PixelAreaStack)>>,
>;

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

pub fn extract_imanot_with_stack(
    decoded: Result<ImageWithMeta<DynamicImage>, StreamImageError<DynamicImage>>,
) -> anyhow::Result<Option<(Image<[u8; 3], 1>, PixelAreaStack)>> {
    let mut meta_image = extract_imanot_or_fallback(decoded)?;
    let image = meta_image.image;
    let stack = imanot::PixelAreaStack::from_iter(extract_from_extensions(
        &mut meta_image.extensions,
        [0, 0, 255, 128],
    ));
    Ok(Some((image, stack)))
}

/// Converts the image to rgb, falling back to the plain [`ImageWithMeta`]
/// (without extensions) of a [`StreamImageError::ProcessingError`], so its
/// image can still be displayed. Other stream errors are propagated.
pub fn extract_imanot_or_fallback(
    decoded: Result<ImageWithMeta<DynamicImage>, StreamImageError<DynamicImage>>,
) -> anyhow::Result<ImageWithMeta<Image<[u8; 3], 1>>> {
    match extract_imanot(decoded)? {
        Ok(meta_image) => Ok(meta_image),
        Err(StreamImageError::ProcessingError { image, .. }) => {
            Ok(ImageWithMeta::with_hash(image, None))
        }
        Err(e) => Err(e.into()),
    }
}

pub fn extract_imanot(
    decoded: Result<ImageWithMeta<DynamicImage>, StreamImageError<DynamicImage>>,
) -> anyhow::Result<Result<ImageWithMeta<RgbImage>, StreamImageError<RgbImage>>> {
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
/// Assigns the dense overlays (extensions of [`SortedRanges`] and
/// [`PixelArea`]) to the layers 0..n, in that order.
pub fn extract_from_extensions(
    extensions: &mut AnyMultiMap,
    rgba: [u8; 4],
) -> impl Iterator<Item = (usize, PixelArea)> {
    extensions
        .iter_extract::<SortedRanges<u32>>()
        .map(move |ranges| PixelArea::from_ranges(ranges, rgba))
        .chain(extensions.iter_extract::<PixelArea>())
        .enumerate()
        .map(|(layer, pixel_area)| (layer, pixel_area))
}
