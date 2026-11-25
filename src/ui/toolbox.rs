use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

use super::{animated_rule, centered, ease};
use crate::app::{App, Session, Status, ToolSpec, TOOLS};
use crate::theme;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if area.width >= 60 && area.height >= 14 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(32.min(area.width.saturating_sub(38))),
                Constraint::Length(2),
                Constraint::Min(36),
            ])
            .split(area);

        draw_arsenal(frame, app, chunks[0]);
        draw_workspace(frame, app, chunks[2]);
    } else if area.width >= 44 && area.height >= 22 {
        let list_height = if area.height >= 26 { 10 } else { 7 };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(list_height),
                Constraint::Length(1),
                Constraint::Min(6),
            ])
            .split(area);

        draw_arsenal(frame, app, chunks[0]);
        draw_workspace(frame, app, chunks[2]);
    } else {
        fallback(frame, app, area);
    }
}

fn draw_arsenal(frame: &mut Frame, app: &App, area: Rect) {
    let enter = ease(app.toolbox_progress());
    let border = theme::blend(theme::BG, theme::FAINT, 0.4 + 0.6 * enter);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(Line::from(vec![
            Span::styled(
                " ⟡ ",
                Style::default().fg(theme::blend(border, theme::ACCENT, app.pulse())),
            ),
            Span::styled(
                "arsenal",
                Style::default()
                    .fg(theme::blend(theme::BG, theme::ACCENT_DIM, enter))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}/{} ", app.selected + 1, TOOLS.len()),
                Style::default().fg(theme::blend(theme::BG, theme::FAINT, enter)),
            ),
        ]))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let inner_width = inner.width as usize;
    let show_group = inner_width >= 30;

    let items: Vec<ListItem> = TOOLS
        .iter()
        .enumerate()
        .map(|(index, tool)| {
            let session = &app.sessions[index];
            let (glyph, glyph_color) = status_glyph(app, &session.status);
            let selected = index == app.selected;
            let fade = ease((app.toolbox_progress() * 1.8 - index as f32 * 0.035).clamp(0.0, 1.0));

            let mut name_style = Style::default().fg(theme::blend(theme::BG, theme::TEXT, fade));
            if selected {
                name_style = name_style.add_modifier(Modifier::BOLD);
            }

            let used = 2
                + tool.name.chars().count()
                + if show_group {
                    tool.group.chars().count() + 2
                } else {
                    0
                };
            let padding = inner_width.saturating_sub(used + 3).max(1);

            let mut spans = vec![
                Span::styled(format!("{glyph} "), Style::default().fg(glyph_color)),
                Span::styled(tool.name.to_string(), name_style),
                Span::raw(" ".repeat(padding)),
            ];

            if show_group {
                spans.push(Span::styled(
                    tool.group.to_ascii_uppercase(),
                    Style::default().fg(theme::blend(theme::BG, theme::FAINT, fade)),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let highlight = Style::default()
        .bg(theme::blend(
            theme::ACCENT_DIM,
            theme::ACCENT,
            0.15 + 0.35 * app.pulse() + 0.3 * app.selection_flash(),
        ))
        .add_modifier(Modifier::BOLD);

    let list = List::new(items)
        .highlight_symbol("❯ ")
        .highlight_style(highlight);

    let mut state = ListState::default();
    state.select(Some(app.selected));
    frame.render_stateful_widget(list, inner, &mut state);
}

fn draw_workspace(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let tool = &TOOLS[app.selected];
    let session = &app.sessions[app.selected];
    let enter = ease(app.toolbox_progress());

    let running = session.status == Status::Running;
    let border = if running {
        theme::blend(theme::ACCENT_DIM, theme::ACCENT, app.pulse())
    } else {
        theme::blend(
            theme::BG,
            theme::ACCENT_DIM,
            enter * 0.55 + app.selection_flash() * 0.45,
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(Line::from(vec![
            Span::styled(
                " // ",
                Style::default().fg(theme::blend(border, theme::ACCENT, app.pulse())),
            ),
            Span::styled(
                tool.name,
                Style::default()
                    .fg(theme::blend(theme::BG, theme::TEXT, enter))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::default()),
        ]))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let field_height = (tool.fields.len().max(1) as u16) + 2;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(field_height),
            Constraint::Length(1),
            Constraint::Min(3),
        ])
        .split(inner);

    if rows[0].height > 0 {
        let tagline = Line::from(vec![
            Span::styled(
                "⟡ ",
                Style::default().fg(theme::blend(
                    theme::ACCENT_DIM,
                    theme::ACCENT,
                    0.3 + 0.7 * app.pulse(),
                )),
            ),
            Span::styled(
                tool.tagline,
                Style::default()
                    .fg(theme::blend(theme::BG, theme::TEXT, enter))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("   {}", tool.group.to_ascii_uppercase()),
                Style::default().fg(theme::blend(theme::BG, theme::FAINT, enter)),
            ),
        ]);
        frame.render_widget(Paragraph::new(tagline), rows[0]);
    }

    if rows[1].height > 0 {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                tool.note,
                Style::default().fg(theme::blend(theme::BG, theme::MUTED, enter)),
            ))),
            rows[1],
        );
    }

    if rows[2].height > 0 && rows[2].width > 2 {
        frame.render_widget(
            Paragraph::new(animated_rule(rows[2].width, app.pulse())),
            rows[2],
        );
    }

    if rows[3].height > 0 {
        draw_fields(frame, app, session, tool, rows[3], enter);
    }

    if rows[4].height > 0 {
        frame.render_widget(Paragraph::new(status_line(app, session, tool)), rows[4]);
    }

    if rows[5].height > 0 && rows[5].width > 0 {
        draw_output(frame, app, session, rows[5], enter);
    }
}

fn draw_fields(
    frame: &mut Frame,
    app: &App,
    session: &Session,
    tool: &ToolSpec,
    area: Rect,
    enter: f32,
) {
    let focused_exists = !tool.fields.is_empty();
    let border = if focused_exists {
        theme::blend(theme::FAINT, theme::ACCENT_DIM, 0.25 + 0.45 * app.pulse())
    } else {
        theme::blend(theme::BG, theme::FAINT, enter)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(Span::styled(
            " parameters ",
            Style::default().fg(theme::blend(theme::BG, theme::ACCENT_DIM, enter)),
        ))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut lines = Vec::new();

    if tool.fields.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("no parameters", Style::default().fg(theme::MUTED)),
            Span::styled("  ·  press ENTER to run", Style::default().fg(theme::FAINT)),
        ]));
    } else {
        let available = (inner.width as usize).saturating_sub(14).max(4);
        let blink = app.frame % 16 < 10;

        for (index, field) in tool.fields.iter().enumerate() {
            let focused = index == session.focus;
            let raw = session.inputs.get(index).map(String::as_str).unwrap_or("");

            let (display, value_style) = if raw.is_empty() {
                (
                    field.placeholder.to_string(),
                    Style::default().fg(theme::FAINT),
                )
            } else {
                (
                    tail(raw, available),
                    Style::default().fg(if focused { theme::TEXT } else { theme::MUTED }),
                )
            };

            let marker_style = if focused {
                Style::default().fg(theme::blend(theme::ACCENT_DIM, theme::ACCENT, app.pulse()))
            } else {
                Style::default().fg(theme::FAINT)
            };

            let mut spans = vec![
                Span::styled(
                    format!(" {} ", if focused { "❯" } else { " " }),
                    marker_style,
                ),
                Span::styled(
                    format!("{:<9}", field.label.to_ascii_uppercase()),
                    Style::default().fg(if focused {
                        theme::ACCENT_DIM
                    } else {
                        theme::FAINT
                    }),
                ),
                Span::styled("▕", Style::default().fg(theme::FAINT)),
                Span::styled(display, value_style),
            ];

            if focused && blink {
                spans.push(Span::styled("▌", marker_style));
            }

            spans.push(Span::styled("▏", Style::default().fg(theme::FAINT)));
            lines.push(Line::from(spans));
        }
    }

    let visible = inner.height as usize;
    let lines: Vec<Line<'static>> = lines.into_iter().take(visible).collect();
    frame.render_widget(Paragraph::new(lines), inner);
}

fn status_line(app: &App, session: &Session, tool: &ToolSpec) -> Line<'static> {
    match &session.status {
        Status::Idle => Line::from(vec![
            Span::styled("○ ", Style::default().fg(theme::FAINT)),
            Span::styled("idle", Style::default().fg(theme::MUTED)),
            Span::styled(
                if tool.fields.is_empty() {
                    " · ENTER runs this tool"
                } else {
                    " · ENTER runs · TAB cycles fields"
                },
                Style::default().fg(theme::FAINT),
            ),
        ]),
        Status::Running => {
            let elapsed = session
                .started
                .map(|start| start.elapsed().as_secs_f32())
                .unwrap_or(0.0);
            Line::from(vec![
                Span::styled(
                    format!("{} ", app.spinner()),
                    Style::default().fg(theme::WARN),
                ),
                Span::styled(
                    format!("channel open {elapsed:.1}s"),
                    Style::default().fg(theme::WARN),
                ),
                Span::styled(" · output streams live", Style::default().fg(theme::FAINT)),
            ])
        }
        Status::Ok => Line::from(vec![
            Span::styled("✓ ", Style::default().fg(theme::GOOD)),
            Span::styled("complete", Style::default().fg(theme::GOOD)),
            Span::styled(" · ENTER reruns", Style::default().fg(theme::FAINT)),
        ]),
        Status::Failed(error) => Line::from(vec![
            Span::styled("✗ ", Style::default().fg(theme::BAD)),
            Span::styled(error.clone(), Style::default().fg(theme::BAD)),
            Span::styled(" · ENTER retries", Style::default().fg(theme::FAINT)),
        ]),
    }
}

fn draw_output(frame: &mut Frame, app: &App, session: &Session, area: Rect, enter: f32) {
    let total = session.out.len();
    let visible = area.height.saturating_sub(2) as usize;
    let maximum_scroll = total.saturating_sub(visible);
    let scroll = session.scroll.min(maximum_scroll);
    let scroll_label = if scroll > 0 {
        format!(" ↑{scroll} ")
    } else {
        String::new()
    };

    let border = theme::blend(theme::BG, theme::FAINT, 0.35 + 0.65 * enter);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(Line::from(vec![
            Span::styled(
                " output ",
                Style::default().fg(theme::blend(theme::BG, theme::ACCENT_DIM, enter)),
            ),
            Span::styled(
                format!("· {total} lines"),
                Style::default().fg(theme::blend(theme::BG, theme::FAINT, enter)),
            ),
            Span::styled(
                scroll_label,
                Style::default().fg(theme::blend(theme::BG, theme::WARN, enter)),
            ),
        ]))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let height = inner.height as usize;

    let lines: Vec<Line<'static>> = if total == 0 {
        empty_output(app, session)
    } else {
        let start = total.saturating_sub(height + scroll);
        let end = (start + height).min(total);

        session.out[start..end]
            .iter()
            .map(|line| Line::from(Span::styled(line.clone(), output_style(line))))
            .collect()
    };

    let paragraph = Paragraph::new(lines);
    let paragraph = if total == 0 {
        paragraph.wrap(Wrap { trim: false })
    } else {
        paragraph
    };
    frame.render_widget(paragraph, inner);
}

fn empty_output(app: &App, session: &Session) -> Vec<Line<'static>> {
    match &session.status {
        Status::Running => vec![Line::from(vec![
            Span::styled(
                format!("{} ", app.spinner()),
                Style::default().fg(theme::WARN),
            ),
            Span::styled(
                "channel open · awaiting output…",
                Style::default().fg(theme::MUTED),
            ),
        ])],
        Status::Failed(error) => vec![Line::from(Span::styled(
            format!("! {error}"),
            Style::default().fg(theme::BAD),
        ))],
        Status::Ok => vec![Line::from(Span::styled(
            "the ritual completed without output",
            Style::default().fg(theme::MUTED),
        ))],
        Status::Idle => vec![
            Line::from(Span::styled(
                "the gate is quiet.",
                Style::default().fg(theme::FAINT),
            )),
            Line::from(Span::styled(
                "press ENTER to run this tool.",
                Style::default().fg(theme::FAINT),
            )),
        ],
    }
}

fn output_style(line: &str) -> Style {
    if line.starts_with("⟡") {
        Style::default()
            .fg(theme::ACCENT)
            .add_modifier(Modifier::BOLD)
    } else if line.starts_with("  open") || line.starts_with("✓") || line.starts_with("  ✓") {
        Style::default().fg(theme::GOOD)
    } else if line.starts_with('✗')
        || line.starts_with("! ")
        || line.to_ascii_lowercase().contains("failed")
        || line.to_ascii_lowercase().contains("error")
    {
        Style::default().fg(theme::BAD)
    } else if line.starts_with("note")
        || line.starts_with("consent")
        || line.starts_with("  (")
        || line.starts_with('…')
    {
        Style::default().fg(theme::MUTED)
    } else {
        Style::default().fg(theme::TEXT)
    }
}

fn status_glyph(app: &App, status: &Status) -> (&'static str, Color) {
    match status {
        Status::Idle => ("·", theme::FAINT),
        Status::Running => (app.spinner(), theme::WARN),
        Status::Ok => ("✓", theme::GOOD),
        Status::Failed(_) => ("✗", theme::BAD),
    }
}

fn tail(text: &str, limit: usize) -> String {
    let count = text.chars().count();
    if count <= limit {
        text.to_string()
    } else {
        let skip = count.saturating_sub(limit.saturating_sub(1));
        let tail: String = text.chars().skip(skip).collect();
        format!("…{tail}")
    }
}

fn fallback(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(area);

    let tool = &TOOLS[app.selected];
    let session = &app.sessions[app.selected];
    let (glyph, color) = status_glyph(app, &session.status);

    if rows[0].height > 0 {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(" ⟡ ", Style::default().fg(theme::ACCENT)),
                    Span::styled(
                        "Hela arsenal",
                        Style::default()
                            .fg(theme::TEXT)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(format!(" {glyph} "), Style::default().fg(color)),
                    Span::styled(tool.name, Style::default().fg(theme::TEXT)),
                    Span::styled(" · resize for full view", Style::default().fg(theme::FAINT)),
                ]),
            ])
            .alignment(Alignment::Left),
            rows[0],
        );
    }

    if rows[1].height > 0 {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "resize for the full workspace",
                Style::default().fg(theme::MUTED),
            )))
            .alignment(Alignment::Center),
            centered(rows[1], rows[1].width.min(40), 1),
        );
    }
}
