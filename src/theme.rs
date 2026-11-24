use ratatui::style::{Color, Modifier, Style};

pub const BG: Color = Color::Rgb(8, 9, 12);
pub const ACCENT: Color = Color::Rgb(132, 216, 226);
pub const ACCENT_DIM: Color = Color::Rgb(84, 146, 158);
pub const TEXT: Color = Color::Rgb(214, 218, 226);
pub const MUTED: Color = Color::Rgb(110, 115, 126);
pub const FAINT: Color = Color::Rgb(54, 57, 66);
pub const GOOD: Color = Color::Rgb(146, 216, 170);
pub const WARN: Color = Color::Rgb(230, 196, 124);
pub const BAD: Color = Color::Rgb(226, 124, 134);

/// Five luminance steps used by the portrait.
pub const TONES: [Color; 5] = [
    Color::Rgb(10, 10, 13),
    Color::Rgb(54, 56, 64),
    Color::Rgb(99, 102, 113),
    Color::Rgb(178, 181, 190),
    Color::Rgb(241, 243, 249),
];

pub fn title() -> Style {
    Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
}

pub fn accent() -> Style {
    Style::default().fg(ACCENT)
}

pub fn faint() -> Style {
    Style::default().fg(FAINT)
}
