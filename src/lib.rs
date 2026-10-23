//! Parser, renderer, pixel edits, and image converter for the `.pxl` pixel
//! art text format.
//!
//! A `.pxl` file has two sections. A blank line separates them.
//!
//! 1. The palette: one entry per line, in the form `<char> - #RRGGBBAA`.
//! 2. The pixel data: one row per line. Each char is one pixel.
//!
//! ```text
//! O - #000000FF
//! X - #FFFFFFFF
//!
//! XXXXX
//! XOOOX
//! XXXXX
//! ```

mod ansi;
mod convert;
mod document;
mod grid;
mod parse;
mod pixel_edit;
mod render;

pub use ansi::render_ansi;
pub use convert::{ConvertError, convert};
pub use document::{Document, PaletteEntry, PaletteError};
pub use grid::render_grid;
pub use parse::{CHECKER, Image, ParseError, ParseErrorKind, Rgba, parse};
pub use pixel_edit::{EditError, EditErrorKind, EditSyntaxError, PixelEdit};
pub use render::{Canvas, RenderError, render, write_png};
