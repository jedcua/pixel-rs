//! An editable `.pxl` document: the palette in file order and the palette
//! index of each pixel.

use std::fmt;
use std::fmt::Write as _;

use crate::parse::{Image, Rgba};

/// One palette line: a key char and its color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteEntry {
    pub key: char,
    pub color: Rgba,
}

/// A palette change that the document does not accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteError {
    /// The key is a whitespace char.
    WhitespaceKey,
    /// Another entry uses the key.
    DuplicateKey(char),
    /// The palette has only one entry, and a document needs one or more.
    LastEntry,
    /// The replacement for a removed entry is the removed entry.
    SelfReplacement,
}

/// Each message is a full sentence that the editor can show to the user.
impl fmt::Display for PaletteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaletteError::WhitespaceKey => write!(f, "Key is whitespace. Use a visible char."),
            PaletteError::DuplicateKey(key) => {
                write!(f, "Key `{key}` is in use. Use a different key.")
            }
            PaletteError::LastEntry => {
                write!(f, "Palette has only one color. Add a color first.")
            }
            PaletteError::SelfReplacement => {
                write!(f, "Replacement is the removed color. Use a different key.")
            }
        }
    }
}

impl std::error::Error for PaletteError {}

/// A `.pxl` image that keeps the palette keys, so that it can change the
/// palette and write the `.pxl` text again.
///
/// The palette has 1 or more entries with different keys. The width and the
/// height are 1 or more. Each pixel is an index into the palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    palette: Vec<PaletteEntry>,
    width: u32,
    height: u32,
    pixels: Vec<usize>,
}

impl Document {
    /// Makes a document from parts that the parser checked.
    pub(crate) fn from_parts(
        palette: Vec<PaletteEntry>,
        width: u32,
        height: u32,
        pixels: Vec<usize>,
    ) -> Self {
        Document {
            palette,
            width,
            height,
            pixels,
        }
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the palette in file order.
    #[must_use]
    pub fn palette(&self) -> &[PaletteEntry] {
        &self.palette
    }

    /// Returns the palette index of the pixel at column `x` and row `y`.
    ///
    /// # Panics
    ///
    /// Panics if `x` or `y` is outside the image.
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> usize {
        self.pixels[self.offset(x, y)]
    }

    /// Sets the pixel at column `x` and row `y` to the palette entry `index`.
    /// Returns true if the pixel changed.
    ///
    /// # Panics
    ///
    /// Panics if `x` or `y` is outside the image, or if `index` is not a
    /// palette index.
    pub fn set_pixel(&mut self, x: u32, y: u32, index: usize) -> bool {
        assert!(index < self.palette.len(), "index is not in the palette");
        let offset = self.offset(x, y);
        let changed = self.pixels[offset] != index;
        self.pixels[offset] = index;
        changed
    }

    /// Returns the number of pixels that use the palette entry `index`.
    #[must_use]
    pub fn usage(&self, index: usize) -> usize {
        self.pixels.iter().filter(|&&pixel| pixel == index).count()
    }

    /// Adds `entry` at the end of the palette and returns its index.
    ///
    /// # Errors
    ///
    /// Returns a [`PaletteError`] if the key is whitespace or in use.
    pub fn add_entry(&mut self, entry: PaletteEntry) -> Result<usize, PaletteError> {
        self.check_key(entry.key, None)?;
        self.palette.push(entry);
        Ok(self.palette.len() - 1)
    }

    /// Replaces the key and the color of the palette entry `index`. All the
    /// pixels that use the entry get the new color.
    ///
    /// # Errors
    ///
    /// Returns a [`PaletteError`] if the key is whitespace or another entry
    /// uses it.
    ///
    /// # Panics
    ///
    /// Panics if `index` is not a palette index.
    pub fn set_entry(&mut self, index: usize, entry: PaletteEntry) -> Result<(), PaletteError> {
        assert!(index < self.palette.len(), "index is not in the palette");
        self.check_key(entry.key, Some(index))?;
        self.palette[index] = entry;
        Ok(())
    }

    /// Removes the palette entry `index`. The pixels that use it change to
    /// the entry `replacement`. The indices after `index` move down by 1.
    /// Returns the new index of `replacement`.
    ///
    /// # Errors
    ///
    /// Returns [`PaletteError::LastEntry`] if the palette has one entry, or
    /// [`PaletteError::SelfReplacement`] if `replacement` is `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index` or `replacement` is not a palette index.
    pub fn remove_entry(
        &mut self,
        index: usize,
        replacement: usize,
    ) -> Result<usize, PaletteError> {
        let len = self.palette.len();
        assert!(
            index < len && replacement < len,
            "index is not in the palette"
        );
        if len == 1 {
            return Err(PaletteError::LastEntry);
        }
        if index == replacement {
            return Err(PaletteError::SelfReplacement);
        }
        let shift = |pixel: usize| if pixel > index { pixel - 1 } else { pixel };
        let target = shift(replacement);
        for pixel in &mut self.pixels {
            *pixel = if *pixel == index {
                target
            } else {
                shift(*pixel)
            };
        }
        self.palette.remove(index);
        Ok(target)
    }

    /// Returns the image with the colors of the palette.
    #[must_use]
    pub fn to_image(&self) -> Image {
        let pixels = self
            .pixels
            .iter()
            .map(|&index| self.palette[index].color)
            .collect();
        Image::new(self.width, self.height, pixels)
    }

    /// Returns the `.pxl` source text: one `<key> - #RRGGBBAA` line per
    /// palette entry, a blank line, and then the pixel rows.
    #[must_use]
    pub fn to_pxl(&self) -> String {
        let mut text = String::new();
        for entry in &self.palette {
            // A `String` cannot fail as a `fmt::Write` target.
            let _ = writeln!(text, "{} - {}", entry.key, entry.color);
        }
        text.push('\n');
        for row in self.pixels.chunks_exact(self.width as usize) {
            text.extend(row.iter().map(|&index| self.palette[index].key));
            text.push('\n');
        }
        text
    }

    /// Checks that `key` is visible and that no entry other than `except`
    /// uses it.
    fn check_key(&self, key: char, except: Option<usize>) -> Result<(), PaletteError> {
        if key.is_whitespace() {
            return Err(PaletteError::WhitespaceKey);
        }
        let in_use = self
            .palette
            .iter()
            .enumerate()
            .any(|(index, entry)| entry.key == key && Some(index) != except);
        if in_use {
            return Err(PaletteError::DuplicateKey(key));
        }
        Ok(())
    }

    fn offset(&self, x: u32, y: u32) -> usize {
        assert!(
            x < self.width && y < self.height,
            "pixel is outside the image"
        );
        y as usize * self.width as usize + x as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "O - #000000FF\nX - #FFFFFFFF\nr - #FF0000FF\n\nXXX\nXOr\n";

    fn document() -> Document {
        Document::parse(SOURCE).unwrap()
    }

    fn entry(key: char) -> PaletteEntry {
        PaletteEntry {
            key,
            color: Rgba([1, 2, 3, 4]),
        }
    }

    #[test]
    fn writes_the_same_document_that_it_parses() {
        let doc = Document::parse("a-#11aa22ff\r\n\r\n\r\naa\r\n").unwrap();
        assert_eq!(doc.to_pxl(), "a - #11AA22FF\n\naa\n");
        assert_eq!(Document::parse(&doc.to_pxl()).unwrap(), doc);
    }

    #[test]
    fn set_pixel_reports_a_change_only_for_a_new_index() {
        let mut doc = document();
        assert!(doc.set_pixel(2, 0, 2));
        assert!(!doc.set_pixel(2, 0, 2));
        assert!(doc.to_pxl().ends_with("XXr\nXOr\n"));
    }

    #[test]
    fn rejects_whitespace_and_duplicate_keys() {
        let mut doc = document();
        assert_eq!(doc.add_entry(entry(' ')), Err(PaletteError::WhitespaceKey));
        assert_eq!(
            doc.add_entry(entry('X')),
            Err(PaletteError::DuplicateKey('X'))
        );
        assert_eq!(
            doc.set_entry(0, entry('X')),
            Err(PaletteError::DuplicateKey('X'))
        );
    }

    #[test]
    fn set_entry_can_keep_the_key_and_renames_all_pixels() {
        let mut doc = document();
        doc.set_entry(1, entry('X')).unwrap();
        doc.set_entry(1, entry('Y')).unwrap();
        assert!(doc.to_pxl().ends_with("YYY\nYOr\n"));
        assert_eq!(doc.to_image().pixel(0, 0), Rgba([1, 2, 3, 4]));
    }

    #[test]
    fn remove_entry_replaces_its_pixels_and_shifts_later_indices() {
        let mut doc = document();
        assert_eq!(doc.remove_entry(0, 2), Ok(1));
        assert_eq!(doc.to_pxl(), "X - #FFFFFFFF\nr - #FF0000FF\n\nXXX\nXrr\n");
        assert_eq!((doc.pixel(0, 0), doc.pixel(2, 1)), (0, 1));
    }

    #[test]
    fn remove_entry_keeps_one_entry_and_rejects_self_replacement() {
        let mut doc = Document::parse("X - #FFFFFFFF\n\nX\n").unwrap();
        assert_eq!(doc.remove_entry(0, 0), Err(PaletteError::LastEntry));
        let mut doc = document();
        assert_eq!(doc.remove_entry(1, 1), Err(PaletteError::SelfReplacement));
    }
}
