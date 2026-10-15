//! Parser for `.pxl` source text.

use std::collections::HashMap;
use std::fmt;
use std::iter::Peekable;

/// One RGBA color, 8 bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba(pub [u8; 4]);

/// A parsed pixel art image.
///
/// The width and the height are 1 or more, and the image has exactly
/// `width * height` pixels in row-major order. Only [`parse`] makes an
/// `Image`, so these rules are always true.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<Rgba>,
}

impl Image {
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns all pixels in row-major order.
    #[must_use]
    pub fn pixels(&self) -> &[Rgba] {
        &self.pixels
    }

    /// Returns the rows of pixels, from top to bottom.
    pub fn rows(&self) -> impl Iterator<Item = &[Rgba]> {
        self.pixels.chunks_exact(self.width as usize)
    }

    /// Returns the color at column `x` and row `y`.
    ///
    /// # Panics
    ///
    /// Panics if `x` or `y` is outside the image.
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> Rgba {
        assert!(
            x < self.width && y < self.height,
            "pixel is outside the image"
        );
        self.pixels[y as usize * self.width as usize + x as usize]
    }
}

/// A parse failure. `line` and `column` start at 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub kind: ParseErrorKind,
}

impl ParseError {
    fn new(line: usize, column: usize, kind: ParseErrorKind) -> Self {
        ParseError { line, column, kind }
    }
}

/// The cause of a [`ParseError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// The file has no palette entries.
    MissingPalette,
    /// The file has no pixel rows after the palette.
    MissingPixels,
    /// A palette line does not have the form `<char> - #RRGGBBAA`.
    InvalidPaletteEntry,
    /// A palette color is not `#` and 8 hex digits.
    InvalidColor(String),
    /// The palette key is a whitespace char.
    WhitespaceKey,
    /// Two palette entries use the same key.
    DuplicateKey(char),
    /// A pixel row uses a char that the palette does not define.
    UnknownKey(char),
    /// A pixel row has a different width than the first row.
    RowWidth { expected: usize, found: usize },
    /// A blank line occurs inside the pixel data.
    BlankLineInPixels,
    /// The image is too large for 32-bit dimensions.
    TooLarge,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::MissingPalette => {
                write!(f, "palette is missing; add lines such as `X - #FFFFFFFF`")
            }
            ParseErrorKind::MissingPixels => write!(
                f,
                "pixel data is missing; add a blank line and then the pixel rows"
            ),
            ParseErrorKind::InvalidPaletteEntry => write!(
                f,
                "palette entry is not valid; use the form `<char> - #RRGGBBAA`"
            ),
            ParseErrorKind::InvalidColor(color) => write!(
                f,
                "color `{color}` is not valid; use `#` and 8 hex digits (RRGGBBAA)"
            ),
            ParseErrorKind::WhitespaceKey => {
                write!(f, "palette key is whitespace; use a visible char")
            }
            ParseErrorKind::DuplicateKey(key) => {
                write!(f, "palette key `{key}` is defined two times; remove one")
            }
            ParseErrorKind::UnknownKey(key) => {
                write!(
                    f,
                    "char `{key}` is not in the palette; add it to the palette"
                )
            }
            ParseErrorKind::RowWidth { expected, found } => write!(
                f,
                "row has {found} pixels but the first row has {expected}; make all rows the same width"
            ),
            ParseErrorKind::BlankLineInPixels => {
                write!(f, "blank line is inside the pixel data; remove it")
            }
            ParseErrorKind::TooLarge => write!(f, "image is too large"),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}, column {}: {}",
            self.line, self.column, self.kind
        )
    }
}

impl std::error::Error for ParseError {}

/// Parses `.pxl` source text into an [`Image`].
///
/// # Errors
///
/// Returns a [`ParseError`] with the line, the column, and the cause if the
/// source is not a valid `.pxl` file.
pub fn parse(source: &str) -> Result<Image, ParseError> {
    let mut lines = source
        .lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line.trim_end()))
        .skip_while(|(_, line)| line.is_empty())
        .peekable();
    let palette = parse_palette(&mut lines)?;
    parse_pixels(lines.skip_while(|(_, line)| line.is_empty()), &palette)
}

/// A source line, trimmed at the end, with its 1-based line number.
type Line<'a> = (usize, &'a str);

type Palette = HashMap<char, Rgba>;

/// Parses the palette entries up to the first blank line.
fn parse_palette<'a>(
    lines: &mut Peekable<impl Iterator<Item = Line<'a>>>,
) -> Result<Palette, ParseError> {
    let mut palette = Palette::new();
    while let Some((number, line)) = lines.next_if(|(_, line)| !line.is_empty()) {
        let (key, color) = parse_palette_entry(number, line)?;
        if palette.insert(key, color).is_some() {
            let kind = ParseErrorKind::DuplicateKey(key);
            return Err(ParseError::new(number, 1, kind));
        }
    }
    if palette.is_empty() {
        let line = lines.peek().map_or(1, |&(number, _)| number);
        return Err(ParseError::new(line, 1, ParseErrorKind::MissingPalette));
    }
    Ok(palette)
}

/// Parses the pixel rows into an [`Image`]. `lines` starts at the first row.
fn parse_pixels<'a>(
    lines: impl Iterator<Item = Line<'a>>,
    palette: &Palette,
) -> Result<Image, ParseError> {
    let mut pixels = Vec::new();
    let mut width = 0;
    let mut height = 0;
    let mut last_line = 1;
    let mut blank_line = None;

    for (number, line) in lines {
        last_line = number;
        if line.is_empty() {
            blank_line.get_or_insert(number);
            continue;
        }
        if let Some(blank) = blank_line {
            return Err(ParseError::new(blank, 1, ParseErrorKind::BlankLineInPixels));
        }
        let row_width = parse_row(number, line, palette, &mut pixels)?;
        if height == 0 {
            width = row_width;
        } else if row_width != width {
            let kind = ParseErrorKind::RowWidth {
                expected: width,
                found: row_width,
            };
            return Err(ParseError::new(number, 1, kind));
        }
        height += 1;
    }

    if height == 0 {
        return Err(ParseError::new(last_line, 1, ParseErrorKind::MissingPixels));
    }
    let too_large = |_| ParseError::new(last_line, 1, ParseErrorKind::TooLarge);
    Ok(Image {
        width: u32::try_from(width).map_err(too_large)?,
        height: u32::try_from(height).map_err(too_large)?,
        pixels,
    })
}

/// Appends the colors of one pixel row to `pixels`, and returns the row width.
/// `number` is the line number for errors.
fn parse_row(
    number: usize,
    line: &str,
    palette: &Palette,
    pixels: &mut Vec<Rgba>,
) -> Result<usize, ParseError> {
    let row_start = pixels.len();
    for (column, key) in line.chars().enumerate() {
        let color = palette
            .get(&key)
            .copied()
            .ok_or_else(|| ParseError::new(number, column + 1, ParseErrorKind::UnknownKey(key)))?;
        pixels.push(color);
    }
    Ok(pixels.len() - row_start)
}

/// Parses one palette line in the form `<char> - #RRGGBBAA`. `number` is the
/// line number for errors. The line must not be empty.
fn parse_palette_entry(number: usize, line: &str) -> Result<(char, Rgba), ParseError> {
    let mut chars = line.chars();
    let key = chars.next().expect("palette line is not empty");
    if key.is_whitespace() {
        return Err(ParseError::new(number, 1, ParseErrorKind::WhitespaceKey));
    }

    let invalid_entry = || ParseError::new(number, 1, ParseErrorKind::InvalidPaletteEntry);
    let color = chars
        .as_str()
        .trim_start()
        .strip_prefix('-')
        .ok_or_else(invalid_entry)?
        .trim_start();
    if color.is_empty() {
        return Err(invalid_entry());
    }

    let column = line.chars().count() - color.chars().count() + 1;
    let invalid_color =
        || ParseError::new(number, column, ParseErrorKind::InvalidColor(color.into()));
    let hex = color.strip_prefix('#').ok_or_else(invalid_color)?;
    // Check the digits first: `from_str_radix` also accepts a leading `+`.
    if hex.len() != 8 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid_color());
    }
    let value = u32::from_str_radix(hex, 16).expect("hex is 8 hex digits");
    Ok((key, Rgba(value.to_be_bytes())))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: Rgba = Rgba([0x00, 0x00, 0x00, 0xFF]);
    const WHITE: Rgba = Rgba([0xFF, 0xFF, 0xFF, 0xFF]);

    fn kind(source: &str) -> ParseErrorKind {
        parse(source).unwrap_err().kind
    }

    #[test]
    fn parses_palette_and_pixels() {
        let image = parse("O - #000000FF\nX - #ffffffff\n\nXXX\nXOX\n").unwrap();
        assert_eq!((image.width(), image.height()), (3, 2));
        assert_eq!(image.pixels(), [WHITE, WHITE, WHITE, WHITE, BLACK, WHITE]);
        assert_eq!(image.pixel(1, 1), BLACK);
    }

    #[test]
    fn accepts_crlf_extra_blank_lines_and_flexible_spacing() {
        let image = parse("\r\n\r\nA-#11223344\r\n\r\n\r\nA\r\n\r\n").unwrap();
        assert_eq!(image.pixels(), [Rgba([0x11, 0x22, 0x33, 0x44])]);
    }

    #[test]
    fn allows_dash_and_hash_as_keys() {
        let image = parse("- - #000000FF\n# - #FFFFFFFF\n\n-#\n").unwrap();
        assert_eq!(image.pixels(), [BLACK, WHITE]);
    }

    #[test]
    fn reports_unknown_key_with_position() {
        let err = parse("O - #000000FF\n\nOO\nO0\n").unwrap_err();
        assert_eq!((err.line, err.column), (4, 2));
        assert_eq!(err.kind, ParseErrorKind::UnknownKey('0'));
    }

    #[test]
    fn rejects_ragged_rows() {
        assert_eq!(
            kind("X - #000000FF\n\nXX\nX\n"),
            ParseErrorKind::RowWidth {
                expected: 2,
                found: 1
            }
        );
    }

    #[test]
    fn rejects_duplicate_keys() {
        assert_eq!(
            kind("X - #000000FF\nX - #FFFFFFFF\n\nX\n"),
            ParseErrorKind::DuplicateKey('X')
        );
    }

    #[test]
    fn rejects_colors_that_are_not_hash_and_8_hex_digits() {
        for color in ["#FFFFFF", "FFFFFFFF", "#GGFFFFFF", "#+FFFFFFF"] {
            let source = format!("X - {color}\n\nX\n");
            assert_eq!(
                kind(&source),
                ParseErrorKind::InvalidColor(color.into()),
                "{color}"
            );
        }
    }

    #[test]
    fn rejects_entry_without_dash() {
        assert_eq!(
            kind("X #FFFFFFFF\n\nX\n"),
            ParseErrorKind::InvalidPaletteEntry
        );
    }

    #[test]
    fn rejects_missing_sections() {
        assert_eq!(kind(""), ParseErrorKind::MissingPalette);
        assert_eq!(kind("X - #FFFFFFFF\n"), ParseErrorKind::MissingPixels);
    }

    #[test]
    fn rejects_blank_line_inside_pixels() {
        assert_eq!(
            kind("X - #FFFFFFFF\n\nX\n\nX\n"),
            ParseErrorKind::BlankLineInPixels
        );
    }
}
