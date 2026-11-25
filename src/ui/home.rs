use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use super::{animated_rule, centered, ease, panel};
use crate::app::App;
use crate::art::{avatar_compact_lines, avatar_lines, logo_lines, COMPACT_H, COMPACT_W};
use crate::art_data::{AVATAR_H, AVATAR_W};
use crate::system::truncate;
use crate::theme;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if area.width < 34 || area.height < 12 {
        fallback(frame, app, area);
        return;
    }

    let progress = app.home_progress();

    if area.width >= 68 && area.height >= 16 {
        let left_width = 30.min(area.width.saturating_sub(36));
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(left_width),
                Constraint::Length(2),
                Constraint::Min(24),
            ])
            .split(area);

        draw_identity(frame, app, chunks[0], progress);
        draw_portrait(frame, app, chunks[2], progress);
    } else {
        draw_identity(frame, app, area, progress);
    }
}

fn draw_identity(frame: &mut Frame, app: &App, area: Rect, progress: f32) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .split(area);

    if rows[0].height > 0 && rows[0].width > 0 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(2),
                Constraint::Min(8),
            ])
            .split(rows[0]);

        let reveal = ease((progress / 0.7).clamp(0.0, 1.0));
        frame.render_widget(Paragraph::new(logo_lines(reveal, app.pulse())), columns[0]);

        let title_color = theme::blend(theme::BG, theme::TEXT, ease(progress));
        let accent_color = theme::blend(theme::BG, theme::ACCENT, ease(progress));
        let muted_color = theme::blend(theme::BG, theme::MUTED, ease(progress));

        let identity = vec![
            Line::from(Span::styled(
                "H E L A",
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "queen of the gate",
                Style::default().fg(accent_color),
            )),
            Line::from(Span::styled(
                "terminal arsenal",
                Style::default().fg(muted_color),
            )),
        ];

        let text_area = centered(columns[2], columns[2].width, 3);
        frame.render_widget(Paragraph::new(identity), text_area);
    }

    if rows[1].height > 0 && rows[1].width > 2 {
        frame.render_widget(
            Paragraph::new(animated_rule(rows[1].width, app.pulse())),
            rows[1],
        );
    }

    if rows[2].height > 0 {
        let block = panel("operator");
        let inner = block.inner(rows[2]);
        frame.render_widget(block, rows[2]);

        if inner.height > 0 && inner.width > 0 {
            let facts = fact_lines(app, inner.width, progress);
            let visible = inner.height as usize;
            let facts: Vec<Line<'static>> = facts.into_iter().take(visible).collect();
            frame.render_widget(Paragraph::new(facts), inner);
        }
    }

    if rows[3].height > 0 {
        let blink = app.frame % 40 < 26;
        let cursor = if blink { "❯ " } else { "  " };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    cursor,
                    Style::default().fg(theme::blend(
                        theme::ACCENT_DIM,
                        theme::ACCENT,
                        app.pulse(),
                    )),
                ),
                Span::styled("the gate is open", Style::default().fg(theme::FAINT)),
            ])),
            rows[3],
        );
    }
}

fn fact_lines(app: &App, width: u16, progress: f32) -> Vec<Line<'static>> {
    let facts: [(&str, &str); 10] = [
        ("operator", app.sys.user.as_str()),
        ("host", app.sys.host.as_str()),
        ("os", app.sys.os.as_str()),
        ("kernel", app.sys.kernel.as_str()),
        ("arch", app.sys.arch.as_str()),
        ("cpu", app.sys.cpu.as_str()),
        ("memory", app.sys.memory.as_str()),
        ("uptime", app.sys.uptime.as_str()),
        ("ip", app.sys.ip.as_str()),
        ("shell", app.sys.shell.as_str()),
    ];

    let value_width = (width as usize).saturating_sub(10).max(4);
    let fade = ease((progress / 0.85).clamp(0.0, 1.0));

    facts
        .iter()
        .enumerate()
        .map(|(index, (label, value))| {
            let step = (fade * 1.4 - index as f32 * 0.045).clamp(0.0, 1.0);
            let label_style = Style::default().fg(theme::blend(theme::BG, theme::MUTED, step));
            let value_style = Style::default().fg(theme::blend(theme::BG, theme::TEXT, step));
            Line::from(vec![
                Span::styled(format!("{:<9}", label.to_ascii_uppercase()), label_style),
                Span::styled(truncate(value, value_width), value_style),
            ])
        })
        .collect()
}

#[derive(Clone, Copy)]
enum Portrait {
    Full,
    Compact,
}

fn draw_portrait(frame: &mut Frame, app: &App, area: Rect, progress: f32) {
    if area.width < 16 || area.height < 8 {
        return;
    }

    let inner_width = area.width.saturating_sub(2);
    let inner_height = area.height.saturating_sub(2);

    let mode = if inner_width >= AVATAR_W as u16 && inner_height >= AVATAR_H as u16 {
        Portrait::Full
    } else if inner_width >= COMPACT_W as u16 && inner_height >= COMPACT_H as u16 {
        Portrait::Compact
    } else {
        return;
    };

    let enter = ease((progress / 0.8).clamp(0.0, 1.0));
    let border_color = theme::blend(
        theme::BG,
        theme::ACCENT_DIM,
        0.15 + 0.65 * enter + 0.1 * app.pulse(),
    );
    let title_color = theme::blend(theme::BG, theme::ACCENT, enter);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(vec![
            Span::styled(
                " ⟡ ",
                Style::default().fg(theme::blend(border_color, theme::ACCENT, app.pulse())),
            ),
            Span::styled(
                match mode {
                    Portrait::Full => "hela // portrait",
                    Portrait::Compact => "hela // sigil",
                },
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let reveal = ease(((progress - 0.08) / 0.85).clamp(0.0, 1.0));
    if reveal <= 0.0 {
        return;
    }

    let total = match mode {
        Portrait::Full => AVATAR_H,
        Portrait::Compact => COMPACT_H,
    };
    let visible = ((reveal * total as f32).ceil() as usize).min(total);
    if visible == 0 {
        return;
    }

    let scan = Some(if visible > 0 { app.scan_row(total) } else { 0 });

    let lines = match mode {
        Portrait::Full => avatar_lines(scan, app.pulse()),
        Portrait::Compact => avatar_compact_lines(scan, app.pulse()),
    };

    let draw_height = (visible as u16).min(inner.height);
    if draw_height == 0 {
        return;
    }

    let offset = if visible > draw_height as usize {
        (visible - draw_height as usize) / 2
    } else {
        0
    };

    let shown: Vec<Line<'static>> = lines
        .into_iter()
        .skip(offset)
        .take(draw_height as usize)
        .collect();

    let top = inner.y + (inner.height - draw_height) / 2;
    let target = Rect::new(inner.x, top, inner.width, draw_height);
    frame.render_widget(Paragraph::new(shown).alignment(Alignment::Center), target);
}

fn fallback(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(1)])
        .split(area);

    if rows[0].height > 0 {
        frame.render_widget(
            Paragraph::new(logo_lines(app.boot.reveal().max(1.0), app.pulse()))
                .alignment(Alignment::Center),
            rows[0],
        );
    }

    if rows[1].height > 0 {
        let summary = format!("{}@{} · {}", app.sys.user, app.sys.host, app.sys.os);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "H E L A · queen of the gate",
                    Style::default().fg(theme::ACCENT),
                )),
                Line::from(Span::styled(
                    truncate(&summary, area.width as usize),
                    Style::default().fg(theme::MUTED),
                )),
            ])
            .alignment(Alignment::Center),
            rows[1],
        );
    }
}
