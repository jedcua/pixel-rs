//! Terminal renderer: text with 24-bit ANSI colors and half-block chars.

use std::fmt::Write as _;

use crate::parse::{Image, Rgba};

/// A pixel with an alpha below this value shows the terminal background.
const OPAQUE_ALPHA: u8 = 128;

const UPPER_HALF: char = '▀';
const LOWER_HALF: char = '▄';
const RESET: &str = "\x1b[0m";

/// The colors of a char cell. `None` is the terminal default.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Style {
    foreground: Option<[u8; 3]>,
    background: Option<[u8; 3]>,
}

impl Style {
    const DEFAULT: Style = Style::new(None, None);

    const fn new(foreground: Option<[u8; 3]>, background: Option<[u8; 3]>) -> Self {
        Style {
            foreground,
            background,
        }
    }
}

/// Renders `image` as text for a terminal with 24-bit color. Each char cell
/// shows two pixels on top of each other: the upper pixel with the
/// foreground color of `▀`, and the lower pixel with the background color.
/// Thus the text has `width` chars and `height / 2` lines, rounded up.
///
/// A pixel with an alpha below 128 shows the terminal background. Other
/// pixels show their RGB color and ignore the alpha. Each line ends with the
/// default colors, so that the text after it is not colored.
#[must_use]
pub fn render_ansi(image: &Image) -> String {
    let mut text = String::new();
    let rows: Vec<&[Rgba]> = image.rows().collect();
    for pair in rows.chunks(2) {
        let lower_row = pair.get(1).copied();
        let mut style = Style::DEFAULT;
        for (x, &upper) in pair[0].iter().enumerate() {
            let (glyph, cell_style) = cell(upper, lower_row.map(|row| row[x]));
            if cell_style != style {
                push_style(&mut text, style, cell_style);
                style = cell_style;
            }
            text.push(glyph);
        }
        if style != Style::DEFAULT {
            text.push_str(RESET);
        }
        text.push('\n');
    }
    text
}

/// Returns the glyph and the style of the cell that shows `upper` above
/// `lower`. The last line of an image with an odd height has no `lower`.
fn cell(upper: Rgba, lower: Option<Rgba>) -> (char, Style) {
    match (visible(upper), lower.and_then(visible)) {
        (Some(top), bottom) => (UPPER_HALF, Style::new(Some(top), bottom)),
        (None, Some(bottom)) => (LOWER_HALF, Style::new(Some(bottom), None)),
        (None, None) => (' ', Style::DEFAULT),
    }
}

/// Returns the RGB color of `color`, or `None` if it is transparent.
fn visible(Rgba([red, green, blue, alpha]): Rgba) -> Option<[u8; 3]> {
    (alpha >= OPAQUE_ALPHA).then_some([red, green, blue])
}

/// Appends the escape codes that change the style from `old` to `new`.
fn push_style(text: &mut String, old: Style, new: Style) {
    if old != Style::DEFAULT {
        text.push_str(RESET);
    }
    // A `String` cannot fail as a `fmt::Write` target.
    if let Some([red, green, blue]) = new.foreground {
        let _ = write!(text, "\x1b[38;2;{red};{green};{blue}m");
    }
    if let Some([red, green, blue]) = new.background {
        let _ = write!(text, "\x1b[48;2;{red};{green};{blue}m");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    fn ansi(source: &str) -> String {
        render_ansi(&parse(source).unwrap())
    }

    #[test]
    fn puts_two_pixel_rows_in_one_line_of_half_blocks() {
        let text = ansi("R - #FF0000FF\nB - #0000FFFF\n\nRB\nBB\n");
        assert_eq!(
            text,
            "\x1b[38;2;255;0;0m\x1b[48;2;0;0;255m▀\x1b[0m\x1b[38;2;0;0;255m\x1b[48;2;0;0;255m▀\x1b[0m\n"
        );
    }

    #[test]
    fn writes_a_style_only_when_it_changes() {
        let text = ansi("R - #FF0000FF\n\nRRR\n");
        assert_eq!(text, "\x1b[38;2;255;0;0m▀▀▀\x1b[0m\n");
    }

    #[test]
    fn transparent_pixels_show_the_terminal_background() {
        let text = ansi(". - #FFFFFF7F\nR - #FF000080\n\n.R.\nR..\n");
        assert_eq!(text, "\x1b[38;2;255;0;0m▄▀\x1b[0m \n");
    }

    #[test]
    fn last_line_of_an_odd_height_has_only_upper_pixels() {
        let text = ansi(". - #00000000\nR - #FF0000FF\n\n..\n..\nR.\n");
        assert_eq!(text, "  \n\x1b[38;2;255;0;0m▀\x1b[0m \n");
    }
}
