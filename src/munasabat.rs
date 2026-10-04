use chrono::{DateTime, Datelike, Duration, Local};

/// The juristic basis for defining the bounds of the Night
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NightCalculationBasis {
    /// Standard Majority Definition: From Sunset (Maghrib) to True Dawn (Fajr)
    SunsetToDawn,
    /// Alternative Juristic Definition: From Isha to True Dawn (Fajr)
    IshaToDawn,
}

impl NightCalculationBasis {
    pub fn default_basis() -> Self {
        NightCalculationBasis::SunsetToDawn
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['-', '_', ' '], "").as_str() {
            "sunset" | "maghrib" | "standard" | "majority" | "1" => {
                Some(NightCalculationBasis::SunsetToDawn)
            }
            "isha" | "alternative" | "2" => Some(NightCalculationBasis::IshaToDawn),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            NightCalculationBasis::SunsetToDawn => "Maghrib to Fajr (Standard Majority)",
            NightCalculationBasis::IshaToDawn => "Isha to Fajr (Alternative)",
        }
    }
}

/// A specific segment or third of the night
#[derive(Debug, Clone)]
pub struct NightSegment {
    pub name: &'static str,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

/// The complete astronomical breakdown of the night
#[derive(Debug, Clone)]
pub struct NightBreakdown {
    pub basis: NightCalculationBasis,
    pub night_start: DateTime<Local>,
    pub night_end: DateTime<Local>,
    pub total_duration: Duration,
    pub midnight: DateTime<Local>,        // Nisf al-Layl (exact halfway point)
    pub first_third: NightSegment,        // Early night
    pub second_third: NightSegment,       // Middle night
    pub last_third: NightSegment,         // Thuluth al-Akhir (Divine descent / Tahajjud)
    pub is_currently_last_third: bool,    // True if current time is within last third
    pub time_until_last_third: Option<Duration>,
}

/// Current state of the fasting cycle
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FastingStatus {
    /// Before Imsak buffer (eating permitted freely)
    SuhoorPermitted,
    /// Within Imsak safety buffer (finishing Suhoor, rinsing, preparing for Fajr)
    ImsakBuffer,
    /// Between Fajr and Maghrib (actively fasting)
    FastingActive,
    /// After Maghrib (fast completed / broken)
    FastCompleted,
}

/// The complete fasting & Suhoor breakdown for the day
#[derive(Debug, Clone)]
pub struct FastingInfo {
    pub imsak_time: DateTime<Local>,
    pub suhoor_cutoff: DateTime<Local>,   // Fajr
    pub iftar_time: DateTime<Local>,       // Maghrib
    pub imsak_buffer_minutes: i64,
    pub fasting_duration: Duration,        // Fajr to Maghrib
    pub eating_window_duration: Duration,  // Maghrib to Fajr
    pub status: FastingStatus,
    pub time_until_imsak: Option<Duration>,
    pub time_until_suhoor_cutoff: Option<Duration>,
    pub time_until_iftar: Option<Duration>,
    pub elapsed_fasting_time: Option<Duration>,
}

/// Prohibited prayer times (Awqat al-Nahy) and Duha (forenoon) window
#[derive(Debug, Clone)]
pub struct ProhibitedTimesInfo {
    pub sunrise_prohibited_start: DateTime<Local>, // Sunrise
    pub sunrise_prohibited_end: DateTime<Local>,   // Sunrise + 15m (Tulu' & Ishraq start)
    pub zawal_prohibited_start: DateTime<Local>,   // Dhuhr - 10m (Istiwa' / exact zenith)
    pub zawal_prohibited_end: DateTime<Local>,     // Dhuhr entry
    pub sunset_prohibited_start: DateTime<Local>,  // Maghrib - 15m (Yellowing of sun)
    pub sunset_prohibited_end: DateTime<Local>,    // Maghrib entry
    pub duha_start: DateTime<Local>,               // Ishraq (Sunrise + 15m)
    pub duha_end: DateTime<Local>,                 // Dhuhr - 10m
    pub is_currently_prohibited: bool,
    pub prohibited_reason: Option<&'static str>,
    pub is_currently_duha: bool,
}

/// Friday (Yawm al-Jumu'ah) Hour of Response (Sa'at al-Istijabah)
#[derive(Debug, Clone)]
pub struct FridayIstijabahInfo {
    pub is_friday: bool,
    pub window_start: DateTime<Local>, // Typically last hour before Maghrib
    pub window_end: DateTime<Local>,   // Maghrib
    pub is_currently_active: bool,
    pub time_until_window: Option<Duration>,
}

/// An annual Islamic sacred day or observance
#[derive(Debug, Clone)]
pub struct AnnualObservanceInfo {
    pub name: &'static str,
    pub hijri_day: u32,
    pub hijri_month: u32,
    pub month_name: &'static str,
    pub greg_date: chrono::NaiveDate,
    pub greg_weekday: &'static str,
    pub days_until: i64,
    pub is_today: bool,
    pub is_fasting_forbidden: bool, // E.g., Eid days, Tashriq
    pub description: &'static str,
}

/// Observances, Special Stations & Astronomical Markers (Domain: munasabat / مناسبات)
pub struct Munasabat;

impl Munasabat {
    /// Calculate the night divisions and the last third of the night
    pub fn calculate_night(
        now: DateTime<Local>,
        today_maghrib: DateTime<Local>,
        today_isha: DateTime<Local>,
        tomorrow_fajr: DateTime<Local>,
        basis: NightCalculationBasis,
    ) -> NightBreakdown {
        let night_start = match basis {
            NightCalculationBasis::SunsetToDawn => today_maghrib,
            NightCalculationBasis::IshaToDawn => today_isha,
        };

        let night_end = tomorrow_fajr;
        let total_secs = (night_end - night_start).num_seconds().max(0);
        let total_duration = Duration::seconds(total_secs);

        // Halfway point (Midnight / Nisf al-Layl)
        let midnight = night_start + Duration::seconds(total_secs / 2);

        // Each third duration
        let third_secs = total_secs / 3;
        let t1_end = night_start + Duration::seconds(third_secs);
        let t2_end = night_start + Duration::seconds(third_secs * 2);

        let first_third = NightSegment {
            name: "First Third",
            start: night_start,
            end: t1_end,
        };

        let second_third = NightSegment {
            name: "Second Third",
            start: t1_end,
            end: t2_end,
        };

        let last_third = NightSegment {
            name: "Last Third (Thuluth al-Akhir)",
            start: t2_end,
            end: night_end,
        };

        let is_currently_last_third = now >= last_third.start && now < last_third.end;

        let time_until_last_third = if now < last_third.start {
            Some(last_third.start - now)
        } else {
            None
        };

        NightBreakdown {
            basis,
            night_start,
            night_end,
            total_duration,
            midnight,
            first_third,
            second_third,
            last_third,
            is_currently_last_third,
            time_until_last_third,
        }
    }

    /// Calculate fasting schedule, Imsak buffer, and fasting status
    pub fn calculate_fasting(
        now: DateTime<Local>,
        today_fajr: DateTime<Local>,
        today_maghrib: DateTime<Local>,
        _tomorrow_fajr: DateTime<Local>,
        imsak_buffer_minutes: i64,
    ) -> FastingInfo {
        let imsak_time = today_fajr - Duration::minutes(imsak_buffer_minutes);
        let suhoor_cutoff = today_fajr;
        let iftar_time = today_maghrib;

        let fasting_duration = (iftar_time - suhoor_cutoff).max(Duration::zero());
        let eating_window_duration = Duration::hours(24) - fasting_duration;

        let status = if now < imsak_time {
            FastingStatus::SuhoorPermitted
        } else if now < suhoor_cutoff {
            FastingStatus::ImsakBuffer
        } else if now < iftar_time {
            FastingStatus::FastingActive
        } else {
            FastingStatus::FastCompleted
        };

        let time_until_imsak = if now < imsak_time {
            Some(imsak_time - now)
        } else {
            None
        };

        let time_until_suhoor_cutoff = if now < suhoor_cutoff {
            Some(suhoor_cutoff - now)
        } else {
            None
        };

        let time_until_iftar = if now >= suhoor_cutoff && now < iftar_time {
            Some(iftar_time - now)
        } else {
            None
        };

        let elapsed_fasting_time = if now >= suhoor_cutoff && now < iftar_time {
            Some(now - suhoor_cutoff)
        } else {
            None
        };

        FastingInfo {
            imsak_time,
            suhoor_cutoff,
            iftar_time,
            imsak_buffer_minutes,
            fasting_duration,
            eating_window_duration,
            status,
            time_until_imsak,
            time_until_suhoor_cutoff,
            time_until_iftar,
            elapsed_fasting_time,
        }
    }

    /// Calculate prohibited prayer times (Awqat al-Nahy) and Duha window
    pub fn calculate_prohibited_times(
        now: DateTime<Local>,
        sunrise: DateTime<Local>,
        dhuhr: DateTime<Local>,
        maghrib: DateTime<Local>,
    ) -> ProhibitedTimesInfo {
        let sunrise_prohibited_start = sunrise;
        let sunrise_prohibited_end = sunrise + Duration::minutes(15);

        let zawal_prohibited_start = dhuhr - Duration::minutes(10);
        let zawal_prohibited_end = dhuhr;

        let sunset_prohibited_start = maghrib - Duration::minutes(15);
        let sunset_prohibited_end = maghrib;

        let duha_start = sunrise_prohibited_end;
        let duha_end = zawal_prohibited_start;

        let (is_currently_prohibited, prohibited_reason) = if now >= sunrise_prohibited_start && now < sunrise_prohibited_end {
            (true, Some("Sunrise / Tulu' al-Shams (Sun ascending above horizon)"))
        } else if now >= zawal_prohibited_start && now < zawal_prohibited_end {
            (true, Some("Solar Zenith / Istiwa' (Sun at exact peak before inclining)"))
        } else if now >= sunset_prohibited_start && now < sunset_prohibited_end {
            (true, Some("Sunset / Ghurub (Yellowing of sun before setting)"))
        } else {
            (false, None)
        };

        let is_currently_duha = now >= duha_start && now < duha_end;

        ProhibitedTimesInfo {
            sunrise_prohibited_start,
            sunrise_prohibited_end,
            zawal_prohibited_start,
            zawal_prohibited_end,
            sunset_prohibited_start,
            sunset_prohibited_end,
            duha_start,
            duha_end,
            is_currently_prohibited,
            prohibited_reason,
            is_currently_duha,
        }
    }

    /// Calculate Friday (Yawm al-Jumu'ah) Hour of Response (Sa'at al-Istijabah)
    pub fn calculate_friday_istijabah(
        now: DateTime<Local>,
        maghrib: DateTime<Local>,
    ) -> FridayIstijabahInfo {
        let is_friday = now.weekday() == chrono::Weekday::Fri;
        let window_start = maghrib - Duration::hours(1);
        let window_end = maghrib;

        let is_currently_active = is_friday && now >= window_start && now < window_end;
        let time_until_window = if is_friday && now < window_start {
            Some(window_start - now)
        } else {
            None
        };

        FridayIstijabahInfo {
            is_friday,
            window_start,
            window_end,
            is_currently_active,
            time_until_window,
        }
    }

    /// Calculate key annual Islamic observances and sacred days for the current Hijri year
    pub fn calculate_annual_observances(
        current_hijri_year: i32,
        anchor_gregorian: chrono::NaiveDate,
        default_offset: i32,
        adjustments: &std::collections::HashMap<String, i32>,
    ) -> Vec<AnnualObservanceInfo> {
        // List of major canonical Islamic stations: (Name, Hijri Day, Hijri Month, IsFastingForbidden, Description)
        let canonical_events = [
            ("Tasu'a (Eve of Ashura)", 9, 1, false, "Recommended voluntary fast preceding Ashura"),
            ("Day of 'Ashura", 10, 1, false, "Major voluntary fast expiating sins of preceding year"),
            ("Mid-Sha'ban (Nisf Sha'ban)", 15, 8, false, "Night of forgiveness and voluntary fast"),
            ("1st of Ramadan", 1, 9, false, "Beginning of the obligatory month of fasting"),
            ("Laylat al-Qadr (27th Night)", 27, 9, false, "Night of Power & Decree (prime odd night search)"),
            ("Eid al-Fitr", 1, 10, true, "Celebration of fast-breaking (Fasting strictly FORBIDDEN)"),
            ("Day of 'Arafah", 9, 12, false, "Peak of Hajj & greatest voluntary fast of the year"),
            ("Eid al-Adha", 10, 12, true, "Feast of the Sacrifice (Fasting strictly FORBIDDEN)"),
            ("Days of Tashriq", 11, 12, true, "Days of eating, drinking, and remembrance (Fasting FORBIDDEN)"),
        ];

        let month_names = crate::taqwim::TaqwimDate::month_names();
        let mut results = Vec::new();

        for (name, h_day, h_month, is_forbidden, desc) in canonical_events {
            let (greg_date, greg_weekday, _) = crate::taqwim::Taqwim::convert_hijri_to_gregorian(
                current_hijri_year,
                h_month,
                h_day,
                anchor_gregorian,
                default_offset,
                adjustments,
            );

            let days_until = (greg_date - anchor_gregorian).num_days();
            let is_today = days_until == 0;
            let m_name = month_names[(h_month - 1) as usize];

            results.push(AnnualObservanceInfo {
                name,
                hijri_day: h_day,
                hijri_month: h_month,
                month_name: m_name,
                greg_date,
                greg_weekday,
                days_until,
                is_today,
                is_fasting_forbidden: is_forbidden,
                description: desc,
            });
        }

        results
    }
}
