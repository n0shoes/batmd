mod app;
mod editor;
mod highlight;
mod renderer;
mod ui;
mod watcher;

use std::io;
use std::path::PathBuf;

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;

use app::{App, AppResult};

#[derive(Parser)]
#[command(name = "batmd", about = "A terminal markdown viewer/editor")]
struct Cli {
    /// Markdown file to open
    file: PathBuf,
}

fn main() -> AppResult<()> {
    let cli = Cli::parse();

    if !cli.file.exists() {
        // Create the file if it doesn't exist
        std::fs::write(&cli.file, "")?;
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let mut app = App::new(cli.file)?;
    let result = app.run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
