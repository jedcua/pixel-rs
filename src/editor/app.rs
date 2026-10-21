//! Editor state and input.

use std::fs;
use std::path::{Path, PathBuf};

use pxl::Document;
use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::{Position, Rect};

use super::form::{EntryForm, Field, RemoveForm};
use super::geometry::{Viewport, line};

/// Number of pixels that one mouse wheel step scrolls.
const WHEEL_STEP: i64 = 3;

/// A command from a button or a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Save,
    Quit,
    Add,
    Edit,
    Remove,
    Select(usize),
    Focus(Field),
    Submit,
    /// Closes the dialog.
    Close,
    SaveAndQuit,
    /// Quits and does not save.
    Exit,
    /// Selects the color of the pixel below the pointer.
    Pick,
}

/// A modal dialog. When a dialog is open, it gets all the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    Entry(EntryForm),
    Remove(RemoveForm),
    /// The user quits. `unsaved` is true if the document has changes that
    /// are not saved.
    Quit {
        unsaved: bool,
    },
    /// The result of a save.
    Alert(Alert),
}

/// A message in a dialog with an OK button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub title: String,
    pub text: String,
    /// True if the message is about a failure.
    pub error: bool,
}

/// The screen areas of the last frame, for mouse input.
#[derive(Debug, Clone, Default)]
pub struct Screen {
    /// The area inside the canvas border.
    pub canvas: Rect,
    /// The clickable areas. When a dialog is open, only its areas are here.
    pub hits: Vec<(Rect, Action)>,
}

impl Screen {
    fn hit(&self, column: u16, row: u16) -> Option<Action> {
        let position = Position::new(column, row);
        self.hits
            .iter()
            .find(|(area, _)| area.contains(position))
            .map(|&(_, action)| action)
    }
}

pub struct App {
    pub path: PathBuf,
    pub document: Document,
    /// The palette index that the pen draws with.
    pub selected: usize,
    pub modified: bool,
    /// The image pixel at the top-left corner of the canvas.
    pub scroll: (u32, u32),
    /// The image pixel below the mouse pointer.
    pub hover: Option<(u32, u32)>,
    pub dialog: Option<Dialog>,
    /// The result of the last command, for the status bar.
    pub message: Option<String>,
    pub quit: bool,
    /// A message to print after the editor closes, for a save and quit.
    pub exit_note: Option<String>,
    screen: Screen,
    /// The last pixel of the current mouse stroke.
    stroke: Option<(u32, u32)>,
}

impl App {
    pub fn new(path: PathBuf, document: Document) -> Self {
        App {
            path,
            document,
            selected: 0,
            modified: false,
            scroll: (0, 0),
            hover: None,
            dialog: None,
            message: None,
            quit: false,
            exit_note: None,
            screen: Screen::default(),
            stroke: None,
        }
    }

    /// Returns the viewport of the canvas in the last frame.
    pub fn viewport(&self) -> Viewport {
        Viewport::new(self.screen.canvas, self.size(), self.scroll)
    }

    /// Stores the screen areas of a new frame.
    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
        let view = self.viewport();
        self.scroll = (view.first_x, view.first_y);
    }

    pub fn handle(&mut self, event: &Event) {
        match event {
            // Windows also reports key releases. Use only key presses.
            Event::Key(key) if key.kind == KeyEventKind::Press => self.key(key),
            Event::Mouse(mouse) => self.mouse(*mouse),
            _ => {}
        }
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::Save => {
                self.save();
            }
            Action::Quit => self.request_quit(),
            Action::Add => self.open_add(),
            Action::Edit => self.open_edit(),
            Action::Remove => self.open_remove(),
            Action::Select(index) => self.selected = index,
            Action::Focus(field) => self.focus(field),
            Action::Submit => self.submit(),
            Action::Close => self.dialog = None,
            Action::SaveAndQuit => self.save_and_quit(),
            Action::Exit => self.quit = true,
            Action::Pick => self.pick(self.hover),
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.document.width(), self.document.height())
    }

    fn key(&mut self, key: &KeyEvent) {
        let action = match &mut self.dialog {
            None => main_key(key),
            Some(Dialog::Quit { unsaved }) => quit_key(*unsaved, key),
            Some(Dialog::Alert(_)) => alert_key(key),
            Some(Dialog::Entry(form)) => entry_key(form, key),
            Some(Dialog::Remove(form)) => remove_key(form, key),
        };
        if let Some(action) = action {
            self.perform(action);
        } else if self.dialog.is_none() {
            self.navigate_key(key.code);
        }
    }

    fn navigate_key(&mut self, code: KeyCode) {
        let count = i64::try_from(self.document.palette().len()).unwrap_or(i64::MAX);
        let selected = i64::try_from(self.selected).unwrap_or(0);
        match code {
            KeyCode::Tab => self.select((selected + 1).rem_euclid(count)),
            KeyCode::BackTab => self.select((selected - 1).rem_euclid(count)),
            KeyCode::Left => self.scroll_by(-1, 0),
            KeyCode::Right => self.scroll_by(1, 0),
            KeyCode::Up => self.scroll_by(0, -1),
            KeyCode::Down => self.scroll_by(0, 1),
            _ => {}
        }
    }

    fn select(&mut self, index: i64) {
        self.selected = usize::try_from(index).unwrap_or(0);
    }

    fn mouse(&mut self, mouse: MouseEvent) {
        let (column, row) = (mouse.column, mouse.row);
        let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.click(column, row),
            MouseEventKind::Drag(MouseButton::Left) => self.drag(column, row),
            MouseEventKind::Up(MouseButton::Left) => self.stroke = None,
            MouseEventKind::Down(MouseButton::Right) => {
                self.pick(self.viewport().pixel_at(column, row));
            }
            MouseEventKind::Moved => self.hover = self.viewport().pixel_at(column, row),
            MouseEventKind::ScrollUp if shift => self.scroll_by(-WHEEL_STEP, 0),
            MouseEventKind::ScrollDown if shift => self.scroll_by(WHEEL_STEP, 0),
            MouseEventKind::ScrollUp => self.scroll_by(0, -WHEEL_STEP),
            MouseEventKind::ScrollDown => self.scroll_by(0, WHEEL_STEP),
            MouseEventKind::ScrollLeft => self.scroll_by(-WHEEL_STEP, 0),
            MouseEventKind::ScrollRight => self.scroll_by(WHEEL_STEP, 0),
            _ => {}
        }
    }

    fn click(&mut self, column: u16, row: u16) {
        if let Some(action) = self.screen.hit(column, row) {
            self.perform(action);
            return;
        }
        if self.dialog.is_some() {
            return;
        }
        if let Some(pixel) = self.viewport().pixel_at(column, row) {
            self.paint(pixel);
            self.stroke = Some(pixel);
        }
    }

    fn drag(&mut self, column: u16, row: u16) {
        let Some(pixel) = self.viewport().pixel_at(column, row) else {
            return;
        };
        self.hover = Some(pixel);
        let Some(from) = self.stroke else {
            return;
        };
        for point in line(from, pixel) {
            self.paint(point);
        }
        self.stroke = Some(pixel);
    }

    /// Selects the color of `pixel`, so that the pen draws with it. A dialog
    /// gets all the input, so this does nothing while a dialog is open.
    fn pick(&mut self, pixel: Option<(u32, u32)>) {
        if let (Some((x, y)), None) = (pixel, &self.dialog) {
            self.selected = self.document.pixel(x, y);
        }
    }

    fn paint(&mut self, (x, y): (u32, u32)) {
        if self.document.set_pixel(x, y, self.selected) {
            self.modified = true;
        }
    }

    fn scroll_by(&mut self, dx: i64, dy: i64) {
        let shift = |value: u32, delta: i64| {
            u32::try_from((i64::from(value) + delta).max(0)).unwrap_or(u32::MAX)
        };
        let scroll = (shift(self.scroll.0, dx), shift(self.scroll.1, dy));
        let view = Viewport::new(self.screen.canvas, self.size(), scroll);
        self.scroll = (view.first_x, view.first_y);
    }

    /// Writes the document to its file, and opens an alert with the result.
    /// Returns true if the save succeeds.
    fn save(&mut self) -> bool {
        let text = self.document.to_pxl();
        let result = write_and_check(&self.path, &text);
        let alert = match &result {
            Ok(()) => {
                self.modified = false;
                let bytes = text.len();
                let saved = format!("Saved {bytes} bytes to {}.", self.path.display());
                self.message = Some(saved.clone());
                Alert {
                    title: " Saved ".into(),
                    text: saved,
                    error: false,
                }
            }
            Err(error) => Alert {
                title: " Save failed ".into(),
                text: error.clone(),
                error: true,
            },
        };
        self.dialog = Some(Dialog::Alert(alert));
        result.is_ok()
    }

    /// Saves and quits. If the save fails, the editor stays open and shows
    /// the failure.
    fn save_and_quit(&mut self) {
        if self.save() {
            self.exit_note = self.message.take();
            self.quit = true;
        }
    }

    fn request_quit(&mut self) {
        self.dialog = Some(Dialog::Quit {
            unsaved: self.modified,
        });
    }

    fn open_add(&mut self) {
        let form = EntryForm::add(&self.document, self.selected);
        self.dialog = Some(Dialog::Entry(form));
    }

    fn open_edit(&mut self) {
        let form = EntryForm::edit(&self.document, self.selected);
        self.dialog = Some(Dialog::Entry(form));
    }

    fn open_remove(&mut self) {
        if self.document.palette().len() == 1 {
            self.message = Some("Cannot delete the last color. Add a color first.".into());
            return;
        }
        let form = RemoveForm::new(&self.document, self.selected);
        self.dialog = Some(Dialog::Remove(form));
    }

    fn focus(&mut self, field: Field) {
        if let Some(Dialog::Entry(form)) = &mut self.dialog {
            form.field = field;
        }
    }

    fn submit(&mut self) {
        let result = match &self.dialog {
            Some(Dialog::Entry(form)) => self.submit_entry(&form.clone()),
            Some(Dialog::Remove(form)) => self.submit_remove(&form.clone()),
            Some(Dialog::Quit { .. } | Dialog::Alert(_)) | None => return,
        };
        match (result, &mut self.dialog) {
            (Ok(()), _) => self.dialog = None,
            (Err(error), Some(Dialog::Entry(form))) => form.error = Some(error),
            (Err(error), Some(Dialog::Remove(form))) => form.error = Some(error),
            (Err(_), _) => {}
        }
    }

    fn submit_entry(&mut self, form: &EntryForm) -> Result<(), String> {
        let entry = form.entry()?;
        let index = match form.index {
            Some(index) => self.document.set_entry(index, entry).map(|()| index),
            None => self.document.add_entry(entry),
        };
        self.selected = index.map_err(|err| capitalize(&err.to_string()))?;
        self.modified = true;
        Ok(())
    }

    fn submit_remove(&mut self, form: &RemoveForm) -> Result<(), String> {
        let replacement = form.replacement_index(&self.document)?;
        self.document
            .remove_entry(form.index, replacement)
            .map_err(|err| capitalize(&err.to_string()))?;
        self.selected = if replacement > form.index {
            replacement - 1
        } else {
            replacement
        };
        self.modified = true;
        Ok(())
    }
}

/// Returns the action for a key in the main view.
fn main_key(key: &KeyEvent) -> Option<Action> {
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char('s') if control => Some(Action::Save),
        KeyCode::Char('q') if control => Some(Action::Quit),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('a') => Some(Action::Add),
        KeyCode::Char('e') | KeyCode::Enter => Some(Action::Edit),
        KeyCode::Char('d') | KeyCode::Delete => Some(Action::Remove),
        KeyCode::Char('i') => Some(Action::Pick),
        _ => None,
    }
}

/// Returns the action for a key in the quit dialog.
fn quit_key(unsaved: bool, key: &KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('s') if unsaved => Some(Action::SaveAndQuit),
        KeyCode::Char('d') if unsaved => Some(Action::Exit),
        KeyCode::Enter | KeyCode::Char('q' | 'y') if !unsaved => Some(Action::Exit),
        KeyCode::Esc | KeyCode::Char('n') => Some(Action::Close),
        _ => None,
    }
}

/// Returns the action for a key in an alert.
fn alert_key(key: &KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => Some(Action::Close),
        _ => None,
    }
}

/// Writes `text` to `path`, then reads the file again and compares it with
/// `text`, so that a success means that the file has the text.
fn write_and_check(path: &Path, text: &str) -> Result<(), String> {
    let name = path.display();
    fs::write(path, text).map_err(|err| {
        format!("Cannot write {name}: {err}. Check the path and the permissions.")
    })?;
    let written = fs::read_to_string(path)
        .map_err(|err| format!("Cannot read {name} after the save: {err}. Save again."))?;
    if written != text {
        return Err(format!(
            "{name} has different text after the save. Save again."
        ));
    }
    Ok(())
}

/// Changes the entry form for a key, or returns the action for the key.
fn entry_key(form: &mut EntryForm, key: &KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Enter => return Some(Action::Submit),
        KeyCode::Esc => return Some(Action::Close),
        KeyCode::Tab | KeyCode::BackTab => form.next_field(),
        KeyCode::Backspace => form.backspace(),
        KeyCode::Char(typed) => form.type_char(typed),
        _ => {}
    }
    form.error = None;
    None
}

/// Changes the remove form for a key, or returns the action for the key.
fn remove_key(form: &mut RemoveForm, key: &KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Enter => return Some(Action::Submit),
        KeyCode::Esc => return Some(Action::Close),
        KeyCode::Backspace => form.replacement = None,
        KeyCode::Char(typed) => form.replacement = Some(typed),
        _ => {}
    }
    form.error = None;
    None
}

/// Makes the first char uppercase and ends the text with a period, so that a
/// library error reads as a status message.
fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    let first = chars.next().map(|first| first.to_uppercase().to_string());
    format!("{}{}.", first.unwrap_or_default(), chars.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns an editor for a 3 x 2 image in a 10 x 4 canvas. The viewport
    /// centers the image, so pixel (x, y) is at column 2 + 2x and row 1 + y.
    fn app() -> App {
        let source = "O - #000000FF\nX - #FFFFFFFF\nr - #FF0000FF\n\nXXX\nXOr\n";
        let mut app = App::new(PathBuf::from("test.pxl"), Document::parse(source).unwrap());
        app.set_screen(Screen {
            canvas: Rect::new(0, 0, 10, 4),
            hits: Vec::new(),
        });
        app
    }

    fn mouse(kind: MouseEventKind, column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    #[test]
    fn right_click_picks_the_pixel_color_and_does_not_paint() {
        let mut app = app();
        app.handle(&mouse(MouseEventKind::Down(MouseButton::Right), 7, 2));
        assert_eq!(app.selected, 2);
        assert!(!app.modified);
        app.handle(&mouse(MouseEventKind::Down(MouseButton::Right), 0, 0));
        assert_eq!(app.selected, 2, "a click outside the image keeps the color");
    }

    #[test]
    fn pick_key_uses_the_pixel_below_the_pointer() {
        let mut app = app();
        app.selected = 1;
        app.handle(&mouse(MouseEventKind::Moved, 4, 2));
        app.perform(main_key(&KeyEvent::from(KeyCode::Char('i'))).unwrap());
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn pick_does_nothing_while_a_dialog_is_open() {
        let mut app = app();
        app.dialog = Some(Dialog::Quit { unsaved: false });
        app.handle(&mouse(MouseEventKind::Down(MouseButton::Right), 7, 2));
        assert_eq!(app.selected, 0);
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::from(code))
    }

    fn alert(app: &App) -> &Alert {
        match &app.dialog {
            Some(Dialog::Alert(alert)) => alert,
            other => panic!("expected an alert, found {other:?}"),
        }
    }

    #[test]
    fn save_writes_the_file_and_shows_a_success_alert() {
        let path = std::env::temp_dir().join(format!("pxl-save-{}.pxl", std::process::id()));
        let mut app = app();
        app.path.clone_from(&path);
        app.handle(&mouse(MouseEventKind::Down(MouseButton::Left), 2, 1));
        app.perform(Action::Save);
        let written = fs::read_to_string(&path).unwrap();
        fs::remove_file(&path).unwrap();

        assert_eq!(written, app.document.to_pxl());
        assert!(written.ends_with("\n\nOXX\nXOr\n"));
        assert!(!alert(&app).error);
        assert!(!app.modified);
        app.handle(&key(KeyCode::Enter));
        assert_eq!(app.dialog, None);
    }

    #[test]
    fn failed_save_shows_an_error_and_keeps_the_editor_open() {
        let mut app = app();
        app.path = PathBuf::from("/nonexistent-pxl-dir/test.pxl");
        app.modified = true;
        app.perform(Action::SaveAndQuit);
        assert!(alert(&app).error);
        assert!(alert(&app).text.starts_with("Cannot write"));
        assert!(app.modified && !app.quit);
    }

    #[test]
    fn quit_asks_for_confirmation_without_unsaved_changes() {
        let mut app = app();
        app.perform(Action::Quit);
        assert_eq!(app.dialog, Some(Dialog::Quit { unsaved: false }));
        app.handle(&key(KeyCode::Esc));
        assert!(app.dialog.is_none() && !app.quit);
        app.perform(Action::Quit);
        app.handle(&key(KeyCode::Char('y')));
        assert!(app.quit);
    }

    #[test]
    fn quit_with_unsaved_changes_ignores_the_simple_confirm_key() {
        let mut app = app();
        app.modified = true;
        app.perform(Action::Quit);
        app.handle(&key(KeyCode::Char('y')));
        assert!(!app.quit, "`y` must not discard unsaved changes");
        app.handle(&key(KeyCode::Char('d')));
        assert!(app.quit);
    }
}
