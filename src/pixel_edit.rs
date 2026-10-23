//! Pixel edits: each edit sets a rectangle of pixels to one palette key. The
//! text form is `x,y=K`, where `x` and `y` are a 0-based coordinate or an
//! inclusive range `a-b`, and `K` is a palette key.

use std::fmt;
use std::str::FromStr;

use crate::document::Document;

/// A rectangle of pixels and the palette key that they get.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelEdit {
    x: Span,
    y: Span,
    key: char,
}

/// An inclusive range of coordinates on one axis. `first` <= `last`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span {
    first: u32,
    last: u32,
}

/// Edit text that is not in the form `x,y=K`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditSyntaxError {
    /// The text has no `=`, or no `,` before the `=`.
    Form,
    /// The text after the `=` is not exactly one char.
    KeyLength,
    /// A coordinate is not a number or a range of two numbers.
    InvalidCoordinate(String),
}

/// An edit that does not agree with the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditError {
    pub edit: PixelEdit,
    pub kind: EditErrorKind,
}

/// The cause of an [`EditError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditErrorKind {
    /// The x range is not inside the image.
    OutsideX { width: u32 },
    /// The y range is not inside the image.
    OutsideY { height: u32 },
    /// The key is not in the palette. `palette` lists the palette keys.
    UnknownKey { key: char, palette: String },
}

impl FromStr for PixelEdit {
    type Err = EditSyntaxError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Split at the first `=` and the first `,`, so that `=` and `,` are
        // valid keys.
        let (coordinates, key) = text.split_once('=').ok_or(EditSyntaxError::Form)?;
        let (x, y) = coordinates.split_once(',').ok_or(EditSyntaxError::Form)?;
        let mut key_chars = key.chars();
        let (Some(key), None) = (key_chars.next(), key_chars.next()) else {
            return Err(EditSyntaxError::KeyLength);
        };
        Ok(PixelEdit {
            x: x.parse()?,
            y: y.parse()?,
            key,
        })
    }
}

impl FromStr for Span {
    type Err = EditSyntaxError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let coordinate = |part: &str| {
            part.parse::<u32>()
                .map_err(|_| EditSyntaxError::InvalidCoordinate(text.into()))
        };
        let (start, end) = text.split_once('-').unwrap_or((text, text));
        let (start, end) = (coordinate(start)?, coordinate(end)?);
        // `7-3` gives the same pixels as `3-7`.
        Ok(Span {
            first: start.min(end),
            last: start.max(end),
        })
    }
}

impl fmt::Display for PixelEdit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}={}", self.x, self.y, self.key)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.first == self.last {
            write!(f, "{}", self.first)
        } else {
            write!(f, "{}-{}", self.first, self.last)
        }
    }
}

impl fmt::Display for EditSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditSyntaxError::Form => write!(
                f,
                "edit is not in the form `x,y=K`; for example, use `3,4=K` or `0-9,4=.`"
            ),
            EditSyntaxError::KeyLength => {
                write!(f, "key is not one char; give one palette key after `=`")
            }
            EditSyntaxError::InvalidCoordinate(text) => write!(
                f,
                "coordinate `{text}` is not valid; use a number such as `3` or a range such as `3-7`"
            ),
        }
    }
}

impl std::error::Error for EditSyntaxError {}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "edit `{}`: {}", self.edit, self.kind)
    }
}

impl fmt::Display for EditErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditErrorKind::OutsideX { width } => {
                write!(f, "x is outside the image; use x from 0 to {}", width - 1)
            }
            EditErrorKind::OutsideY { height } => {
                write!(f, "y is outside the image; use y from 0 to {}", height - 1)
            }
            EditErrorKind::UnknownKey { key, palette } => write!(
                f,
                "key `{key}` is not in the palette; use one of `{palette}`"
            ),
        }
    }
}

impl std::error::Error for EditError {}

impl Document {
    /// Applies `edits` in order, so that a later edit wins where edits
    /// overlap. Returns the number of pixels that changed.
    ///
    /// # Errors
    ///
    /// Returns an [`EditError`] for the first edit that is outside the image
    /// or uses a key that is not in the palette. Then the document does not
    /// change.
    pub fn apply_edits(&mut self, edits: &[PixelEdit]) -> Result<usize, EditError> {
        let indices = edits
            .iter()
            .map(|edit| self.edit_index(edit))
            .collect::<Result<Vec<_>, _>>()?;
        let before = self.clone();
        for (edit, index) in edits.iter().zip(indices) {
            for (x, y) in positions(edit.x, edit.y) {
                self.set_pixel(x, y, index);
            }
        }
        let all_x = Span {
            first: 0,
            last: self.width() - 1,
        };
        let all_y = Span {
            first: 0,
            last: self.height() - 1,
        };
        Ok(positions(all_x, all_y)
            .filter(|&(x, y)| self.pixel(x, y) != before.pixel(x, y))
            .count())
    }

    /// Checks that `edit` is inside the image, and returns the palette index
    /// of its key.
    fn edit_index(&self, edit: &PixelEdit) -> Result<usize, EditError> {
        let error = |kind| EditError { edit: *edit, kind };
        if edit.x.last >= self.width() {
            return Err(error(EditErrorKind::OutsideX {
                width: self.width(),
            }));
        }
        if edit.y.last >= self.height() {
            return Err(error(EditErrorKind::OutsideY {
                height: self.height(),
            }));
        }
        self.palette()
            .iter()
            .position(|entry| entry.key == edit.key)
            .ok_or_else(|| {
                let keys: Vec<String> = self
                    .palette()
                    .iter()
                    .map(|entry| entry.key.into())
                    .collect();
                error(EditErrorKind::UnknownKey {
                    key: edit.key,
                    palette: keys.join(" "),
                })
            })
    }
}

/// Returns the positions of the rectangle `x` by `y`, row by row.
fn positions(x: Span, y: Span) -> impl Iterator<Item = (u32, u32)> {
    (y.first..=y.last).flat_map(move |row| (x.first..=x.last).map(move |column| (column, row)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "O - #000000FF\nX - #FFFFFFFF\n= - #FF0000FF\n\nXXXX\nXXXX\nXXXX\n";

    fn edits(texts: &[&str]) -> Vec<PixelEdit> {
        texts.iter().map(|text| text.parse().unwrap()).collect()
    }

    fn apply(texts: &[&str]) -> (Document, Result<usize, EditError>) {
        let mut document = Document::parse(SOURCE).unwrap();
        let result = document.apply_edits(&edits(texts));
        (document, result)
    }

    #[test]
    fn parses_points_ranges_and_reversed_ranges() {
        assert_eq!(edits(&["3,4=K"])[0].to_string(), "3,4=K");
        assert_eq!(edits(&["0-9,4=."])[0].to_string(), "0-9,4=.");
        assert_eq!(edits(&["7-3,2-2=K"])[0].to_string(), "3-7,2=K");
    }

    #[test]
    fn accepts_equals_and_comma_as_keys() {
        assert_eq!(edits(&["1,2=="])[0].key, '=');
        assert_eq!(edits(&["1,2=,"])[0].key, ',');
    }

    #[test]
    fn rejects_text_that_is_not_an_edit() {
        let error = |text: &str| text.parse::<PixelEdit>().unwrap_err();
        assert_eq!(error("3,4"), EditSyntaxError::Form);
        assert_eq!(error("3=K"), EditSyntaxError::Form);
        assert_eq!(error("3,4="), EditSyntaxError::KeyLength);
        assert_eq!(error("3,4=KK"), EditSyntaxError::KeyLength);
        assert_eq!(
            error("a,4=K"),
            EditSyntaxError::InvalidCoordinate("a".into())
        );
        assert_eq!(
            error("3-,4=K"),
            EditSyntaxError::InvalidCoordinate("3-".into())
        );
    }

    #[test]
    fn sets_a_rectangle_and_later_edits_win() {
        let (document, result) = apply(&["1-2,0-1=O", "2,1=="]);
        assert_eq!(result, Ok(4));
        assert!(document.to_pxl().ends_with("XOOX\nXO=X\nXXXX\n"));
    }

    #[test]
    fn counts_only_pixels_that_end_with_a_new_key() {
        let (_, result) = apply(&["0,0=O", "0,0=X", "1,0=X"]);
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn rejects_an_edit_outside_the_image_and_changes_nothing() {
        let (document, result) = apply(&["0,0=O", "1-4,0=O"]);
        let error = result.unwrap_err();
        assert_eq!(error.kind, EditErrorKind::OutsideX { width: 4 });
        assert_eq!(
            error.to_string(),
            "edit `1-4,0=O`: x is outside the image; use x from 0 to 3"
        );
        assert_eq!(document, Document::parse(SOURCE).unwrap());
    }

    #[test]
    fn rejects_a_row_outside_the_image() {
        let (_, result) = apply(&["0,3=O"]);
        assert_eq!(
            result.unwrap_err().kind,
            EditErrorKind::OutsideY { height: 3 }
        );
    }

    #[test]
    fn rejects_a_key_that_is_not_in_the_palette() {
        let (_, result) = apply(&["0,0=Q"]);
        assert_eq!(
            result.unwrap_err().to_string(),
            "edit `0,0=Q`: key `Q` is not in the palette; use one of `O X =`"
        );
    }
}
