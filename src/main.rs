mod app;
mod art;
mod art_data;
mod boot;
mod system;
mod theme;
mod tools;
mod ui;

use std::io::{self, stdout};
use std::time::Duration;

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::{
    cursor,
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::Terminal;

use app::App;

type Tui = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> io::Result<()> {
    install_panic_hook();

    let mut terminal = match start_tui() {
        Ok(terminal) => terminal,
        Err(error) => {
            let _ = restore();
            return Err(error);
        }
    };

    let mut app = App::new();
    let run_result = run(&mut terminal, &mut app);
    let restore_result = restore_terminal(&mut terminal);

    println!("\n  hela closed the gate. farewell.\n");

    run_result?;
    restore_result
}

fn start_tui() -> io::Result<Tui> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore() -> io::Result<()> {
    let mut result = disable_raw_mode();
    if let Err(error) = execute!(stdout(), LeaveAlternateScreen, cursor::Show) {
        if result.is_ok() {
            result = Err(error);
        }
    }
    result
}

fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    let mut result = disable_raw_mode();
    if let Err(error) = execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show) {
        if result.is_ok() {
            result = Err(error);
        }
    }
    if let Err(error) = terminal.show_cursor() {
        if result.is_ok() {
            result = Err(error);
        }
    }
    result
}

fn run(terminal: &mut Tui, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        app.tick();
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key)
                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
                {
                    app.on_key(key);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn install_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, cursor::Show);
        hook(info);
    }));
}
