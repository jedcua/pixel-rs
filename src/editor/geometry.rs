//! Mapping between image pixels and terminal cells.

use ratatui::layout::Rect;

/// Terminal columns for one pixel. A terminal cell is about two times as
/// tall as it is wide, so two columns make a pixel that is about square.
pub const CELL_WIDTH: u16 = 2;

/// An image pixel position. The top-left pixel is (0, 0).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub const fn new(x: u32, y: u32) -> Self {
        Point { x, y }
    }
}

/// The size of an image in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

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
    /// Image position of the first visible pixel.
    pub first: Point,
    /// Number of visible pixel columns.
    pub columns: u32,
    /// Number of visible pixel rows.
    pub rows: u32,
}

impl Viewport {
    /// Makes the viewport for an image of `size` in `area`. The scroll
    /// position `scroll` is clamped so that the viewport stays inside the
    /// image.
    pub fn new(area: Rect, size: Size, scroll: Point) -> Self {
        let horizontal = Axis::new((area.x, area.width), CELL_WIDTH, size.width, scroll.x);
        let vertical = Axis::new((area.y, area.height), 1, size.height, scroll.y);
        Viewport {
            left: horizontal.screen_start,
            top: vertical.screen_start,
            first: Point::new(horizontal.first, vertical.first),
            columns: horizontal.count,
            rows: vertical.count,
        }
    }

    /// Returns the image pixel at the screen `column` and `row`, or `None` if
    /// the cell does not show a pixel.
    pub fn pixel_at(&self, column: u16, row: u16) -> Option<Point> {
        let dx = u32::from(column.checked_sub(self.left)? / CELL_WIDTH);
        let dy = u32::from(row.checked_sub(self.top)?);
        (dx < self.columns && dy < self.rows)
            .then(|| Point::new(self.first.x + dx, self.first.y + dy))
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
pub fn line(from: Point, to: Point) -> Vec<Point> {
    let (x0, y0) = (i64::from(from.x), i64::from(from.y));
    let (x1, y1) = (i64::from(to.x), i64::from(to.y));
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (step_x, step_y) = ((x1 - x0).signum(), (y1 - y0).signum());
    let (mut x, mut y, mut error) = (x0, y0, dx + dy);
    let mut points = Vec::new();
    loop {
        points.push(Point::new(to_u32(x), to_u32(y)));
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
        let size = Size {
            width: 4,
            height: 2,
        };
        let view = Viewport::new(Rect::new(10, 5, 20, 8), size, Point::new(9, 9));
        assert_eq!((view.left, view.top, view.first), (16, 8, Point::new(0, 0)));
        assert_eq!(view.pixel_at(16, 8), Some(Point::new(0, 0)));
        assert_eq!(view.pixel_at(23, 9), Some(Point::new(3, 1)));
        assert_eq!(view.pixel_at(24, 9), None);
        assert_eq!(view.pixel_at(15, 8), None);
    }

    #[test]
    fn clamps_the_scroll_of_a_large_image() {
        let size = Size {
            width: 80,
            height: 95,
        };
        let view = Viewport::new(Rect::new(0, 0, 20, 10), size, Point::new(200, 3));
        assert_eq!(
            (view.first, view.columns, view.rows),
            (Point::new(70, 3), 10, 10)
        );
        assert_eq!(view.pixel_at(19, 9), Some(Point::new(79, 12)));
        assert_eq!(view.screen_of(9, 9), (18, 9));
    }

    #[test]
    fn line_includes_both_ends_and_has_no_gaps() {
        let point = |x, y| Point::new(x, y);
        assert_eq!(line(point(2, 2), point(2, 2)), [point(2, 2)]);
        assert_eq!(
            line(point(3, 1), point(0, 0)),
            [point(3, 1), point(2, 1), point(1, 0), point(0, 0)]
        );
        let points = line(point(0, 0), point(2, 5));
        assert_eq!(
            (points.first(), points.last()),
            (Some(&point(0, 0)), Some(&point(2, 5)))
        );
        assert_eq!(points.len(), 6);
    }
}
