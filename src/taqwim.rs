use chrono::{DateTime, Datelike, Duration, Local, NaiveDate};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// The Islamic Calendar Month Representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaqwimDate {
    pub day: u32,
    pub month: u32,
    pub month_name: &'static str,
    pub year: i32,
    pub weekday_name: &'static str, // e.g. "Sunday", "Monday"
    pub applied_offset: i32,        // The exact offset active for this month (+1, 0, -1)
}

impl TaqwimDate {
    pub fn month_names() -> [&'static str; 12] {
        [
            "Muharram",
            "Safar",
            "Rabi' al-Awwal",
            "Rabi' al-Thani",
            "Jumada al-Ula",
            "Jumada al-Akhirah",
            "Rajab",
            "Sha'ban",
            "Ramadan",
            "Shawwal",
            "Dhu al-Qi'dah",
            "Dhu al-Hijjah",
        ]
    }

    /// Format standard textual date string: e.g. "Sunday, 21 Rabi' al-Thani 1448 AH"
    pub fn format(&self) -> String {
        format!("{}, {} {} {} AH", self.weekday_name, self.day, self.month_name, self.year)
    }

    /// Short format: e.g. "21 Rabi' al-Thani 1448 AH"
    pub fn format_short(&self) -> String {
        format!("{} {} {} AH", self.day, self.month_name, self.year)
    }

    /// Return Month key string e.g. "1448-04" for month-indexed adjustments
    pub fn month_key(&self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }
}

/// Information about the 1st of the current lunar month (anchor for moonsighting verification)
#[derive(Debug, Clone)]
pub struct MonthStartInfo {
    pub month_key: String,
    pub month_name: &'static str,
    pub year: i32,
    pub first_gregorian: NaiveDate,
    pub first_weekday: &'static str,
    pub applied_offset: i32,
}

/// Information about the White Days (Ayyam al-Bid: 13th, 14th, 15th)
#[derive(Debug, Clone)]
pub struct WhiteDaysInfo {
    pub month_name: &'static str,
    pub year: i32,
    pub day13_gregorian: NaiveDate,
    pub day13_weekday: &'static str,
    pub day14_gregorian: NaiveDate,
    pub day14_weekday: &'static str,
    pub day15_gregorian: NaiveDate,
    pub day15_weekday: &'static str,
    pub is_currently_white_day: bool,
    pub current_white_day_number: Option<u32>, // Some(13), Some(14), or Some(15)
    pub days_until_start: i64,                 // Negative if past, 0 if active, positive if upcoming
}

/// Calendar Ledger & Calculator (Domain: taqwim / تقويم)
pub struct Taqwim;

impl Taqwim {
    /// Calculate base astronomical Hijri date from Julian Day Number
    /// Using standard Astronomical Tabular algorithm (Meeus / Kuwaiti algorithm)
    pub fn jd_to_hijri(jd: f64) -> (i32, u32, u32) {
        let jd = jd.floor() + 0.5;

        // Days since Hijri epoch (July 16, 622 CE = JD 1948439.5)
        let z = jd - 1948440.0;
        let cyc = (z / 10631.0).floor();
        let z_rem = z - cyc * 10631.0;

        let j = ((z_rem + 0.5) / 354.36667).floor() as i32;
        let year = (cyc as i32 * 30) + j + 1;

        let days_in_year = z_rem - (j as f64 * 354.36667).floor();

        let mut month = ((days_in_year + 0.5) / 29.5).floor() as u32 + 1;
        if month > 12 {
            month = 12;
        }

        let prior_days = if month == 1 {
            0.0
        } else {
            ((month as f64 - 1.0) * 29.5).round()
        };

        let day = (days_in_year - prior_days + 1.0).floor().max(1.0).min(30.0) as u32;

        (year, month, day)
    }

    /// Calculate Hijri date taking into account:
    /// 1. Sunset / Maghrib boundary (new Islamic day starts at Maghrib)
    /// 2. Month-specific ledger adjustments (e.g. "1448-04" -> +1)
    /// 3. Default base offset
    pub fn calculate(
        now: DateTime<Local>,
        today_maghrib: DateTime<Local>,
        default_offset: i32,
        adjustments: &HashMap<String, i32>,
    ) -> TaqwimDate {
        let is_after_maghrib = now >= today_maghrib;
        let mut target_date = now.date_naive();
        if is_after_maghrib {
            target_date += Duration::days(1);
        }

        let jd = Self::gregorian_to_jd(target_date);
        let (base_year, base_month, _) = Self::jd_to_hijri(jd);

        let month_key = format!("{:04}-{:02}", base_year, base_month);
        let effective_offset = adjustments
            .get(&month_key)
            .copied()
            .unwrap_or(default_offset);

        let adjusted_jd = jd + effective_offset as f64;
        let (final_year, final_month, final_day) = Self::jd_to_hijri(adjusted_jd);

        let month_names = TaqwimDate::month_names();
        let month_idx = (final_month.saturating_sub(1) as usize).min(11);
        let weekday_name = Self::weekday_name(target_date.weekday());

        TaqwimDate {
            day: final_day,
            month: final_month,
            month_name: month_names[month_idx],
            year: final_year,
            weekday_name,
            applied_offset: effective_offset,
        }
    }

    /// Find the Gregorian date corresponding to Day 1 of the given Hijri month
    pub fn find_month_start(
        hijri_year: i32,
        hijri_month: u32,
        anchor_gregorian: NaiveDate,
        default_offset: i32,
        adjustments: &HashMap<String, i32>,
    ) -> MonthStartInfo {
        let month_key = format!("{:04}-{:02}", hijri_year, hijri_month);
        let offset = adjustments.get(&month_key).copied().unwrap_or(default_offset);

        // Compute estimated Gregorian date of Day 1 of target Hijri month
        // An Islamic month is ~29.53 days.
        let anchor_jd = Self::gregorian_to_jd(anchor_gregorian);
        let (anchor_y, anchor_m, anchor_d) = Self::jd_to_hijri(anchor_jd);
        let months_diff = (hijri_year - anchor_y) * 12 + (hijri_month as i32 - anchor_m as i32);
        let days_shift = (months_diff as f64 * 29.53059).round() as i64 - (anchor_d as i64 - 1);

        let center_date = anchor_gregorian + Duration::days(days_shift);

        // Search backward and forward around center date to find Day 1
        let mut candidate = center_date - Duration::days(20);
        let mut found_date = center_date;

        for _ in 0..40 {
            let jd = Self::gregorian_to_jd(candidate) + offset as f64;
            let (y, m, d) = Self::jd_to_hijri(jd);
            if y == hijri_year && m == hijri_month && d == 1 {
                found_date = candidate;
                break;
            }
            candidate += Duration::days(1);
        }

        let month_names = TaqwimDate::month_names();
        let month_idx = (hijri_month.saturating_sub(1) as usize).min(11);

        MonthStartInfo {
            month_key,
            month_name: month_names[month_idx],
            year: hijri_year,
            first_gregorian: found_date,
            first_weekday: Self::weekday_name(found_date.weekday()),
            applied_offset: offset,
        }
    }

    /// Convert a given Hijri date (year, month, day) to its Gregorian equivalent
    pub fn convert_hijri_to_gregorian(
        hijri_year: i32,
        hijri_month: u32,
        hijri_day: u32,
        anchor_gregorian: NaiveDate,
        default_offset: i32,
        adjustments: &HashMap<String, i32>,
    ) -> (NaiveDate, &'static str, i32) {
        let month_start = Self::find_month_start(
            hijri_year,
            hijri_month,
            anchor_gregorian,
            default_offset,
            adjustments,
        );

        let target_gregorian = month_start.first_gregorian + Duration::days((hijri_day.saturating_sub(1)) as i64);
        let weekday = Self::weekday_name(target_gregorian.weekday());

        (target_gregorian, weekday, month_start.applied_offset)
    }

    /// Calculate White Days (Ayyam al-Bid 13th, 14th, 15th) for the given Hijri month
    pub fn calculate_white_days(
        current_taqwim: &TaqwimDate,
        today_gregorian: NaiveDate,
        default_offset: i32,
        adjustments: &HashMap<String, i32>,
    ) -> WhiteDaysInfo {
        let (d13, w13, _) = Self::convert_hijri_to_gregorian(
            current_taqwim.year,
            current_taqwim.month,
            13,
            today_gregorian,
            default_offset,
            adjustments,
        );
        let (d14, w14, _) = Self::convert_hijri_to_gregorian(
            current_taqwim.year,
            current_taqwim.month,
            14,
            today_gregorian,
            default_offset,
            adjustments,
        );
        let (d15, w15, _) = Self::convert_hijri_to_gregorian(
            current_taqwim.year,
            current_taqwim.month,
            15,
            today_gregorian,
            default_offset,
            adjustments,
        );

        let is_currently_white_day = current_taqwim.day >= 13 && current_taqwim.day <= 15;
        let current_white_day_number = if is_currently_white_day {
            Some(current_taqwim.day)
        } else {
            None
        };

        let days_until_start = (d13 - today_gregorian).num_days();

        WhiteDaysInfo {
            month_name: current_taqwim.month_name,
            year: current_taqwim.year,
            day13_gregorian: d13,
            day13_weekday: w13,
            day14_gregorian: d14,
            day14_weekday: w14,
            day15_gregorian: d15,
            day15_weekday: w15,
            is_currently_white_day,
            current_white_day_number,
            days_until_start,
        }
    }

    pub fn weekday_name(w: chrono::Weekday) -> &'static str {
        match w {
            chrono::Weekday::Mon => "Monday",
            chrono::Weekday::Tue => "Tuesday",
            chrono::Weekday::Wed => "Wednesday",
            chrono::Weekday::Thu => "Thursday",
            chrono::Weekday::Fri => "Friday",
            chrono::Weekday::Sat => "Saturday",
            chrono::Weekday::Sun => "Sunday",
        }
    }

    /// Convert Gregorian NaiveDate to Julian Day Number at 12:00 UTC
    pub fn gregorian_to_jd(date: NaiveDate) -> f64 {
        let mut y = date.year();
        let mut m = date.month() as i32;
        let d = date.day() as f64;

        if m <= 2 {
            y -= 1;
            m += 12;
        }

        let a = (y as f64 / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();

        (365.25 * (y as f64 + 4716.0)).floor()
            + (30.6001 * (m as f64 + 1.0)).floor()
            + d
            + b
            - 1524.5
    }

    /// Audit log path
    pub fn log_path() -> PathBuf {
        let base_dir = if let Ok(app_data) = std::env::var("APPDATA") {
            PathBuf::from(app_data).join("mawaqit")
        } else if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            PathBuf::from(home).join(".config").join("mawaqit")
        } else {
            PathBuf::from(".").join(".mawaqit")
        };

        base_dir.join("taqwim_adjustments.log")
    }

    /// Append an adjustment audit log entry
    pub fn record_audit_log(
        month_key: &str,
        offset: i32,
        reason: Option<&str>,
    ) -> Result<(), String> {
        let log_file = Self::log_path();
        if let Some(parent) = log_file.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let reason_str = reason.unwrap_or("Manual CLI update");

        writeln!(
            file,
            "{} | Month: {} | Offset: {:+} | Reason: {}",
            timestamp, month_key, offset, reason_str
        )
        .map_err(|e| format!("Failed to write to log: {}", e))?;

        Ok(())
    }
}
