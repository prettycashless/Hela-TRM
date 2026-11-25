use ratatui::style::Color;

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

pub fn blend(a: Color, b: Color, t: f32) -> Color {
    let (ar, ag, ab) = rgb(a);
    let (br, bg, bb) = rgb(b);
    let t = t.clamp(0.0, 1.0);
    let mix = |start: u8, end: u8| -> u8 {
        (start as f32 + (end as f32 - start as f32) * t).round() as u8
    };
    Color::Rgb(mix(ar, br), mix(ag, bg), mix(ab, bb))
}

fn rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(red, green, blue) => (red, green, blue),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        _ => (128, 128, 128),
    }
}

#[cfg(test)]
mod tests {
    use super::{blend, Color};

    #[test]
    fn blend_hits_both_ends() {
        let start = Color::Rgb(0, 0, 0);
        let end = Color::Rgb(255, 255, 255);
        assert_eq!(blend(start, end, 0.0), start);
        assert_eq!(blend(start, end, 1.0), end);
        assert_eq!(blend(start, end, 0.5), Color::Rgb(128, 128, 128));
    }
}
