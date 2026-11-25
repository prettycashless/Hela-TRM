use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::art_data::{AVATAR, AVATAR_H, AVATAR_W};
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
pub const COMPACT_W: usize = AVATAR_W.div_ceil(2);
pub const COMPACT_H: usize = AVATAR_H.div_ceil(2);

/// The wordmark, revealed left-to-right, toned by a breathing glow.
pub fn logo_lines(reveal: f32, glow: f32) -> Vec<Line<'static>> {
    let reveal = reveal.clamp(0.0, 1.0);
    let glow = glow.clamp(0.0, 1.0);

    LOGO_H
        .iter()
        .enumerate()
        .map(|(row, line)| {
            let base = match row {
                0 | 1 => theme::ACCENT,
                2 => theme::TEXT,
                _ => theme::ACCENT_DIM,
            };
            let color = theme::blend(base, theme::TEXT, 0.15 + glow * 0.55);
            let characters: Vec<char> = line.chars().collect();
            debug_assert_eq!(characters.len(), LOGO_W);
            let width = LOGO_W;
            let row_reveal = ((reveal * 1.25 - row as f32 * 0.06).clamp(0.0, 1.0) * width as f32)
                .ceil() as usize;

            let mut spans = Vec::with_capacity(width);
            for (index, character) in characters.iter().enumerate() {
                if index < row_reveal {
                    let is_head = index + 1 == row_reveal && row_reveal < width;
                    let glyph = if is_head {
                        theme::blend(color, theme::TEXT, 0.75)
                    } else {
                        color
                    };
                    spans.push(Span::styled(
                        character.to_string(),
                        Style::default().fg(glyph),
                    ));
                } else {
                    spans.push(Span::raw(" "));
                }
            }

            Line::from(spans)
        })
        .collect()
}

/// The portrait at full resolution, optionally with a travelling scan line.
pub fn avatar_lines(scan: Option<usize>, glow: f32) -> Vec<Line<'static>> {
    let rows = AVATAR
        .iter()
        .enumerate()
        .map(|(row, line)| (row, line.chars().collect::<Vec<char>>()))
        .collect();

    render_avatar(rows, AVATAR_W, scan, glow)
}

/// A compact portrait for narrow terminals.
pub fn avatar_compact_lines(scan: Option<usize>, glow: f32) -> Vec<Line<'static>> {
    let rows = AVATAR
        .iter()
        .step_by(2)
        .enumerate()
        .map(|(row, line)| (row, line.chars().step_by(2).collect::<Vec<char>>()))
        .collect();

    render_avatar(rows, COMPACT_W, scan, glow)
}

fn render_avatar(
    rows: Vec<(usize, Vec<char>)>,
    width: usize,
    scan: Option<usize>,
    glow: f32,
) -> Vec<Line<'static>> {
    let glow = glow.clamp(0.0, 1.0);

    rows.into_iter()
        .map(|(row, characters)| {
            let on_scan = scan == Some(row);
            let mut spans = Vec::with_capacity(width);

            for index in 0..width {
                let mut level = characters.get(index).map(|c| level(*c)).unwrap_or(0);

                if on_scan && level > 0 {
                    level = (level + 2).min(24);
                }

                if level == 0 {
                    spans.push(Span::raw(" "));
                    continue;
                }

                let top = level / 5;
                let bottom = level % 5;
                let mut top_color = theme::TONES[top];
                let mut bottom_color = theme::TONES[bottom];

                if on_scan {
                    top_color = theme::blend(top_color, theme::TEXT, 0.30 + 0.45 * glow);
                    bottom_color = theme::blend(bottom_color, theme::TEXT, 0.30 + 0.45 * glow);
                }

                let span = if top == bottom {
                    Span::styled("█", Style::default().fg(top_color))
                } else if top == 0 {
                    Span::styled("▄", Style::default().fg(bottom_color))
                } else if bottom == 0 {
                    Span::styled("▀", Style::default().fg(top_color))
                } else {
                    Span::styled("▀", Style::default().fg(top_color).bg(bottom_color))
                };

                spans.push(span);
            }

            Line::from(spans)
        })
        .collect()
}

fn level(character: char) -> usize {
    match character {
        '0'..='9' => character as usize - '0' as usize,
        'a'..='o' => 10 + (character as usize - 'a' as usize),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{avatar_compact_lines, avatar_lines, logo_lines, COMPACT_H, COMPACT_W, LOGO_W};
    use crate::art_data::{AVATAR_H, AVATAR_W};

    #[test]
    fn logo_has_fixed_shape() {
        for line in logo_lines(1.0, 0.5) {
            assert_eq!(line.width(), LOGO_W);
        }
        assert_eq!(logo_lines(1.0, 0.5).len(), 6);
    }

    #[test]
    fn avatar_shapes_are_stable() {
        let full = avatar_lines(Some(3), 0.5);
        assert_eq!(full.len(), AVATAR_H);
        assert!(full.iter().all(|line| line.width() == AVATAR_W));

        let compact = avatar_compact_lines(Some(3), 0.5);
        assert_eq!(compact.len(), COMPACT_H);
        assert!(compact.iter().all(|line| line.width() == COMPACT_W));
    }
}
