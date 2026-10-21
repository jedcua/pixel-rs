//! Drawing of the editor screen.

use pxl::{CHECKER, Rgba};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Margin, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use super::app::{Action, Alert, App, Dialog, Hit, Screen};
use super::form::{EntryForm, Field, RemoveForm};

/// Width of the palette panel, with its border.
const PANEL_WIDTH: u16 = 24;
/// Columns between two buttons.
const BUTTON_GAP: u16 = 1;
/// Width of a dialog, with its border. The longest message must fit.
const DIALOG_WIDTH: u16 = 60;
/// Width of the value of a text field. The longest value is a color text.
const FIELD_WIDTH: usize = 9;
/// The label of the color field.
const COLOR_LABEL: &str = "Color: ";

const BUTTON: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::Gray)
    .add_modifier(Modifier::BOLD);
const SELECTED: Style = Style::new().add_modifier(Modifier::REVERSED);
const MUTED: Style = Style::new().fg(Color::DarkGray);
const ERROR: Style = Style::new().fg(Color::LightRed);
const SUCCESS: Style = Style::new().fg(Color::LightGreen);

/// A button label and the action that the button runs.
#[derive(Clone, Copy)]
struct Button<'a> {
    label: &'a str,
    action: Action,
}

impl<'a> Button<'a> {
    const fn new(label: &'a str, action: Action) -> Self {
        Button { label, action }
    }

    fn width(&self) -> u16 {
        u16::try_from(self.label.chars().count() + 2).unwrap_or(u16::MAX)
    }
}

/// Draws the editor and returns the screen areas for mouse input.
pub fn draw(frame: &mut Frame, app: &App) -> Screen {
    let [top, body, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let [canvas, panel] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(PANEL_WIDTH)]).areas(body);

    let mut hits = Vec::new();
    draw_top_bar(frame, top, app, &mut hits);
    let canvas = draw_canvas(frame, canvas, app);
    draw_palette(frame, panel, app, &mut hits);
    draw_status(frame, status, app);
    if let Some(dialog) = &app.dialog {
        hits.clear();
        draw_dialog(frame, dialog, app, &mut hits);
    }
    Screen { canvas, hits }
}

/// Draws the file name and the Save and Quit buttons.
fn draw_top_bar(frame: &mut Frame, area: Rect, app: &App, hits: &mut Vec<Hit>) {
    let changed = if app.modified { " [modified]" } else { "" };
    let size = app.image_size();
    let title = format!(
        " pxl  {}{changed}  {}x{}",
        app.path.display(),
        size.width,
        size.height
    );
    frame.render_widget(Paragraph::new(title).style(MUTED), area);

    let buttons = [
        Button::new("Save", Action::Save),
        Button::new("Quit", Action::Quit),
    ];
    let start = area.right().saturating_sub(buttons_width(&buttons) + 1);
    draw_buttons(frame, Position::new(start, area.y), &buttons, hits);
}

/// Draws the border and the visible pixels, and returns the inner area.
fn draw_canvas(frame: &mut Frame, area: Rect, app: &App) -> Rect {
    let block = Block::bordered().title(" Canvas ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let view = app.viewport_in(inner);
    let buffer = frame.buffer_mut();
    for dy in 0..view.rows {
        for dx in 0..view.columns {
            let (x, y) = (view.first.x + dx, view.first.y + dy);
            let color = app.document.palette()[app.document.pixel(x, y)].color;
            let (column, row) = view.screen_of(dx, dy);
            let fill = color.over(CHECKER[((x + y) % 2) as usize]);
            paint_cells(buffer, Position::new(column, row), [fill, fill]);
        }
    }
    inner
}

/// Draws the palette list and the Add, Edit, and Delete buttons.
fn draw_palette(frame: &mut Frame, area: Rect, app: &App, hits: &mut Vec<Hit>) {
    let count = app.document.palette().len();
    let block = Block::bordered().title(format!(" Palette ({count}) "));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [list, buttons] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(inner);

    let visible = usize::from(list.height);
    let first = app.selected.saturating_sub(visible.saturating_sub(1));
    for (row, index) in (list.y..list.bottom()).zip(first..count) {
        let row_area = Rect::new(list.x, row, list.width, 1);
        draw_palette_row(frame, row_area, app, index);
        hits.push(Hit {
            area: row_area,
            action: Action::Select(index),
        });
    }

    let actions = [
        Button::new("Add", Action::Add),
        Button::new("Edit", Action::Edit),
        Button::new("Delete", Action::Remove),
    ];
    draw_buttons(frame, buttons.as_position(), &actions, hits);
}

/// Draws one palette row: a swatch, the key, and the color.
fn draw_palette_row(frame: &mut Frame, area: Rect, app: &App, index: usize) {
    let entry = app.document.palette()[index];
    let style = if index == app.selected {
        SELECTED
    } else {
        Style::new()
    };
    let marker = if index == app.selected { ">" } else { " " };
    let text = format!("{marker}    {}  {}", entry.key, entry.color);
    frame.render_widget(Paragraph::new(text).style(style), area);
    paint_swatch(
        frame.buffer_mut(),
        Position::new(area.x + 2, area.y),
        entry.color,
    );
}

/// Draws the pixel below the pointer, the selected color, and a message.
fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let entry = app.document.palette()[app.selected];
    let pointer = app.hover.map_or_else(String::new, |pixel| {
        let key = app.document.palette()[app.document.pixel(pixel.x, pixel.y)].key;
        format!("x {}, y {}: {key}  |  ", pixel.x, pixel.y)
    });
    let usage = app.document.usage(app.selected);
    let message = app
        .message
        .as_deref()
        .unwrap_or("Drag: draw  Right-click/i: pick  Tab: next color  ^S: save  q: quit");
    let text = format!(
        " {pointer}pen {} {} ({usage} px)  |  {message}",
        entry.key, entry.color
    );
    frame.render_widget(Paragraph::new(text).style(MUTED), area);
}

fn draw_dialog(frame: &mut Frame, dialog: &Dialog, app: &App, hits: &mut Vec<Hit>) {
    match dialog {
        Dialog::Entry(form) => draw_entry_dialog(frame, form, app, hits),
        Dialog::Remove(form) => draw_remove_dialog(frame, form, app, hits),
        Dialog::Quit { unsaved } => draw_quit_dialog(frame, *unsaved, hits),
        Dialog::Alert(alert) => draw_alert(frame, alert, hits),
    }
}

fn draw_entry_dialog(frame: &mut Frame, form: &EntryForm, app: &App, hits: &mut Vec<Hit>) {
    let title = match form.index {
        None => " Add color ".to_string(),
        Some(index) => format!(" Edit color `{}` ", app.document.palette()[index].key),
    };
    let inner = open_dialog(frame, &title, 7);
    let key = form.key.map(String::from).unwrap_or_default();
    let fields = [
        (Field::Key, "Key:   ", key),
        (Field::Color, COLOR_LABEL, form.color.clone()),
    ];
    for (row, (field, label, value)) in (inner.y..).zip(fields) {
        let area = Rect::new(inner.x, row, inner.width, 1);
        let text = TextField {
            label,
            value: &value,
            active: form.field == field,
        };
        draw_field(frame, area, &text);
        hits.push(Hit {
            area,
            action: Action::Focus(field),
        });
    }
    if let Some(color) = Rgba::from_lenient_hex(&form.color) {
        paint_swatch(
            frame.buffer_mut(),
            Position::new(inner.x + field_end(COLOR_LABEL) + 2, inner.y + 1),
            color,
        );
    }
    let hint = "Type #RRGGBB or #RRGGBBAA. Tab: next field.";
    draw_line(frame, inner, 3, Span::styled(hint, MUTED));
    draw_error(frame, inner, form.error.as_deref());
    let buttons = [
        Button::new("OK", Action::Submit),
        Button::new("Cancel", Action::Close),
    ];
    let start = Position::new(inner.x, inner.bottom() - 1);
    draw_buttons(frame, start, &buttons, hits);
}

fn draw_remove_dialog(frame: &mut Frame, form: &RemoveForm, app: &App, hits: &mut Vec<Hit>) {
    let key = app.document.palette()[form.index].key;
    let inner = open_dialog(frame, &format!(" Delete color `{key}` "), 7);
    if form.usage == 0 {
        draw_line(frame, inner, 0, Span::raw("No pixels use this color."));
    } else {
        let text = format!("{} pixels use this color.", form.usage);
        draw_line(frame, inner, 0, Span::raw(text));
        let replacement = form.replacement.map(String::from).unwrap_or_default();
        let area = Rect::new(inner.x, inner.y + 1, inner.width, 1);
        let field = TextField {
            label: "Replace with key: ",
            value: &replacement,
            active: true,
        };
        draw_field(frame, area, &field);
        let hint = "Type a palette key.";
        draw_line(frame, inner, 3, Span::styled(hint, MUTED));
    }
    draw_error(frame, inner, form.error.as_deref());
    let buttons = [
        Button::new("Delete", Action::Submit),
        Button::new("Cancel", Action::Close),
    ];
    let start = Position::new(inner.x, inner.bottom() - 1);
    draw_buttons(frame, start, &buttons, hits);
}

fn draw_quit_dialog(frame: &mut Frame, unsaved: bool, hits: &mut Vec<Hit>) {
    let (title, text, buttons): (_, _, &[Button]) = if unsaved {
        (
            " Unsaved changes ",
            "Save the changes before you quit?",
            &[
                Button::new("Save and quit (s)", Action::SaveAndQuit),
                Button::new("Discard (d)", Action::Discard),
                Button::new("Cancel", Action::Close),
            ],
        )
    } else {
        (
            " Quit ",
            "Quit the editor?",
            &[
                Button::new("Quit (y)", Action::Discard),
                Button::new("Cancel (n)", Action::Close),
            ],
        )
    };
    let inner = open_dialog(frame, title, 4);
    draw_line(frame, inner, 0, Span::raw(text));
    let start = Position::new(inner.x, inner.bottom() - 1);
    draw_buttons(frame, start, buttons, hits);
}

/// Draws the result of a save, with an OK button. The text wraps, so that a
/// long path stays visible.
fn draw_alert(frame: &mut Frame, alert: &Alert, hits: &mut Vec<Hit>) {
    let inner = open_dialog(frame, &alert.title, 5);
    let style = if alert.error { ERROR } else { SUCCESS };
    let text_area = Rect::new(inner.x, inner.y, inner.width, inner.height - 2);
    let text = Paragraph::new(alert.text.as_str())
        .style(style)
        .wrap(Wrap { trim: false });
    frame.render_widget(text, text_area);
    let start = Position::new(inner.x, inner.bottom() - 1);
    draw_buttons(frame, start, &[Button::new("OK", Action::Close)], hits);
}

/// Clears a centered box with a border and returns its inner area. `height`
/// is the number of inner rows.
fn open_dialog(frame: &mut Frame, title: &str, height: u16) -> Rect {
    let area = frame.area();
    let width = DIALOG_WIDTH.min(area.width);
    let height = (height + 2).min(area.height);
    let outer = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    let block = Block::bordered().title(title.to_string());
    let inner = block.inner(outer).inner(Margin::new(1, 0));
    frame.render_widget(Clear, outer);
    frame.render_widget(block, outer);
    inner
}

/// Returns the column offset of the end of the value of a field with `label`.
fn field_end(label: &str) -> u16 {
    u16::try_from(label.len() + FIELD_WIDTH).unwrap_or(u16::MAX)
}

/// A text field: a label and a value. The active field shows the cursor.
struct TextField<'a> {
    label: &'a str,
    value: &'a str,
    active: bool,
}

fn draw_field(frame: &mut Frame, area: Rect, field: &TextField) {
    let TextField {
        label,
        value,
        active,
    } = *field;
    let style = if active { SELECTED } else { Style::new() };
    let line = Line::from(vec![
        Span::raw(label),
        Span::styled(format!("{value:<FIELD_WIDTH$}"), style),
    ]);
    frame.render_widget(Paragraph::new(line), area);
    if active {
        let offset = u16::try_from(label.len() + value.chars().count()).unwrap_or(0);
        frame.set_cursor_position(Position::new(area.x + offset, area.y));
    }
}

fn draw_error(frame: &mut Frame, inner: Rect, error: Option<&str>) {
    if let Some(error) = error {
        draw_line(frame, inner, 4, Span::styled(error, ERROR));
    }
}

/// Draws `span` on row `row` of the dialog area `inner`.
fn draw_line(frame: &mut Frame, inner: Rect, row: u16, span: Span) {
    if row < inner.height {
        let area = Rect::new(inner.x, inner.y + row, inner.width, 1);
        frame.render_widget(Paragraph::new(Line::from(span)), area);
    }
}

/// Draws a row of buttons that starts at `start`, and records their areas.
fn draw_buttons(frame: &mut Frame, start: Position, buttons: &[Button], hits: &mut Vec<Hit>) {
    let mut column = start.x;
    for button in buttons {
        let width = button.width();
        let area = Rect::new(column, start.y, width, 1).intersection(frame.area());
        let text = format!(" {} ", button.label);
        frame.render_widget(Paragraph::new(text).style(BUTTON), area);
        hits.push(Hit {
            area,
            action: button.action,
        });
        column += width + BUTTON_GAP;
    }
}

fn buttons_width(buttons: &[Button]) -> u16 {
    let gaps = u16::try_from(buttons.len().saturating_sub(1)).unwrap_or(0) * BUTTON_GAP;
    buttons.iter().map(Button::width).sum::<u16>() + gaps
}

/// Draws a swatch of two cells. The left cell shows the color over the light
/// checker color and the right cell over the dark one, so that transparency
/// is visible.
fn paint_swatch(buffer: &mut Buffer, position: Position, color: Rgba) {
    paint_cells(buffer, position, CHECKER.map(|back| color.over(back)));
}

/// Fills the two cells that start at `position` with full blocks.
fn paint_cells(buffer: &mut Buffer, position: Position, fills: [[u8; 3]; 2]) {
    for (offset, [red, green, blue]) in (0..).zip(fills) {
        if let Some(cell) = buffer.cell_mut(Position::new(position.x + offset, position.y)) {
            let fill = Color::Rgb(red, green, blue);
            cell.set_char('█').set_fg(fill).set_bg(fill);
        }
    }
}
