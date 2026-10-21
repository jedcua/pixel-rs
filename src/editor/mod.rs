//! Terminal editor for `.pxl` files.

mod app;
mod form;
mod geometry;
mod view;

use std::fs;
use std::io::{self, stdout};
use std::path::Path;

use pxl::Document;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;

use app::{App, Screen};

/// Opens the `.pxl` file at `path` in the terminal editor.
///
/// # Errors
///
/// Returns a message if the file cannot be read or parsed, or if the
/// terminal fails.
pub fn run(path: &Path) -> Result<(), String> {
    let name = path.display();
    let source = fs::read_to_string(path).map_err(|err| format!("cannot read {name}: {err}"))?;
    let document = Document::parse(&source).map_err(|err| format!("{name}: {err}"))?;
    let mut app = App::new(path.to_path_buf(), document);

    set_panic_hook();
    let mut terminal =
        ratatui::try_init().map_err(|err| format!("cannot start the editor: {err}"))?;
    let result =
        execute!(stdout(), EnableMouseCapture).and_then(|()| event_loop(&mut terminal, &mut app));
    // Restore the terminal also if the loop fails, so that the shell works.
    let _ = execute!(stdout(), DisableMouseCapture);
    ratatui::restore();
    result.map_err(|err| format!("terminal failed: {err}"))?;
    // The alert of a save and quit closes with the editor, so print it.
    if let Some(note) = app.exit_note() {
        println!("{note}");
    }
    Ok(())
}

fn event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.is_done() {
        let mut screen = Screen::default();
        terminal.draw(|frame| screen = view::draw(frame, app))?;
        app.set_screen(screen);
        app.handle(&event::read()?);
    }
    Ok(())
}

/// Turns off mouse capture before a panic message. The panic hook of
/// `ratatui::try_init` restores the rest of the terminal.
fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(stdout(), DisableMouseCapture);
        hook(info);
    }));
}
