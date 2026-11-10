mod app;
mod character;
mod ui;

use std::io;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use app::{App, Phase};

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();

    match result? {
        Some(summary) => print!("{summary}"),
        None => println!("No character created."),
    }

    Ok(())
}

fn run(mut terminal: DefaultTerminal) -> io::Result<Option<String>> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => app.handle_key(key),
            _ => {}
        }

        if app.should_quit || app.phase == Phase::Finished {
            break;
        }
    }

    Ok(app.accepted_summary())
}
