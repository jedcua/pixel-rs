//! Grid view: an inspection image with grid lines, coordinate labels, and a
//! checkerboard behind transparent pixels.

use std::num::NonZeroU32;

use crate::parse::{CHECKER, Image, Rgba};
use crate::render::{Canvas, RenderError};

/// Grid lines at multiples of this cell count are darker.
const MAJOR_STEP: u32 = 10;
/// Size in output pixels of one square of the checkerboard.
const CHECKER_SIZE: u32 = 4;
/// Space in output pixels between a label and the grid or the image edge.
const LABEL_PAD: u32 = 4;

const MARGIN_COLOR: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
const LABEL_COLOR: [u8; 4] = [0x20, 0x20, 0x20, 0xFF];
const MINOR_LINE_COLOR: [u8; 4] = [0xC8, 0xC8, 0xC8, 0xFF];
const MAJOR_LINE_COLOR: [u8; 4] = [0x40, 0x40, 0x40, 0xFF];

/// Size in output pixels of one dot in a digit glyph.
const GLYPH_DOT: u32 = 2;
const GLYPH_COLUMNS: u32 = 3;
const GLYPH_ROWS: u32 = 5;
const GLYPH_WIDTH: u32 = GLYPH_COLUMNS * GLYPH_DOT;
const GLYPH_HEIGHT: u32 = GLYPH_ROWS * GLYPH_DOT;
const GLYPH_GAP: u32 = GLYPH_DOT;

/// 3 x 5 bitmaps for the digits 0-9. Each row uses the low 3 bits, left bit first.
const DIGITS: [[u8; GLYPH_ROWS as usize]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b111, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b001, 0b001, 0b001],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
];

/// Renders `image` as a grid for inspection. Each source pixel becomes a
/// `scale` x `scale` cell, with a 1-pixel line between cells. The lines at
/// multiples of 10 are darker. Labels give the 0-based x coordinate at the top
/// and the y coordinate at the left. Transparent pixels show a checkerboard.
///
/// # Errors
///
/// Returns [`RenderError::TooLarge`] if the grid image does not fit in
/// 32-bit dimensions or in memory.
pub fn render_grid(image: &Image, scale: NonZeroU32) -> Result<Canvas, RenderError> {
    let layout = GridLayout::new(image, scale)?;
    let mut canvas = Canvas::filled(layout.width, layout.height, MARGIN_COLOR)
        .ok_or(RenderError::TooLarge { scale })?;
    draw_lines(&mut canvas, image, &layout);
    draw_cells(&mut canvas, image, &layout);
    draw_labels(&mut canvas, image, &layout);
    Ok(canvas)
}

/// Positions of the parts of a grid canvas.
struct GridLayout {
    /// Size in output pixels of one cell.
    cell: u32,
    /// Distance in output pixels from one cell to the next: `cell` plus one grid line.
    pitch: u32,
    /// Width of the left margin, which holds the row labels.
    left: u32,
    /// Height of the top margin, which holds the column labels.
    top: u32,
    /// Cell count between two labels.
    label_step: u32,
    width: u32,
    height: u32,
}

impl GridLayout {
    fn new(image: &Image, scale: NonZeroU32) -> Result<Self, RenderError> {
        let too_large = || RenderError::TooLarge { scale };
        let cell = scale.get();
        let pitch = cell.checked_add(1).ok_or_else(too_large)?;
        let largest_label = image.width().max(image.height()) - 1;
        let label_width = text_width(largest_label);

        let left = label_width + 2 * LABEL_PAD;
        let top = GLYPH_HEIGHT + 2 * LABEL_PAD;
        // Each side of the grid has one line more than it has cells.
        let span = |cells: u32, margin: u32| {
            cells
                .checked_mul(pitch)
                .and_then(|size| size.checked_add(1 + margin))
                .ok_or_else(too_large)
        };
        Ok(GridLayout {
            cell,
            pitch,
            left,
            top,
            label_step: label_step(label_width.max(GLYPH_HEIGHT) + LABEL_PAD, pitch),
            width: span(image.width(), left)?,
            height: span(image.height(), top)?,
        })
    }

    /// Returns the output position of the line before cell `index`, in the
    /// direction of `origin` (`left` for columns, `top` for rows).
    fn line(&self, origin: u32, index: u32) -> u32 {
        origin + index * self.pitch
    }

    /// Returns the output position of the top-left pixel of cell (`x`, `y`).
    fn cell_origin(&self, x: u32, y: u32) -> (u32, u32) {
        (self.line(self.left, x) + 1, self.line(self.top, y) + 1)
    }
}

/// Returns the smallest step of 10, 20, 50, 100, 200, ... cells that puts at
/// least `room` output pixels between two labels, so that labels do not touch.
fn label_step(room: u32, pitch: u32) -> u32 {
    let mut base = MAJOR_STEP;
    loop {
        for factor in [1, 2, 5] {
            let step = base.saturating_mul(factor);
            if step.saturating_mul(pitch) >= room {
                return step;
            }
        }
        base = base.saturating_mul(10);
    }
}

/// Draws the grid lines. The outer border and the lines at multiples of
/// [`MAJOR_STEP`] are major lines.
fn draw_lines(canvas: &mut Canvas, image: &Image, layout: &GridLayout) {
    let is_major = |index: u32, count: u32| index.is_multiple_of(MAJOR_STEP) || index == count;
    let right = layout.line(layout.left, image.width());
    let bottom = layout.line(layout.top, image.height());

    // Draw the minor lines first so that the major lines stay unbroken where they cross.
    for (major, color) in [(false, MINOR_LINE_COLOR), (true, MAJOR_LINE_COLOR)] {
        for row in (0..=image.height()).filter(|&row| is_major(row, image.height()) == major) {
            let y = layout.line(layout.top, row);
            for x in layout.left..=right {
                canvas.set(x, y, color);
            }
        }
        for column in (0..=image.width()).filter(|&column| is_major(column, image.width()) == major)
        {
            let x = layout.line(layout.left, column);
            for y in layout.top..=bottom {
                canvas.set(x, y, color);
            }
        }
    }
}

/// Fills each cell with its pixel color, blended over a checkerboard.
fn draw_cells(canvas: &mut Canvas, image: &Image, layout: &GridLayout) {
    for y in 0..image.height() {
        for x in 0..image.width() {
            let origin = layout.cell_origin(x, y);
            fill_cell(canvas, origin, layout.cell, image.pixel(x, y));
        }
    }
}

/// Fills the `size` x `size` cell with its top-left corner at `origin` with
/// `color`, blended over a checkerboard.
fn fill_cell(canvas: &mut Canvas, origin: (u32, u32), size: u32, color: Rgba) {
    let (origin_x, origin_y) = origin;
    for dy in 0..size {
        for dx in 0..size {
            let checker = (dx / CHECKER_SIZE + dy / CHECKER_SIZE) % 2;
            let back = CHECKER[checker as usize];
            canvas.set(origin_x + dx, origin_y + dy, blend(color, back));
        }
    }
}

/// Blends `color` over the opaque color `back` and returns an opaque color.
fn blend(color: Rgba, back: [u8; 3]) -> [u8; 4] {
    let [red, green, blue] = color.over(back);
    [red, green, blue, 0xFF]
}

/// Draws the x labels above the grid and the y labels at its left.
fn draw_labels(canvas: &mut Canvas, image: &Image, layout: &GridLayout) {
    let step = layout.label_step as usize;
    for column in (0..image.width()).step_by(step) {
        let (origin_x, _) = layout.cell_origin(column, 0);
        let width = text_width(column);
        // Center the label on the cell, but keep it inside the canvas.
        let x = (origin_x + layout.cell / 2)
            .saturating_sub(width / 2)
            .min(layout.width - width);
        draw_number(canvas, column, x, LABEL_PAD);
    }
    for row in (0..image.height()).step_by(step) {
        let (_, origin_y) = layout.cell_origin(0, row);
        let y = (origin_y + layout.cell / 2)
            .saturating_sub(GLYPH_HEIGHT / 2)
            .min(layout.height - GLYPH_HEIGHT);
        // Align the label to the right, next to the grid.
        let x = layout.left - LABEL_PAD - text_width(row);
        draw_number(canvas, row, x, y);
    }
}

/// Returns the width in output pixels of the decimal label for `number`.
fn text_width(number: u32) -> u32 {
    let digits = number.checked_ilog10().unwrap_or(0) + 1;
    digits * GLYPH_WIDTH + (digits - 1) * GLYPH_GAP
}

/// Draws the decimal digits of `number` with the top-left corner at (`x`, `y`).
fn draw_number(canvas: &mut Canvas, number: u32, x: u32, y: u32) {
    let mut glyph_x = x;
    for digit in number.to_string().bytes() {
        draw_glyph(canvas, DIGITS[usize::from(digit - b'0')], glyph_x, y);
        glyph_x += GLYPH_WIDTH + GLYPH_GAP;
    }
}

/// Draws one digit glyph with the top-left corner at (`x`, `y`).
fn draw_glyph(canvas: &mut Canvas, glyph: [u8; GLYPH_ROWS as usize], x: u32, y: u32) {
    for (row, bits) in (0..GLYPH_ROWS).zip(glyph) {
        for column in 0..GLYPH_COLUMNS {
            let mask = 1 << (GLYPH_COLUMNS - 1 - column);
            if bits & mask != 0 {
                fill_dot(canvas, x + column * GLYPH_DOT, y + row * GLYPH_DOT);
            }
        }
    }
}

/// Fills one `GLYPH_DOT` x `GLYPH_DOT` square with its top-left corner at (`x`, `y`).
fn fill_dot(canvas: &mut Canvas, x: u32, y: u32) {
    for dy in 0..GLYPH_DOT {
        for dx in 0..GLYPH_DOT {
            canvas.set(x + dx, y + dy, LABEL_COLOR);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    const CELL: u32 = 8;

    /// Renders a 2 x 1 image: half-transparent red, then transparent.
    fn two_cell_grid() -> (Canvas, GridLayout) {
        let image = parse("A - #FF000080\n. - #00000000\n\nA.\n").unwrap();
        let scale = NonZeroU32::new(CELL).unwrap();
        let canvas = render_grid(&image, scale).unwrap();
        let layout = GridLayout::new(&image, scale).unwrap();
        (canvas, layout)
    }

    #[test]
    fn grid_size_is_margins_plus_cells_plus_lines() {
        let (canvas, layout) = two_cell_grid();
        let pitch = CELL + 1;
        assert_eq!(
            (canvas.width, canvas.height),
            (layout.left + 2 * pitch + 1, layout.top + pitch + 1)
        );
        assert_eq!(
            layout.cell_origin(1, 0).0,
            layout.cell_origin(0, 0).0 + pitch
        );
    }

    #[test]
    fn grid_cells_blend_pixels_over_a_checkerboard() {
        let (canvas, layout) = two_cell_grid();

        // Half-transparent red over the white and the gray checker squares.
        let (ax, ay) = layout.cell_origin(0, 0);
        assert_eq!(canvas.get(ax, ay), [0xFF, 0x7F, 0x7F, 0xFF]);
        assert_eq!(canvas.get(ax + 4, ay), [0xE6, 0x66, 0x66, 0xFF]);
        assert_eq!(canvas.get(ax + 7, ay + 7), [0xFF, 0x7F, 0x7F, 0xFF]);

        // A transparent pixel shows only the checkerboard.
        let (bx, by) = layout.cell_origin(1, 0);
        assert_eq!(canvas.get(bx, by), [0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(canvas.get(bx + 4, by), [0xCC, 0xCC, 0xCC, 0xFF]);
    }

    #[test]
    fn grid_border_and_line_0_are_major_and_line_1_is_minor() {
        let (canvas, layout) = two_cell_grid();
        let (ax, ay) = layout.cell_origin(0, 0);
        let (bx, by) = layout.cell_origin(1, 0);
        assert_eq!(canvas.get(ax - 1, ay), MAJOR_LINE_COLOR);
        assert_eq!(canvas.get(bx - 1, by), MINOR_LINE_COLOR);
        assert_eq!(canvas.get(bx + CELL, by), MAJOR_LINE_COLOR);
        assert_eq!(canvas.get(bx, by - 1), MAJOR_LINE_COLOR);
    }

    #[test]
    fn label_step_is_the_smallest_step_that_leaves_room() {
        // A pitch of 9 px gives 90 px for 10 cells, which is enough for 30 px.
        assert_eq!(label_step(30, 9), 10);
        // A pitch of 2 px gives 20 px for 10 cells and 40 px for 20 cells.
        assert_eq!(label_step(30, 2), 20);
        // A pitch of 1 px needs 50 cells for 30 px.
        assert_eq!(label_step(30, 1), 50);
        assert_eq!(label_step(150, 1), 200);
    }
}
