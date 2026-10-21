//! Renderer that turns an [`Image`] into scaled RGBA pixels and PNG files.

use std::fmt;
use std::io::Write;
use std::num::NonZeroU32;

use crate::parse::Image;

/// A render failure.
#[derive(Debug)]
pub enum RenderError {
    /// The scaled image does not fit in 32-bit dimensions or in memory.
    TooLarge { scale: NonZeroU32 },
    /// The PNG encoder failed.
    Encode(png::EncodingError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::TooLarge { scale } => {
                write!(
                    f,
                    "image is too large at scale {scale}; use a smaller scale"
                )
            }
            RenderError::Encode(err) => write!(f, "PNG encoder failed: {err}"),
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RenderError::Encode(err) => Some(err),
            RenderError::TooLarge { .. } => None,
        }
    }
}

impl From<png::EncodingError> for RenderError {
    fn from(err: png::EncodingError) -> Self {
        RenderError::Encode(err)
    }
}

/// A rendered RGBA8 buffer. The data is in row-major order, 4 bytes per pixel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Canvas {
    /// Makes a canvas of `width` x `height` pixels, all set to `color`.
    /// Returns `None` if the buffer size does not fit in `usize`.
    pub(crate) fn filled(width: u32, height: u32, color: [u8; 4]) -> Option<Self> {
        let pixels = (width as usize).checked_mul(height as usize)?;
        // `repeat` panics if the size in bytes overflows, so check it first.
        pixels.checked_mul(color.len())?;
        Some(Canvas {
            width,
            height,
            rgba: color.repeat(pixels),
        })
    }

    /// Returns the color of the pixel at (`x`, `y`).
    ///
    /// # Panics
    ///
    /// Panics if `x` or `y` is outside the canvas.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> [u8; 4] {
        let index = self.index(x, y);
        self.rgba[index..index + 4]
            .try_into()
            .expect("slice has 4 bytes")
    }

    /// Sets the pixel at (`x`, `y`) to `color`.
    pub(crate) fn set(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let index = self.index(x, y);
        self.rgba[index..index + 4].copy_from_slice(&color);
    }

    fn index(&self, x: u32, y: u32) -> usize {
        assert!(
            x < self.width && y < self.height,
            "pixel is outside the canvas"
        );
        (y as usize * self.width as usize + x as usize) * 4
    }
}

/// Renders `image` so that each source pixel becomes a `scale` x `scale` block.
///
/// # Errors
///
/// Returns [`RenderError::TooLarge`] if the scaled image does not fit in
/// 32-bit dimensions or in memory.
pub fn render(image: &Image, scale: NonZeroU32) -> Result<Canvas, RenderError> {
    let too_large = || RenderError::TooLarge { scale };
    let factor = scale.get();
    let width = image.width().checked_mul(factor).ok_or_else(too_large)?;
    let height = image.height().checked_mul(factor).ok_or_else(too_large)?;
    let row_bytes = (width as usize).checked_mul(4).ok_or_else(too_large)?;
    let total = row_bytes
        .checked_mul(height as usize)
        .ok_or_else(too_large)?;

    let mut data = Vec::with_capacity(total);
    for source_row in image.rows() {
        let row_start = data.len();
        for color in source_row {
            for _ in 0..factor {
                data.extend_from_slice(&color.0);
            }
        }
        for _ in 1..factor {
            data.extend_from_within(row_start..row_start + row_bytes);
        }
    }

    Ok(Canvas {
        width,
        height,
        rgba: data,
    })
}

/// Writes `canvas` as a PNG to `writer`.
///
/// # Errors
///
/// Returns [`RenderError::Encode`] if the PNG encoder or `writer` fails.
pub fn write_png<W: Write>(canvas: &Canvas, writer: W) -> Result<(), RenderError> {
    let mut encoder = png::Encoder::new(writer, canvas.width, canvas.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header()?;
    png_writer.write_image_data(&canvas.rgba)?;
    png_writer.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    #[test]
    fn scale_repeats_each_pixel_as_a_block() {
        let image = parse("A - #FF000080\nB - #0000FFFF\n\nAB\nBA\n").unwrap();
        let canvas = render(&image, NonZeroU32::new(2).unwrap()).unwrap();
        assert_eq!((canvas.width, canvas.height), (4, 4));

        let a = [0xFF, 0x00, 0x00, 0x80];
        let b = [0x00, 0x00, 0xFF, 0xFF];
        let expected: Vec<u8> = [[a, a, b, b], [a, a, b, b], [b, b, a, a], [b, b, a, a]]
            .concat()
            .concat();
        assert_eq!(canvas.rgba, expected);
    }

    #[test]
    fn rejects_overflowing_scale() {
        let image = parse("A - #000000FF\n\nAA\n").unwrap();
        assert!(matches!(
            render(&image, NonZeroU32::MAX),
            Err(RenderError::TooLarge { .. })
        ));
    }
}
