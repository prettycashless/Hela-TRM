use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use super::{animated_rule, centered, ease};
use crate::app::App;
use crate::art::logo_lines;
use crate::boot::SCRIPT;
use crate::theme;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if area.width < 52 || area.height < 19 {
        compact(frame, app, area);
        return;
    }

    let card = centered(area, area.width.min(76), area.height.min(19));
    let elapsed = app.boot.elapsed();
    let enter = ease((elapsed / 0.5).clamp(0.0, 1.0));
    let pulse = app.pulse();

    let border_color = theme::blend(theme::BG, theme::ACCENT_DIM, 0.12 + 0.7 * enter);
    let title_color = theme::blend(theme::BG, theme::ACCENT, enter);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(vec![
            Span::styled(
                " ⟡ ",
                Style::default().fg(theme::blend(border_color, theme::ACCENT, pulse)),
            ),
            Span::styled(
                "hela // boot sequence",
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .style(Style::default().bg(theme::BG).fg(theme::TEXT));

    let inner = block.inner(card);
    frame.render_widget(block, card);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(6),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    if rows[0].height > 0 {
        let logo = logo_lines(app.boot.reveal(), pulse);
        frame.render_widget(Paragraph::new(logo).alignment(Alignment::Center), rows[0]);
    }

    if rows[1].height > 0 {
        let mut spans = vec![
            Span::styled(
                "queen of the gate",
                Style::default().fg(theme::blend(theme::BG, theme::MUTED, enter)),
            ),
            Span::styled(
                "  ·  ",
                Style::default().fg(theme::blend(theme::BG, theme::FAINT, enter)),
            ),
            Span::styled(
                "terminal arsenal",
                Style::default().fg(theme::blend(theme::BG, theme::ACCENT_DIM, enter)),
            ),
        ];

        if app.boot.reveal() >= 1.0 && app.frame % 30 < 15 {
            spans.push(Span::styled(
                "  ✦",
                Style::default().fg(theme::blend(theme::ACCENT_DIM, theme::TEXT, pulse)),
            ));
        }

        frame.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            rows[1],
        );
    }

    if rows[2].height > 0 && rows[2].width >= 12 {
        let rule_width = rows[2].width.min(52);
        let rule_area = centered(rows[2], rule_width, 1);
        frame.render_widget(Paragraph::new(animated_rule(rule_width, pulse)), rule_area);
    }

    if rows[3].height > 0 {
        frame.render_widget(
            Paragraph::new(progress_line(app, rows[3].width)).alignment(Alignment::Center),
            rows[3],
        );
    }

    if rows[4].height > 0 {
        let logs_area = centered(rows[4], rows[4].width.min(58), rows[4].height.min(6));
        frame.render_widget(Paragraph::new(log_lines(app, enter)), logs_area);
    }

    if rows[6].height > 0 {
        let hint_color = theme::blend(theme::BG, theme::MUTED, 0.35 + 0.65 * pulse);
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                "press any key to skip",
                Style::default().fg(hint_color),
            )]))
            .alignment(Alignment::Center),
            rows[6],
        );
    }
}

fn progress_line(app: &App, width: u16) -> Line<'static> {
    let progress = app.boot.progress();
    let bar_width = (width as usize).saturating_sub(8).clamp(8, 46);
    let filled = ((progress * bar_width as f32).round() as usize).min(bar_width);

    let mut spans = Vec::with_capacity(bar_width + 4);
    spans.push(Span::styled(
        "⟦",
        Style::default().fg(theme::blend(theme::FAINT, theme::ACCENT_DIM, progress)),
    ));

    for index in 0..bar_width {
        if index < filled {
            let color = theme::blend(
                theme::ACCENT_DIM,
                theme::ACCENT,
                0.25 + 0.75 * (index as f32 / bar_width as f32),
            );
            spans.push(Span::styled("█", Style::default().fg(color)));
        } else {
            spans.push(Span::styled("░", Style::default().fg(theme::FAINT)));
        }
    }

    spans.push(Span::styled(
        "⟧",
        Style::default().fg(theme::blend(theme::FAINT, theme::ACCENT_DIM, progress)),
    ));
    spans.push(Span::styled(
        format!(" {:>3.0}%", progress * 100.0),
        Style::default().fg(theme::MUTED),
    ));

    Line::from(spans)
}

fn log_lines(app: &App, enter: f32) -> Vec<Line<'static>> {
    let count = app.boot.log_lines();
    let mut lines = Vec::new();

    for (name, description) in SCRIPT.iter().take(count) {
        let final_line = *name == "hela";
        let name_style = if final_line {
            Style::default()
                .fg(theme::GOOD)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::blend(theme::BG, theme::GOOD, enter * 0.9))
        };
        let description_style = if final_line {
            Style::default().fg(theme::ACCENT)
        } else {
            Style::default().fg(theme::blend(theme::BG, theme::MUTED, enter))
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  ✓ {name:<7}"), name_style),
            Span::styled(format!(" {description}"), description_style),
        ]));
    }

    if count < SCRIPT.len() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", app.spinner()),
                Style::default().fg(theme::ACCENT_DIM),
            ),
            Span::styled("listening…", Style::default().fg(theme::FAINT)),
        ]));
    }

    lines
}

fn compact(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let logo = logo_lines(app.boot.reveal(), app.pulse());
    let logo_area = centered(rows[0], rows[0].width, rows[0].height.min(6));
    frame.render_widget(Paragraph::new(logo).alignment(Alignment::Center), logo_area);

    if rows[1].height > 0 {
        let percent = app.boot.progress() * 100.0;
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                format!("boot {percent:.0}%  ·  any key skips"),
                Style::default().fg(theme::MUTED),
            )]))
            .alignment(Alignment::Center),
            rows[1],
        );
    }
}
