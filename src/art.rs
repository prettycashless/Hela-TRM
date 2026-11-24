use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::art_data::{AVATAR, AVATAR_W};
use crate::theme;

/// Custom "H" wordmark.
pub const LOGO_H: [&str; 6] = [
    "██╗  ██╗",
    "██║  ██║",
    "███████╗",
    "██╔══██║",
    "██║  ██║",
    "╚═╝  ╚═╝",
];

pub const LOGO_W: usize = 8;

fn level(c: char) -> usize {
    match c {
        '0'..='9' => c as usize - '0' as usize,
        'a'..='o' => 10 + (c as usize - 'a' as usize),
        _ => 0,
    }
}

/// The wordmark, toned by a 0..=1 breathing phase.
pub fn logo_lines(phase: f32) -> Vec<Line<'static>> {
    let phase = phase.clamp(0.0, 1.0);
    let top = blend(theme::ACCENT, theme::TEXT, 0.35 + 0.45 * phase);
    let bar = blend(theme::ACCENT, theme::TEXT, 0.55 + 0.45 * phase);
    let bottom = blend(theme::ACCENT_DIM, theme::ACCENT, 0.30 + 0.50 * phase);

    LOGO_H
        .iter()
        .enumerate()
        .map(|(row, line)| {
            let color = match row {
                0 | 1 => top,
                2 => bar,
                _ => bottom,
            };
            let mut spans = vec![Span::styled((*line).to_string(), Style::default().fg(color))];
            if row == 2 {
                spans.push(Span::styled(
                    "  ✦",
                    Style::default().fg(blend(theme::ACCENT, theme::TEXT, phase)),
                ));
            }
            Line::from(spans)
        })
        .collect()
}

/// Hela's portrait, optionally with a scan line travelling down it.
pub fn avatar_lines(scan: Option<usize>) -> Vec<Line<'static>> {
    AVATAR
        .iter()
        .enumerate()
        .map(|(row, line)| {
            let mut spans: Vec<Span<'static>> = Vec::with_capacity(AVATAR_W);
            let on_scan = scan == Some(row);
            for ch in line.chars() {
                let mut idx = level(ch);
                if on_scan && idx > 0 {
                    idx = (idx + 1).min(4);
                }
                let top = idx / 5;
                let bottom = idx % 5;

                let span = if top == 0 && bottom == 0 {
                    Span::raw(" ")
                } else if top == bottom {
                    Span::styled("█", Style::default().fg(theme::TONES[top]))
                } else if top == 0 {
                    Span::styled("▄", Style::default().fg(theme::TONES[bottom]))
                } else if bottom == 0 {
                    Span::styled("▀", Style::default().fg(theme::TONES[top]))
                } else {
                    Span::styled(
                        "▀",
                        Style::default().fg(theme::TONES[top]).bg(theme::TONES[bottom]),
                    )
                };
                spans.push(span);
            }
            Line::from(spans)
        })
        .collect()
}

fn blend(a: ratatui::style::Color, b: ratatui::style::Color, t: f32) -> ratatui::style::Color {
    let (ar, ag, ab) = rgb(a);
    let (br, bg, bb) = rgb(b);
    let mix = |x: u8, y: u8| -> u8 {
        (x as f32 + (y as f32 - x as f32) * t.clamp(0.0, 1.0)).round() as u8
    };
    ratatui::style::Color::Rgb(mix(ar, br), mix(ag, bg), mix(ab, bb))
}

fn rgb(c: ratatui::style::Color) -> (u8, u8, u8) {
    match c {
        ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255),
    }
}
