use ratatui::widgets::{Block, Borders, TableCell, TableRow, Table};
use ratatui::style::{Color, Style};
use ratatui::text::Span;

pub fn draw(frame: &mut ratatui::Frame, app: &super::App) {
    match app.screen {
        super::Screen::Boot => boot::draw_boot(frame, app),
        super::Screen::Home => home::draw_home(frame, app),
        super::Screen::Toolbox => toolbox::draw_toolbox(frame, app),
    }
}

pub fn header(app: &super::App) -> Line<'static> {
    let title = "H E L A · terminal arsenal";
    let version = env!("CARGO_PKG_VERSION");
    Line::from(vec![
        Span::styled(title, Style::default().fg(ratatui::style::Color::Rgb(132, 216, 226))),
        Span::styled(format!(" v{version}"), Style::default().fg(ratatui::style::Color::MUTED)),
    ])
}

pub fn footer(app: &super::App) -> Line<'static> {
    Line::from(vec![Span::styled(
        "Press Esc to exit, Ctrl+Q to force-quit, Tab to cycle",
        Style::default().fg(ratatui::style::Color::MUTATED),
    )])
}

pub fn hint(app: &super::App) -> Line<'static> {
    let selected = &app.sessions[app.selected];
    let tool = &TOOLS[app.selected];
    Line::from(vec![
        Span::styled("❯ ", Style::default().fg(ratatui::style::Color::ACCENT)),
        Span::styled(tool.name.clone(), Style::default().fg(ratatui::style::Color::TEXT)),
    ])
}

pub fn rule() -> Line<'static> {
    Line::from("─".to_string())
}
