use anyhow::Context;
use imanot::{ImageStateLoaded, PixelArea, PixelAreaStack};
use imask::SortedRanges;
use imbuf::{DynamicImage, Image};
use pilatus_engineering::image::{AnyMultiMap, ImageWithMeta, MetaImageDecoder, StreamImageError};

type RgbImage = imbuf::Image<[u8; 3], 1>;

/// Runs once the image is shown, see [`imanot::PendingImage::with_callback`]
pub type ShownCallback = Box<dyn FnOnce(&mut ImageStateLoaded)>;

/// An image as displayed by the viewer, with what comes along with it.
pub struct ExtractedImage {
    pub image: RgbImage,
    /// The overlays drawn on the image, see [`PixelAreaStack`]
    pub stack: PixelAreaStack,
    /// Runs when the image is shown, and is dropped unrun if it never makes
    /// it. It can compute things from the shown image (e.g. add a layer)
    /// instead of doing that for every frame of the stream.
    pub callback: Option<ShownCallback>,
}

impl ExtractedImage {
    pub fn new(image: RgbImage, stack: PixelAreaStack) -> Self {
        Self {
            image,
            stack,
            callback: None,
        }
    }

    #[must_use]
    pub fn with_callback(mut self, callback: impl FnOnce(&mut ImageStateLoaded) + 'static) -> Self {
        self.callback = Some(Box::new(callback));
        self
    }
}

/// Callback that converts a streamed image into the rgb image displayed by
/// the viewer together with the [`PixelAreaStack`] it draws as overlays.
pub type ExtractImage = Box<
    dyn FnMut(
        Result<ImageWithMeta<DynamicImage>, StreamImageError<DynamicImage>>,
    ) -> anyhow::Result<Option<ExtractedImage>>,
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
) -> anyhow::Result<Option<ExtractedImage>> {
    let mut meta_image = extract_imanot_or_fallback(decoded)?;
    let image = meta_image.image;
    let stack = imanot::PixelAreaStack::from_sparse_layers(extract_from_extensions(
        &mut meta_image.extensions,
        [0, 0, 255, 128],
    ));
    Ok(Some(ExtractedImage::new(image, stack)))
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

fn extract_rgb(img: imbuf::DynamicImage) -> anyhow::Result<RgbImage> {
    let first = img.first();
    let image = match (
        first.primitive_size().get(),
        first.pixel_elements().get(),
        img.len(),
    ) {
        (1, 1, 1) => {
            let buffer = first.buffer_flat::<u8>()?;
            imbuf::Image::<[u8; 3], 1>::new_arc(
                buffer.iter().map(|&c| [c; 3]).collect(),
                first.width(),
                first.height(),
            )?
        }
        (1, 3, 1) => {
            imbuf::Image::<[u8; 3], 1>::try_from(img).expect("Checked dimensions in match")
        }
        x => return Err(anyhow::anyhow!("Unknown image format {x:?}")),
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
}
