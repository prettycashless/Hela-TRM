use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::style::{Color, Style};
use ratatui::text::Span;

use crate::art::{avatar_lines, logo_lines};
use crate::theme;

pub fn draw_home(frame: &mut ratatui::Frame, app: &super::App) {
    let size = frame.size();

    // Left column: facts
    let facts = vec![
        Line::from("OS: " + &app.sys.os),
        Line::from("Host: " + &app.sys.host),
        Line::from("IP: " + &app.sys.ip),
        Line::from("Uptime: " + &app.sys.uptime),
    ];
    let facts_para = Paragraph::new(facts)
        .block(Block::default().borders(Borders::RIGHT).style(Style::default().fg(theme::MUTED)))
        .style(Style::default().fg(theme::TEXT));
    frame.render_widget(facts_para, size);

    // Right column: logo + portrait
    let logo = logo_lines(0.5);
    let avatar = avatar_lines(app.scan_row());
    let combined: Vec<Line<'static>> = logo
        .into_iter()
        .zip(avatar.into_iter())
        .map(|(l, a)| Line::from(vec![l.spans, a.spans].concat())
        )
        .collect();
    let avatar_para = Paragraph::new(combined)
        .block(Block::default().borders(Borders::LEFT).style(Style::default().fg(theme::ACCENT)))
        .style(Style::default().fg(theme::TEXT));
    frame.render_widget(avatar_para, size);

    // Caption
    let caption = Line::from("H E L A · queen of hel");
    frame.render_widget(caption, size);
}
