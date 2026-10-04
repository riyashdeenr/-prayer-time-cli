use chrono::{DateTime, Local};
use salah::prelude::Prayer;

/// Represents the status of a prayer relative to current time (Passive Participle: mawqut)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MawqutStatus {
    Passed,
    Current,
    Upcoming,
}

impl MawqutStatus {
    /// Concise symbol for terminal display (no emojis)
    #[allow(dead_code)]
    pub fn symbol(&self) -> &'static str {
        match self {
            MawqutStatus::Passed => "[x]",
            MawqutStatus::Current => "[>]",
            MawqutStatus::Upcoming => "[ ]",
        }
    }

    /// Clean textual status
    #[allow(dead_code)]
    pub fn text(&self) -> &'static str {
        match self {
            MawqutStatus::Passed => "Passed",
            MawqutStatus::Current => "Current",
            MawqutStatus::Upcoming => "Upcoming",
        }
    }
}

/// A single prayer time entry (Singular Noun: miqat)
#[derive(Debug, Clone)]
pub struct Miqat {
    #[allow(dead_code)]
    pub prayer: Prayer,
    pub name: &'static str,
    pub time: DateTime<Local>,
    pub status: MawqutStatus,
}

impl Miqat {
    pub fn new(prayer: Prayer, time: DateTime<Local>, status: MawqutStatus) -> Self {
        let name = match prayer {
            Prayer::Fajr => "Fajr",
            Prayer::Sunrise => "Sunrise",
            Prayer::Dhuhr => "Dhuhr",
            Prayer::Asr => "Asr",
            Prayer::Maghrib => "Maghrib",
            Prayer::Isha => "Isha",
            Prayer::Qiyam => "Qiyam",
            Prayer::FajrTomorrow => "Fajr (Tomorrow)",
        };

        Self {
            prayer,
            name,
            time,
            status,
        }
    }
}
