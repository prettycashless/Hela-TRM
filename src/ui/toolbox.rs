use ratatui::widgets::{Block, Borders, Paragraph, Table, Row, TableState};
use ratatui::style::{Color, Style};
use ratatui::text::Span;

use crate::tools::TOOLS;

pub fn draw_toolbox(frame: &mut ratatui::Frame, app: &super::App) {
    let size = frame.size();

    // Sidebar: groups + tools
    let items: Vec<String> = TOOLS
        .iter()
        .flat_map(|group| {
            group.tools.iter().map(|t| format!("{}", t.name))
        })
        .collect();
    let items_len = items.len();
    let mut rows = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let mut spans = vec![Span::styled(item, Style::default())];
        if i == app.selected {
            spans[0] = Span::styled(
                item,
                Style::default().bg(theme::ACCENT).fg(theme::BG),
            );
        }
        rows.push(Row::new().cells(spans));
    }
    let mut table_state = TableState::default().selected(Some(app.selected));
    let table = Table::new(rows)
        .header(Row::new().cells([Span::styled("Tools", Style::default().fg(theme::TEXT))]))
        .highlight_style(Style::default().bg(theme::ACCENT).fg(theme::BG));
    frame.render_stateful_widget(table, size, &mut table_state);

    // Panel: tagline/note/fields/enter-hint/output
    let selected = &app.sessions[app.selected];
    let tool = &TOOLS[app.selected];

    // Tagline
    let tagline = Line::from(vec![
        Span::styled("Hela " + &tool.tagline, Style::default().fg(theme::ACCENT)),
    ]);

    // Note
    let note = Line::from(vec![
        Span::styled(&tool.note, Style::default().fg(theme::FAINT)),
    ]);

    // Enter hint
    let enter_hint = Line::from(vec![
        Span::styled("Enter: run", Style::default().fg(theme::MUTED)),
        Span::styled("  Tab: cycle", Style::default().fg(theme::MUTED)),
        Span::styled("  Esc: back", Style::default().fg(theme::MUTED)),
    ]);

    // Output area (styled output lines)
    let output_lines: Vec<Line<'static>> = selected
        .out
        .iter()
        .map(|line| {
            let prefix = match selected.status {
                super::Status::Idle => "",
                super::Status::Running => "[running] ",
                super::Status::Ok => "[ ok ] ",
                super::Status::Failed(e) => format!("! {e} "),
            };
            let trimmed = if line.len() > 60 { line.chars().take(60).collect::<String>() + "…" } else { line.clone() };
            Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme::FAINT)),
                Span::styled(trimmed, Style::default().fg(theme::TEXT)),
            ])
        })
        .collect();

    let output = Paragraph::new(output_lines)
        .block(Block::default().borders(Borders::ALL).title(" output "));
    frame.render_widget(output, size);

    // Tagline + note + enter hint stacked
    let y = size.height - 7;
    frame.render_widget(tagline, size.x/2, 0, 1, 1);
    frame.render_widget(note, size.x/2, 1, 1, 1);
    frame.render_widget(enter_hint, size.x/2, 2, 1, 1);
}
