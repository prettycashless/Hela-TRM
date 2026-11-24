use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::style::{Color, Style};
use ratatui::text::Span;

use crate::art::logo_lines;
use crate::boot::Boot;

pub fn draw_boot(frame: &mut ratatui::Frame, app: &super::App) {
    let size = frame.size();
    let reveal = app.boot.reveal();
    let progress = app.boot.progress();
    let log_lines = app.boot.log_lines();

    // Logo reveal
    let logo = logo_lines(reveal);
    let logo_para = Paragraph::new(logo)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(logo_para, size);

    // Progress bar
    let bar_width = size.width.saturating_sub(4) as f32;
    let bar_height = 1;
    let progress_width = (bar_width * progress) as u16;
    let progress_bar = Line::from(vec![
        Span::styled("[".into(), Style::default().fg(theme::MUTED)),
        Span::styled("=".repeat(progress_width as usize), Style::default().fg(theme::GOOD)),
        Span::styled(" ".repeat((bar_width - progress_width) as usize), Style::default().fg(theme::MUTED)),
        Span::styled("]", Style::default().fg(theme::MUTED)),
    ]);
    frame.render_widget(progress_bar, size.x / 2, size.height - 3, progress_width, 1);

    // Log lines
    let log_area = size.height - 5;
    for (i, (name, desc)) in crate::boot::SCRIPT.iter().enumerate().take(log_lines) {
        let line = Line::from(vec![
            Span::styled(format!("  [ ok ] {name}"), Style::default().fg(theme::GOOD)),
            Span::styled(format!(" {desc}"), Style::default().fg(theme::MUTED)),
        ]);
        frame.render_widget(line, size.x / 2, log_area.saturating_sub(i as u16), 1, 1);
    }

    // Skip hint
    let skip_hint = Line::from(vec![
        Span::styled("Press any key to skip"), Style::default().fg(theme::FAINT),
    ]);
    frame.render_widget(skip_hint, size.x / 2, size.height - 1, 1, 1);
}
