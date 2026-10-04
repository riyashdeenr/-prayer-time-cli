use crate::miqat::MawqutStatus;

/// The 5 UI/UX styles for prayer status rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeStyle {
    Dots,      // Pattern 1: ● ◉ ○
    Timeline,  // Pattern 2: ├─ ✓, ├─ ▶, ├─ ·
    Blocks,    // Pattern 3: ■ ◆ □
    Radio,     // Pattern 4: ✔ ● ◌
    Minimal,   // Pattern 5: Pure typography & colors, no glyphs
}

impl ThemeStyle {
    /// The default style used when none is specified
    pub fn default_style() -> Self {
        ThemeStyle::Dots
    }

    /// Parse style name from CLI flag string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "dots" | "1" => Some(ThemeStyle::Dots),
            "timeline" | "2" => Some(ThemeStyle::Timeline),
            "blocks" | "3" => Some(ThemeStyle::Blocks),
            "radio" | "4" => Some(ThemeStyle::Radio),
            "minimal" | "5" => Some(ThemeStyle::Minimal),
            _ => None,
        }
    }

    /// Format a prayer row (name, time, status) using ANSI colors and selected style
    pub fn format_row(
        &self,
        name: &str,
        time_str: &str,
        status: MawqutStatus,
        is_last: bool,
    ) -> String {
        // ANSI escape codes:
        // \x1b[90m = Dim Gray
        // \x1b[1;32m = Bold Green
        // \x1b[1;36m = Bold Cyan
        // \x1b[37m = Crisp White
        // \x1b[0m = Reset
        const DIM: &str = "\x1b[90m";
        const ACTIVE: &str = "\x1b[38;2;46;204;113m"; // Emerald Green (#2ecc71)
        const NORMAL: &str = "\x1b[37m";
        const RESET: &str = "\x1b[0m";

        match self {
            // Pattern 1: Modern Dots (● ◉ ○)
            ThemeStyle::Dots => match status {
                MawqutStatus::Passed => {
                    format!("{}  ●  {:<10} {:<8}{}", DIM, name, time_str, RESET)
                }
                MawqutStatus::Current => {
                    format!("{}  ◉  {:<10} {:<8} (now){}", ACTIVE, name, time_str, RESET)
                }
                MawqutStatus::Upcoming => {
                    format!("{}  ○  {:<10} {:<8}{}", NORMAL, name, time_str, RESET)
                }
            },

            // Pattern 2: Timeline Track (├─ ✓, ├─ ▶, └─ ·)
            ThemeStyle::Timeline => {
                let branch = if is_last { "  └─ " } else { "  ├─ " };
                match status {
                    MawqutStatus::Passed => {
                        format!("{}{}✓  {:<10} {:<8}{}", DIM, branch, name, time_str, RESET)
                    }
                    MawqutStatus::Current => {
                        format!("{}{}\x1b[1;32m▶  {:<10} {:<8} (active){}", ACTIVE, branch, name, time_str, RESET)
                    }
                    MawqutStatus::Upcoming => {
                        format!("{}{}·  {:<10} {:<8}{}", NORMAL, branch, name, time_str, RESET)
                    }
                }
            }

            // Pattern 3: Minimalist Blocks (■ ◆ □)
            ThemeStyle::Blocks => match status {
                MawqutStatus::Passed => {
                    format!("{}  ■  {:<10} {:<8}{}", DIM, name, time_str, RESET)
                }
                MawqutStatus::Current => {
                    format!("{}  ◆  {:<10} {:<8} (current){}", "\x1b[1;33m", name, time_str, RESET) // Golden amber
                }
                MawqutStatus::Upcoming => {
                    format!("{}  □  {:<10} {:<8}{}", NORMAL, name, time_str, RESET)
                }
            },

            // Pattern 4: Radio Indicator (✔ ● ◌)
            ThemeStyle::Radio => match status {
                MawqutStatus::Passed => {
                    format!("{}  ✔  {:<10} {:<8}{}", DIM, name, time_str, RESET)
                }
                MawqutStatus::Current => {
                    format!("{}  ●  {:<10} {:<8} (now){}", "\x1b[1;32m", name, time_str, RESET) // Emerald green
                }
                MawqutStatus::Upcoming => {
                    format!("{}  ◌  {:<10} {:<8}{}", NORMAL, name, time_str, RESET)
                }
            },

            // Pattern 5: Pure Typography & Color (No extra glyphs)
            ThemeStyle::Minimal => match status {
                MawqutStatus::Passed => {
                    format!("{}     {:<10} {:<8}{}", DIM, name, time_str, RESET)
                }
                MawqutStatus::Current => {
                    format!("{}  >  {:<10} {:<8} (now){}", ACTIVE, name, time_str, RESET)
                }
                MawqutStatus::Upcoming => {
                    format!("{}     {:<10} {:<8}{}", NORMAL, name, time_str, RESET)
                }
            },
        }
    }
}
