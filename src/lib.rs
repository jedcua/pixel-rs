//! Parser and renderer for the `.pxl` pixel art text format.
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

mod grid;
mod parse;
mod render;

pub use grid::render_grid;
pub use parse::{Image, ParseError, ParseErrorKind, Rgba, parse};
pub use render::{Canvas, RenderError, render, write_png};
