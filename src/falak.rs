use chrono::{DateTime, Datelike, Timelike, Utc};

/// Styles for displaying Moon glyphs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonGlyphStyle {
    Geometric, // Default: ○ ◔ ◑ ◕ ● ◕ ◐ ◔ (clean 1-cell monospace geometric shapes)
    Lunar,     // Unicode Lunar Emojis/Symbols: 🌑 🌒 🌓 🌔 🌕 🌖 🌗 🌘
    Classic,   // Traditional Astrological symbols: ○ ☽ ● ☾
    Text,      // Pure text: [Waxing], [Full], etc.
}

impl MoonGlyphStyle {
    pub fn default_style() -> Self {
        MoonGlyphStyle::Geometric
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "geometric" | "geo" | "1" => Some(MoonGlyphStyle::Geometric),
            "lunar" | "unicode" | "2" => Some(MoonGlyphStyle::Lunar),
            "classic" | "astro" | "3" => Some(MoonGlyphStyle::Classic),
            "text" | "none" | "4" => Some(MoonGlyphStyle::Text),
            _ => None,
        }
    }
}

/// Information about the current Moon Phase
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoonPhase {
    pub illumination: f64,        // 0.0 to 1.0 (e.g. 0.54 = 54%)
    pub age_days: f64,            // Age in current synodic cycle (~0 to 29.53 days)
    pub phase_name: &'static str, // "New Moon", "Waxing Crescent", etc.
    pub phase_normalized: f64,    // 0.0 to 1.0 cycle fraction
}

impl MoonPhase {
    pub fn glyph(&self, style: MoonGlyphStyle) -> &'static str {
        match style {
            MoonGlyphStyle::Geometric => match self.phase_normalized {
                p if p < 0.03 || p >= 0.97 => "○",
                p if p < 0.22 => "◔",
                p if p < 0.28 => "◑",
                p if p < 0.47 => "◕",
                p if p < 0.53 => "●",
                p if p < 0.72 => "◕",
                p if p < 0.78 => "◐",
                _ => "◔",
            },
            MoonGlyphStyle::Lunar => match self.phase_normalized {
                p if p < 0.03 || p >= 0.97 => "🌑",
                p if p < 0.22 => "🌒",
                p if p < 0.28 => "🌓",
                p if p < 0.47 => "🌔",
                p if p < 0.53 => "🌕",
                p if p < 0.72 => "🌖",
                p if p < 0.78 => "🌗",
                _ => "🌘",
            },
            MoonGlyphStyle::Classic => match self.phase_normalized {
                p if p < 0.03 || p >= 0.97 => "○",
                p if p < 0.47 => "☽",
                p if p < 0.53 => "●",
                _ => "☾",
            },
            MoonGlyphStyle::Text => match self.phase_normalized {
                p if p < 0.03 || p >= 0.97 => "[NEW]",
                p if p < 0.22 => "[WAX-CR]",
                p if p < 0.28 => "[1ST-QTR]",
                p if p < 0.47 => "[WAX-GB]",
                p if p < 0.53 => "[FULL]",
                p if p < 0.72 => "[WAN-GB]",
                p if p < 0.78 => "[LST-QTR]",
                _ => "[WAN-CR]",
            },
        }
    }
}

/// Real-time Solar Position & Sky State (shams)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunPosition {
    pub altitude: f64,           // Degrees above or below horizon (-90° to +90°)
    pub azimuth: f64,            // Compass heading in degrees (0° North, 90° East, 180° South, 270° West)
    pub state_name: &'static str,// "Daylight", "Civil Twilight", "Nautical Twilight", "Astronomical Twilight", "Night"
    pub compass_direction: &'static str, // "N", "NE", "E", "SE", "S", "SW", "W", "NW"
}

/// The Celestial Engine (Domain: falak)
pub struct Falak;

impl Falak {
    /// Calculate the Julian Day Number from a UTC DateTime
    pub fn julian_day(utc: DateTime<Utc>) -> f64 {
        let mut year = utc.year();
        let mut month = utc.month() as i32;
        let day = utc.day() as f64;
        let hour = utc.hour() as f64 + (utc.minute() as f64 / 60.0) + (utc.second() as f64 / 3600.0);

        if month <= 2 {
            year -= 1;
            month += 12;
        }

        let a = (year as f64 / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();

        (365.25 * (year as f64 + 4716.0)).floor()
            + (30.6001 * (month as f64 + 1.0)).floor()
            + day
            + (hour / 24.0)
            + b
            - 1524.5
    }

    /// Calculate the Moon's phase, illumination, and normalized cycle
    pub fn calculate_moon(utc: DateTime<Utc>) -> MoonPhase {
        let jd = Self::julian_day(utc);

        let known_new_moon = 2451549.26;
        let synodic_month = 29.53058867;

        let cycles = (jd - known_new_moon) / synodic_month;
        let phase_normalized = cycles - cycles.floor(); // 0.0 to 1.0
        let age_days = phase_normalized * synodic_month;

        let phase_angle = phase_normalized * 2.0 * std::f64::consts::PI;
        let illumination = (1.0 - phase_angle.cos()) / 2.0;

        let phase_name = match phase_normalized {
            p if p < 0.03 || p >= 0.97 => "New Moon",
            p if p < 0.22 => "Waxing Crescent",
            p if p < 0.28 => "First Quarter",
            p if p < 0.47 => "Waxing Gibbous",
            p if p < 0.53 => "Full Moon",
            p if p < 0.72 => "Waning Gibbous",
            p if p < 0.78 => "Last Quarter",
            _ => "Waning Crescent",
        };

        MoonPhase {
            illumination,
            age_days,
            phase_name,
            phase_normalized,
        }
    }

    /// Calculate the Sun's real-time Altitude (elevation) and Azimuth (compass direction)
    /// Algorithm: NOAA Solar Position Algorithm / Astronomical Algorithms
    pub fn calculate_sun(utc: DateTime<Utc>, lat_deg: f64, lon_deg: f64) -> SunPosition {
        let jd = Self::julian_day(utc);
        let t = (jd - 2451545.0) / 36525.0; // Julian centuries since J2000.0

        // Geometric Mean Longitude of Sun (deg)
        let l0 = (280.46646 + t * (36000.76983 + 0.0003032 * t)) % 360.0;

        // Mean Anomaly of Sun (deg)
        let m = (357.52911 + t * (35999.05029 - 0.0001537 * t)) % 360.0;
        let m_rad = m.to_radians();

        // Equation of Center (deg)
        let c = (1.914602 - t * (0.004817 + 0.000014 * t)) * m_rad.sin()
            + (0.019993 - 0.000101 * t) * (2.0 * m_rad).sin()
            + 0.000289 * (3.0 * m_rad).sin();

        // Sun True Longitude (deg)
        let sun_true_long = l0 + c;

        // Mean Obliquity of Ecliptic (deg)
        let eps0 = 23.439291 - t * (0.0130042 + 0.00000016 * t);
        let eps_rad = eps0.to_radians();
        let lambda_rad = sun_true_long.to_radians();

        // Solar Declination delta (rad)
        let sin_delta = eps_rad.sin() * lambda_rad.sin();
        let delta = sin_delta.asin();

        // Right Ascension alpha (rad)
        let y = eps_rad.cos() * lambda_rad.sin();
        let x = lambda_rad.cos();
        let alpha = y.atan2(x);

        // Greenwich Mean Sidereal Time (GMST in degrees)
        let gmst0 = (280.46061837 + 360.98564736629 * (jd - 2451545.0) + t * t * (0.000387933 - t / 38710000.0)) % 360.0;
        let local_sidereal_time = (gmst0 + lon_deg) % 360.0;

        // Local Hour Angle H (deg -> rad)
        let hour_angle_deg = (local_sidereal_time - alpha.to_degrees() + 360.0) % 360.0;
        let h_rad = hour_angle_deg.to_radians();

        let lat_rad = lat_deg.to_radians();

        // Altitude angle alpha (rad)
        let sin_alt = lat_rad.sin() * delta.sin() + lat_rad.cos() * delta.cos() * h_rad.cos();
        let alt_deg = sin_alt.asin().to_degrees();

        // Azimuth angle (from North through East)
        let cos_az = (delta.sin() - lat_rad.sin() * sin_alt) / (lat_rad.cos() * sin_alt.acos());
        let az_candidate = cos_az.clamp(-1.0, 1.0).acos().to_degrees();
        let az_deg = if h_rad.sin() > 0.0 {
            360.0 - az_candidate
        } else {
            az_candidate
        };

        // Determine sky state name
        let state_name = match alt_deg {
            a if a > 0.0 => "Daylight",
            a if a > -6.0 => "Civil Twilight (Golden Hour)",
            a if a > -12.0 => "Nautical Twilight",
            a if a > -18.0 => "Astronomical Twilight",
            _ => "Night",
        };

        // Compass 8-point direction
        let compass = match az_deg {
            a if a >= 337.5 || a < 22.5 => "N",
            a if a < 67.5 => "NE",
            a if a < 112.5 => "E",
            a if a < 157.5 => "SE",
            a if a < 202.5 => "S",
            a if a < 247.5 => "SW",
            a if a < 292.5 => "W",
            _ => "NW",
        };

        SunPosition {
            altitude: alt_deg,
            azimuth: az_deg,
            state_name,
            compass_direction: compass,
        }
    }
}
