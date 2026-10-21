//! Mapping between image pixels and terminal cells.

use ratatui::layout::Rect;

/// Terminal columns for one pixel. A terminal cell is about two times as
/// tall as it is wide, so two columns make a pixel that is about square.
pub const CELL_WIDTH: u16 = 2;

/// The part of the image that a canvas area shows.
///
/// If the image is smaller than the area, the image is centered. If it is
/// larger, the viewport shows the part that starts at the scroll position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    /// Screen column of the first visible pixel.
    pub left: u16,
    /// Screen row of the first visible pixel.
    pub top: u16,
    /// Image column of the first visible pixel.
    pub first_x: u32,
    /// Image row of the first visible pixel.
    pub first_y: u32,
    /// Number of visible pixel columns.
    pub columns: u32,
    /// Number of visible pixel rows.
    pub rows: u32,
}

impl Viewport {
    /// Makes the viewport for an image of `size` (width, height) in `area`.
    /// The scroll position `scroll` (x, y) is clamped so that the viewport
    /// stays inside the image.
    pub fn new(area: Rect, size: (u32, u32), scroll: (u32, u32)) -> Self {
        let horizontal = Axis::new((area.x, area.width), CELL_WIDTH, size.0, scroll.0);
        let vertical = Axis::new((area.y, area.height), 1, size.1, scroll.1);
        Viewport {
            left: horizontal.screen_start,
            top: vertical.screen_start,
            first_x: horizontal.first,
            first_y: vertical.first,
            columns: horizontal.count,
            rows: vertical.count,
        }
    }

    /// Returns the image pixel at the screen `column` and `row`, or `None` if
    /// the cell does not show a pixel.
    pub fn pixel_at(&self, column: u16, row: u16) -> Option<(u32, u32)> {
        let dx = u32::from(column.checked_sub(self.left)? / CELL_WIDTH);
        let dy = u32::from(row.checked_sub(self.top)?);
        (dx < self.columns && dy < self.rows).then(|| (self.first_x + dx, self.first_y + dy))
    }

    /// Returns the screen column and row of the visible pixel number
    /// (`dx`, `dy`), counted from the first visible pixel.
    pub fn screen_of(&self, dx: u32, dy: u32) -> (u16, u16) {
        let dx = u16::try_from(dx).expect("visible pixel column fits on screen");
        let dy = u16::try_from(dy).expect("visible pixel row fits on screen");
        (self.left + dx * CELL_WIDTH, self.top + dy)
    }
}

/// The viewport on one axis.
struct Axis {
    screen_start: u16,
    first: u32,
    count: u32,
}

impl Axis {
    /// `screen` is the start and the length of the area on the screen. `cell`
    /// is the number of terminal cells for one pixel. `image` is the image
    /// length.
    fn new((start, length): (u16, u16), cell: u16, image: u32, scroll: u32) -> Self {
        let capacity = length / cell;
        let count = u16::try_from(image).map_or(capacity, |image| image.min(capacity));
        let first = scroll.min(image - u32::from(count));
        let padding = (length - count * cell) / 2;
        Axis {
            screen_start: start + padding,
            first,
            count: u32::from(count),
        }
    }
}

/// Returns the pixels on the line from `from` to `to`, both included, in
/// order. A mouse drag skips cells when it is fast. This line fills the gap.
pub fn line(from: (u32, u32), to: (u32, u32)) -> Vec<(u32, u32)> {
    let (x0, y0) = (i64::from(from.0), i64::from(from.1));
    let (x1, y1) = (i64::from(to.0), i64::from(to.1));
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (step_x, step_y) = ((x1 - x0).signum(), (y1 - y0).signum());
    let (mut x, mut y, mut error) = (x0, y0, dx + dy);
    let mut points = Vec::new();
    loop {
        points.push((to_u32(x), to_u32(y)));
        if x == x1 && y == y1 {
            return points;
        }
        let double = 2 * error;
        if double >= dy {
            error += dy;
            x += step_x;
        }
        if double <= dx {
            error += dx;
            y += step_y;
        }
    }
}

/// Converts a line coordinate back to `u32`. The line stays between its two
/// `u32` end points, so the conversion cannot fail.
fn to_u32(value: i64) -> u32 {
    u32::try_from(value).expect("line point is between the end points")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_a_small_image_and_maps_cells_to_pixels() {
        let view = Viewport::new(Rect::new(10, 5, 20, 8), (4, 2), (9, 9));
        assert_eq!(
            (view.left, view.top, view.first_x, view.first_y),
            (16, 8, 0, 0)
        );
        assert_eq!(view.pixel_at(16, 8), Some((0, 0)));
        assert_eq!(view.pixel_at(23, 9), Some((3, 1)));
        assert_eq!(view.pixel_at(24, 9), None);
        assert_eq!(view.pixel_at(15, 8), None);
    }

    #[test]
    fn clamps_the_scroll_of_a_large_image() {
        let view = Viewport::new(Rect::new(0, 0, 20, 10), (80, 95), (200, 3));
        assert_eq!(
            (view.first_x, view.first_y, view.columns, view.rows),
            (70, 3, 10, 10)
        );
        assert_eq!(view.pixel_at(19, 9), Some((79, 12)));
        assert_eq!(view.screen_of(9, 9), (18, 9));
    }

    #[test]
    fn line_includes_both_ends_and_has_no_gaps() {
        assert_eq!(line((2, 2), (2, 2)), [(2, 2)]);
        assert_eq!(line((3, 1), (0, 0)), [(3, 1), (2, 1), (1, 0), (0, 0)]);
        let points = line((0, 0), (2, 5));
        assert_eq!(
            (points.first(), points.last()),
            (Some(&(0, 0)), Some(&(2, 5)))
        );
        assert_eq!(points.len(), 6);
    }
}
