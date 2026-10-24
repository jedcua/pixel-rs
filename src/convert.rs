//! Converter that turns a PNG or JPG image of pixel art into a [`Document`].

use std::cmp::Reverse;
use std::collections::HashMap;
use std::fmt;
use std::num::NonZeroU32;

use image::{GenericImageView, ImageError, ImageFormat, RgbaImage};

use crate::document::{Document, PaletteEntry};
use crate::parse::Rgba;

/// The largest RGBA distance between two JPG colors that become one palette
/// color. JPG compression moves each color by a small amount.
const JPG_TOLERANCE: u32 = 32;

/// All pixels with the alpha 0 become this color, because their RGB values
/// are not visible.
const TRANSPARENT: Rgba = Rgba([0; 4]);
const TRANSPARENT_KEY: char = '.';

/// A conversion failure.
#[derive(Debug)]
pub enum ConvertError {
    /// The input is not a PNG or JPG file, or the decoder failed.
    Decode(ImageError),
    /// The image width or height is not a multiple of the scale.
    ScaleMismatch {
        width: u32,
        height: u32,
        scale: NonZeroU32,
    },
    /// The image has more colors than there are palette keys.
    TooManyColors,
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConvertError::Decode(err) => {
                write!(f, "cannot decode the image: {err}; use a PNG or JPG file")
            }
            ConvertError::ScaleMismatch {
                width,
                height,
                scale,
            } => write!(
                f,
                "image size {width}x{height} is not a multiple of scale {scale}; use a scale that divides the width and the height"
            ),
            ConvertError::TooManyColors => write!(
                f,
                "image has more colors than there are palette keys; use an image with fewer colors"
            ),
        }
    }
}

impl std::error::Error for ConvertError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConvertError::Decode(err) => Some(err),
            ConvertError::ScaleMismatch { .. } | ConvertError::TooManyColors => None,
        }
    }
}

impl From<ImageError> for ConvertError {
    fn from(err: ImageError) -> Self {
        ConvertError::Decode(err)
    }
}

/// Converts the PNG or JPG file `encoded` to a document. Each `scale` x
/// `scale` block of image pixels becomes one pixel, with the most frequent
/// color of the block.
///
/// Fully transparent pixels get the key `.`. The other colors get keys in
/// order of frequency. In a JPG, colors that are almost equal become one
/// palette color.
///
/// # Errors
///
/// Returns a [`ConvertError`] if the file is not a PNG or JPG, if `scale`
/// does not divide the image size, or if the image has too many colors.
pub fn convert(encoded: &[u8], scale: NonZeroU32) -> Result<Document, ConvertError> {
    let format = image::guess_format(encoded)?;
    let decoded = image::load_from_memory_with_format(encoded, format)?.into_rgba8();
    let size = scale.get();
    let (width, height) = decoded.dimensions();
    if width % size != 0 || height % size != 0 {
        return Err(ConvertError::ScaleMismatch {
            width,
            height,
            scale,
        });
    }
    let tolerance = if format == ImageFormat::Jpeg {
        JPG_TOLERANCE
    } else {
        0
    };
    let colors = ColorMap::new(&decoded, tolerance);
    let blocks = sample_blocks(&decoded, size, &colors);
    make_document(&colors, &blocks, width / size, height / size)
}

/// The merged colors of an image, and the merged color of each image color.
struct ColorMap {
    colors: Vec<Rgba>,
    index: HashMap<[u8; 4], usize>,
}

impl ColorMap {
    /// Merges the colors of `decoded`, most frequent first. Each color goes
    /// to the nearest merged color within `tolerance`, or becomes a new
    /// merged color.
    fn new(decoded: &RgbaImage, tolerance: u32) -> Self {
        let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
        for pixel in decoded.pixels() {
            *counts.entry(visible_color(pixel.0).0).or_insert(0) += 1;
        }
        let mut by_frequency: Vec<_> = counts.into_iter().collect();
        by_frequency.sort_unstable_by_key(|&(color, count)| (Reverse(count), color));

        let mut map = ColorMap {
            colors: Vec::new(),
            index: HashMap::with_capacity(by_frequency.len()),
        };
        for (color, _) in by_frequency {
            let index = map.nearest(Rgba(color), tolerance).unwrap_or_else(|| {
                map.colors.push(Rgba(color));
                map.colors.len() - 1
            });
            map.index.insert(color, index);
        }
        map
    }

    /// Returns the index of the merged color for the image color `rgba`.
    fn index(&self, rgba: [u8; 4]) -> usize {
        self.index[&visible_color(rgba).0]
    }

    /// Returns the index of the merged color nearest to `color`, if its
    /// distance is `tolerance` or less. The distance is the Euclidean
    /// distance of the RGBA channels.
    fn nearest(&self, color: Rgba, tolerance: u32) -> Option<usize> {
        let squared_distance = |merged: &Rgba| {
            let pairs = merged.0.into_iter().zip(color.0);
            pairs
                .map(|(a, b)| u32::from(a.abs_diff(b)).pow(2))
                .sum::<u32>()
        };
        self.colors
            .iter()
            .map(squared_distance)
            .enumerate()
            .filter(|&(_, squared)| squared <= tolerance.pow(2))
            .min_by_key(|&(_, squared)| squared)
            .map(|(index, _)| index)
    }
}

fn visible_color(rgba: [u8; 4]) -> Rgba {
    if rgba[3] == 0 {
        TRANSPARENT
    } else {
        Rgba(rgba)
    }
}

/// Returns the merged color index of each `size` x `size` block of
/// `decoded`, in row-major order. The size of `decoded` must be a multiple
/// of `size`.
fn sample_blocks(decoded: &RgbaImage, size: u32, colors: &ColorMap) -> Vec<usize> {
    let (width, height) = (decoded.width() / size, decoded.height() / size);
    let mut counts = HashMap::new();
    let mut blocks = Vec::with_capacity(width as usize * height as usize);
    for y in 0..height {
        for x in 0..width {
            let block = decoded.view(x * size, y * size, size, size);
            let indices = block.pixels().map(|(_, _, pixel)| colors.index(pixel.0));
            blocks.push(most_frequent(indices, &mut counts));
        }
    }
    blocks
}

/// Returns the index that occurs most in `indices`. Of two indices with the
/// same count, the index that got to the count first wins. `counts` is
/// scratch space.
fn most_frequent(indices: impl Iterator<Item = usize>, counts: &mut HashMap<usize, u32>) -> usize {
    counts.clear();
    let mut best = (0, 0);
    for index in indices {
        let count = counts.entry(index).or_insert(0);
        *count += 1;
        if *count > best.1 {
            best = (index, *count);
        }
    }
    best.0
}

/// Makes a document of `width` x `height` pixels from the merged color index
/// of each block. The palette has only the merged colors that a block uses:
/// the transparent color first, and then the others in order of frequency.
fn make_document(
    colors: &ColorMap,
    blocks: &[usize],
    width: u32,
    height: u32,
) -> Result<Document, ConvertError> {
    let mut used = vec![false; colors.colors.len()];
    for &index in blocks {
        used[index] = true;
    }
    let mut palette_order: Vec<usize> = (0..used.len()).filter(|&index| used[index]).collect();
    palette_order.sort_by_key(|&index| colors.colors[index] != TRANSPARENT);

    let mut keys = keys();
    let mut palette_index = vec![0; used.len()];
    let mut palette = Vec::with_capacity(palette_order.len());
    for (position, &index) in palette_order.iter().enumerate() {
        let color = colors.colors[index];
        let key = if color == TRANSPARENT {
            TRANSPARENT_KEY
        } else {
            keys.next().ok_or(ConvertError::TooManyColors)?
        };
        palette.push(PaletteEntry {
            key,
            color,
            label: None,
        });
        palette_index[index] = position;
    }
    let pixels = blocks.iter().map(|&index| palette_index[index]).collect();
    Ok(Document::from_parts(palette, width, height, pixels))
}

/// Returns the keys for opaque and partly transparent colors: letters,
/// digits, ASCII punctuation other than `.`, and then other Unicode letters
/// and digits.
fn keys() -> impl Iterator<Item = char> {
    let punctuation =
        ('!'..='~').filter(|key| key.is_ascii_punctuation() && *key != TRANSPARENT_KEY);
    let other = ('\u{C0}'..=char::MAX).filter(|key| key.is_alphanumeric());
    ('A'..='Z')
        .chain('a'..='z')
        .chain('0'..='9')
        .chain(punctuation)
        .chain(other)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{Image, parse};
    use crate::render::{render, write_png};

    const SOURCE: &str = "\
. - #00000000
K - #2B1410FF
o - #E8742AFF
w - #FFD9A0FF

..KKKK..
.KooooK.
KoowwooK
KoowwooK
.KooooK.
..KKKK..
";

    fn scale(size: u32) -> NonZeroU32 {
        NonZeroU32::new(size).unwrap()
    }

    fn png(image: &Image, size: u32) -> Vec<u8> {
        let mut encoded = Vec::new();
        write_png(&render(image, scale(size)).unwrap(), &mut encoded).unwrap();
        encoded
    }

    #[test]
    fn png_at_a_scale_converts_back_to_the_same_image() {
        let image = parse(SOURCE).unwrap();
        let document = convert(&png(&image, 3), scale(3)).unwrap();
        assert_eq!(document.to_image(), image);
        assert_eq!(document.palette()[0].key, TRANSPARENT_KEY);
    }

    #[test]
    fn transparent_pixels_with_different_rgb_become_one_entry() {
        let image = parse(". - #00000000\n, - #FF000000\nX - #FFFFFFFF\n\n.,X\n").unwrap();
        let document = convert(&png(&image, 1), scale(1)).unwrap();
        assert_eq!(document.to_pxl(), ". - #00000000\nA - #FFFFFFFF\n\n..A\n");
    }

    #[test]
    fn jpg_noise_merges_into_the_original_colors() {
        let image = parse(&SOURCE.replace(". - #00000000", ". - #FFFFFFFF")).unwrap();
        let canvas = render(&image, scale(4)).unwrap();
        let (pixels, _) = canvas.rgba.as_chunks::<4>();
        let rgb: Vec<u8> = pixels.iter().flat_map(|&[r, g, b, _]| [r, g, b]).collect();
        let mut encoded = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded, 80)
            .encode(
                &rgb,
                canvas.width,
                canvas.height,
                image::ExtendedColorType::Rgb8,
            )
            .unwrap();

        let document = convert(&encoded, scale(4)).unwrap();
        assert_eq!(document.palette().len(), 4);
        let converted = document.to_image();
        for (original, found) in image.pixels().iter().zip(converted.pixels()) {
            let difference = original.0.iter().zip(found.0).map(|(a, b)| a.abs_diff(b));
            assert!(u32::from(difference.max().unwrap()) <= JPG_TOLERANCE);
        }
    }

    #[test]
    fn rejects_a_scale_that_does_not_divide_the_size() {
        let image = parse(SOURCE).unwrap();
        assert!(matches!(
            convert(&png(&image, 3), scale(5)),
            Err(ConvertError::ScaleMismatch {
                width: 24,
                height: 18,
                ..
            })
        ));
    }
}
