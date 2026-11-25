mod boot;
mod home;
mod toolbox;

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Screen};
use crate::theme;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(theme::BG)), area);

    if area.height == 0 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header(frame, app, rows[0]);

    match app.screen {
        Screen::Boot => boot::draw(frame, app, rows[1]),
        Screen::Home => home::draw(frame, app, rows[1]),
        Screen::Toolbox => toolbox::draw(frame, app, rows[1]),
    }

    draw_footer(frame, app, rows[2]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let area = sides(area, 1);
    if area.width == 0 || area.height == 0 {
        return;
    }

    let pulse = app.pulse();
    let sigil = theme::blend(theme::ACCENT_DIM, theme::ACCENT, 0.3 + 0.7 * pulse);
    let left = Line::from(vec![
        Span::styled(
            " ◈ ",
            Style::default().fg(sigil).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "H E L A",
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  terminal arsenal", Style::default().fg(theme::MUTED)),
    ]);

    if area.width >= 64 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(28), Constraint::Length(24)])
            .split(area);

        frame.render_widget(Paragraph::new(left), chunks[0]);

        let (glyph, label, color) = if app.is_running() {
            (app.spinner(), "channel open", theme::WARN)
        } else {
            ("●", "gate open", theme::GOOD)
        };

        let right = Line::from(vec![
            Span::styled(format!("{glyph} "), Style::default().fg(color)),
            Span::styled(format!("{label}  "), Style::default().fg(theme::MUTED)),
            Span::styled(
                format!("v{} ", env!("CARGO_PKG_VERSION")),
                Style::default().fg(theme::FAINT),
            ),
        ]);
        frame.render_widget(Paragraph::new(right).alignment(Alignment::Right), chunks[1]);
    } else if area.width >= 28 {
        frame.render_widget(Paragraph::new(left), area);
    } else {
        let short = Line::from(vec![
            Span::styled(
                " ◈ ",
                Style::default().fg(sigil).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if area.width >= 12 { "H E L A" } else { "HELA" },
                Style::default()
                    .fg(theme::TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(short), area);
    }
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let area = sides(area, 1);
    if area.width == 0 || area.height == 0 {
        return;
    }

    let hints: &[(&str, &str)] = match app.screen {
        Screen::Boot => &[],
        Screen::Home => {
            if area.width >= 27 {
                &[("ENTER", "open arsenal"), ("Q", "quit")]
            } else if area.width >= 19 {
                &[("ENTER", "open"), ("Q", "quit")]
            } else {
                &[("ENTER", "open")]
            }
        }
        Screen::Toolbox => {
            if area.width >= 75 {
                &[
                    ("↑↓", "tool"),
                    ("TAB", "field"),
                    ("ENTER", "run"),
                    ("PgUp/PgDn", "scroll"),
                    ("ESC", "back"),
                    ("CTRL+Q", "quit"),
                ]
            } else if area.width >= 52 {
                &[
                    ("↑↓", "tool"),
                    ("TAB", "field"),
                    ("ENTER", "run"),
                    ("ESC", "back"),
                    ("^Q", "quit"),
                ]
            } else if area.width >= 34 {
                &[("↑↓", "tool"), ("ENTER", "run"), ("ESC", "back")]
            } else {
                &[("ESC", "back")]
            }
        }
    };

    let mut spans = Vec::new();
    for (index, (key, description)) in hints.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", Style::default().fg(theme::FAINT)));
        }
        spans.push(Span::styled(*key, Style::default().fg(theme::ACCENT_DIM)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            *description,
            Style::default().fg(theme::MUTED),
        ));
    }

    if spans.is_empty() {
        return;
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

pub(crate) fn sides(area: Rect, margin: u16) -> Rect {
    let width = area.width.saturating_sub(margin.saturating_mul(2));
    if width == 0 {
        area
    } else {
        Rect::new(area.x + margin, area.y, width, area.height)
    }
}

pub(crate) fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}

pub(crate) fn ease(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    1.0 - (1.0 - value).powi(3)
}

pub(crate) fn panel(title: impl Into<String>) -> Block<'static> {
    let title = title.into();
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::FAINT))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(theme::ACCENT_DIM),
        ))
        .style(Style::default().bg(theme::BG))
}

pub(crate) fn animated_rule(width: u16, phase: f32) -> Line<'static> {
    let width = width as usize;
    if width == 0 {
        return Line::from("");
    }

    let position = ((phase.clamp(0.0, 1.0) * width as f32) as usize).min(width - 1);
    let left = "─".repeat(position);
    let right = "─".repeat(width - position - 1);

    Line::from(vec![
        Span::styled(left, Style::default().fg(theme::FAINT)),
        Span::styled("◆", Style::default().fg(theme::ACCENT)),
        Span::styled(right, Style::default().fg(theme::FAINT)),
    ])
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::draw;
    use crate::app::{App, Screen, Status};

    fn render(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal.draw(|frame| draw(frame, app)).expect("draw");
        let buffer = terminal.backend().buffer();

        let mut output = String::with_capacity((width as usize + 1) * height as usize);
        for row in 0..height {
            for column in 0..width {
                output.push_str(buffer[(column, row)].symbol());
            }
            output.push('\n');
        }
        output
    }

    fn prepare() -> App {
        let mut app = App::new();
        app.boot.skip();
        app.home_entered = Instant::now() - Duration::from_secs(2);
        app.toolbox_entered = Instant::now() - Duration::from_secs(2);
        app.selection_changed = Instant::now() - Duration::from_secs(2);
        app
    }

    #[test]
    fn screens_render_at_common_sizes() {
        let mut app = prepare();

        for (width, height) in [
            (20, 5),
            (40, 10),
            (44, 12),
            (50, 24),
            (60, 14),
            (60, 16),
            (80, 24),
            (100, 34),
            (140, 44),
        ] {
            for screen in [Screen::Boot, Screen::Home, Screen::Toolbox] {
                app.screen = screen;
                let frame = render(&app, width, height);
                assert_eq!(
                    frame.lines().count(),
                    height as usize,
                    "{width}x{height} {screen:?}"
                );
            }
        }
    }

    #[test]
    fn home_and_toolbox_show_identity() {
        let mut app = prepare();

        app.screen = Screen::Home;
        let home = render(&app, 100, 34);
        assert!(home.contains("queen of the gate"));
        assert!(home.contains("operator"));
        assert!(home.contains("portrait"));

        app.screen = Screen::Toolbox;
        app.selected = 0;
        app.sessions[0].status = Status::Ok;
        app.sessions[0].out = vec!["⟡ hash forge".into(), "sha256   ...".into()];
        let toolbox = render(&app, 100, 34);
        assert!(toolbox.contains("arsenal"));
        assert!(toolbox.contains("hash-forge"));
        assert!(toolbox.contains("parameters"));
        assert!(toolbox.contains("output"));

        let compact = render(&app, 80, 24);
        assert!(compact.contains("CTRL+Q quit"));
    }

    #[test]
    #[ignore = "manual visual preview"]
    fn preview_frames() {
        let mut app = prepare();

        app.screen = Screen::Home;
        println!("\n=== HOME 100x34 ===\n{}", render(&app, 100, 34));

        app.boot.seek(2.4);
        app.screen = Screen::Boot;
        println!("\n=== BOOT 100x34 ===\n{}", render(&app, 100, 34));

        app.screen = Screen::Toolbox;
        app.sessions[0].status = Status::Ok;
        app.sessions[0].out = vec![
            "⟡ hash forge".into(),
            "input    the gate is open".into(),
            "md5      7c4f...".into(),
            String::new(),
            "sha256   1b2d...".into(),
        ];
        println!("\n=== TOOLBOX 100x34 ===\n{}", render(&app, 100, 34));

        app.screen = Screen::Home;
        println!("\n=== HOME 80x24 ===\n{}", render(&app, 80, 24));

        app.screen = Screen::Toolbox;
        println!("\n=== TOOLBOX 80x24 ===\n{}", render(&app, 80, 24));

        app.selected = 7;
        app.sessions[7].status = Status::Idle;
        app.sessions[7].out.clear();
        println!(
            "\n=== TOOLBOX PORT-SCAN 80x24 ===\n{}",
            render(&app, 80, 24)
        );

        app.selected = 5;
        println!("\n=== TOOLBOX WORDLIST 80x24 ===\n{}", render(&app, 80, 24));

        app.selected = 7;
        println!(
            "\n=== TOOLBOX PORT-SCAN 60x16 ===\n{}",
            render(&app, 60, 16)
        );

        println!("\n=== TOOLBOX WORDLIST 50x24 ===\n{}", render(&app, 50, 24));

        app.screen = Screen::Home;
        println!("\n=== HOME 80x18 ===\n{}", render(&app, 80, 18));
        app.screen = Screen::Toolbox;

        app.screen = Screen::Home;
        println!("\n=== HOME 60x15 ===\n{}", render(&app, 60, 15));
        println!("\n=== HOME 44x10 ===\n{}", render(&app, 44, 10));

        app.boot.seek(1.8);
        app.screen = Screen::Boot;
        println!("\n=== BOOT 80x18 ===\n{}", render(&app, 80, 18));

        app.screen = Screen::Toolbox;
        println!("\n=== TOOLBOX 40x8 ===\n{}", render(&app, 40, 8));
    }
}
