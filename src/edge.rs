use crate::bayan::Bayan;
use crate::falak::{Falak, MoonGlyphStyle};
use crate::munasabat::{Munasabat, NightCalculationBasis};
use crate::muwaqqit::Muwaqqit;
use crate::taqwim::Taqwim;
use crate::tawqit::Tawqit;
use crate::theme::ThemeStyle;
use chrono::{DateTime, Duration, Local, Utc};
use salah::prelude::{Coordinates, Madhab, Method, Prayer, PrayerSchedule};
use std::collections::HashMap;

/// Parameters for rendering an edge request
#[derive(Debug, Clone)]
pub struct EdgeRenderParams {
    pub location_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone_offset_hours: i32,
    pub method: Method,
    pub madhab: Madhab,
    pub custom_fajr_angle: Option<f64>,
    pub custom_isha_angle: Option<f64>,
    pub format_template: Option<String>,
    pub subroute: EdgeSubroute,
    pub is_terminal: bool, // true if curl/httpie/wget, false if browser
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeSubroute {
    DefaultTable,
    Help,
    Fasting,
    Night,
    WhiteDays,
    Prohibited,
    Observances,
}

fn format_duration(total_secs: i64) -> String {
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

/// Renders the complete output for an edge request as a string
pub fn render_edge_response(params: &EdgeRenderParams, utc_timestamp: i64) -> String {
    if params.subroute == EdgeSubroute::Help {
        return render_edge_help();
    }

    let utc_dt = DateTime::<Utc>::from_timestamp(utc_timestamp, 0)
        .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap());

    // In WebAssembly, Local has +00:00 offset (UTC).
    // salah::PrayerSchedule calculates times in UTC.
    // Therefore, muwaqqit.calculate must compare against UTC time (utc_dt) to correctly identify current/next prayer!
    let now_for_calculation: DateTime<Local> = DateTime::from_naive_utc_and_offset(utc_dt.naive_utc(), *Local::now().offset());
    let local_naive = utc_dt.naive_utc() + Duration::hours(params.timezone_offset_hours as i64);
    let now_local: DateTime<Local> = DateTime::from_naive_utc_and_offset(local_naive, *Local::now().offset());



    let tawqit = Tawqit {
        location_name: params.location_name.clone(),
        coordinates: Coordinates::new(params.latitude, params.longitude),
        method: params.method,
        madhab: params.madhab,
        custom_fajr_angle: params.custom_fajr_angle,
        custom_isha_angle: params.custom_isha_angle,
    };

    let muwaqqit = Muwaqqit::new(tawqit.clone());
    let mawaqit = match muwaqqit.calculate(now_for_calculation) {
        Ok(mut m) => {
            m.date = local_naive.date();
            m
        },
        Err(e) => return format!("Error calculating prayer times: {}\n", e),
    };


    let today_maghrib = mawaqit
        .schedule
        .iter()
        .find(|m| m.prayer == Prayer::Maghrib)
        .map(|m| m.time)
        .unwrap_or(now_local);

    let tomorrow_date = now_local.date_naive() + Duration::days(1);
    let tomorrow_fajr: DateTime<Local> = PrayerSchedule::new()
        .on(tomorrow_date)
        .for_location(tawqit.coordinates)
        .with_configuration(tawqit.to_salah_params())
        .calculate()
        .map(|s| {
            let salah_dt = s.time(Prayer::Fajr);
            DateTime::from_timestamp(salah_dt.timestamp(), 0)
                .map(|u: DateTime<Utc>| u.with_timezone(&Local))
                .unwrap_or(today_maghrib + Duration::hours(10))
        })
        .unwrap_or(today_maghrib + Duration::hours(10));

    let today_fajr = mawaqit
        .schedule
        .iter()
        .find(|m| m.prayer == Prayer::Fajr)
        .map(|m| m.time)
        .unwrap_or(now_local);

    let today_sunrise = mawaqit
        .schedule
        .iter()
        .find(|m| m.prayer == Prayer::Sunrise)
        .map(|m| m.time)
        .unwrap_or(now_local);

    let today_dhuhr = mawaqit
        .schedule
        .iter()
        .find(|m| m.prayer == Prayer::Dhuhr)
        .map(|m| m.time)
        .unwrap_or(now_local);

    let _today_asr = mawaqit
        .schedule
        .iter()
        .find(|m| m.prayer == Prayer::Asr)
        .map(|m| m.time)
        .unwrap_or(now_local);

    let today_isha = mawaqit
        .schedule
        .iter()
        .find(|m| m.prayer == Prayer::Isha)
        .map(|m| m.time)
        .unwrap_or(now_local);

    let night = Munasabat::calculate_night(
        now_local,
        today_maghrib,
        today_isha,
        tomorrow_fajr,
        NightCalculationBasis::SunsetToDawn,
    );

    let fasting = Munasabat::calculate_fasting(
        now_local,
        today_fajr,
        today_maghrib,
        tomorrow_fajr,
        10, // 10 min default imsak
    );

    let prohibited = Munasabat::calculate_prohibited_times(
        now_local,
        today_sunrise,
        today_dhuhr,
        today_maghrib,
    );

    let friday = Munasabat::calculate_friday_istijabah(now_local, today_maghrib);

    let adjustments = HashMap::new();
    let taqwim_date = Taqwim::calculate(now_local, today_maghrib, 0, &adjustments);
    let _month_start = Taqwim::find_month_start(
        taqwim_date.year,
        taqwim_date.month,
        now_local.date_naive(),
        0,
        &adjustments,
    );

    let white_days = Taqwim::calculate_white_days(
        &taqwim_date,
        now_local.date_naive(),
        0,
        &adjustments,
    );
    let annual_events = Munasabat::calculate_annual_observances(
        taqwim_date.year,
        now_local.date_naive(),
        0,
        &adjustments,
    );

    let moon = Falak::calculate_moon(utc_dt);
    let sun = Falak::calculate_sun(utc_dt, params.latitude, params.longitude);

    let mut out = String::new();

    // 1. Check custom format template first
    if let Some(ref template) = params.format_template {
        let rendered = Bayan::format(
            template,
            &mawaqit,
            &moon,
            &sun,
            MoonGlyphStyle::Geometric,
            &taqwim_date,
            &night,
            &white_days,
            &fasting,
            &prohibited,
            &friday,
        );
        return format!("{}\n", rendered);
    }

    // 2. Specific Subroutes
    let tz_shift = Duration::hours(params.timezone_offset_hours as i64);

    match params.subroute {
        EdgeSubroute::Fasting => {
            let f_dur_m = fasting.fasting_duration.num_minutes();
            out.push_str("============================================================\n");
            out.push_str("  munasabat - Suhoor, Imsak & Fasting Schedule\n");
            out.push_str("============================================================\n");
            out.push_str(&format!("Location:           {} ({:.2}°, {:.2}°)\n", mawaqit.location_name, params.latitude, params.longitude));
            out.push_str(&format!("Date:               {} ({} / {})\n", mawaqit.date, taqwim_date.format(), taqwim_date.weekday_name));
            out.push_str(&format!("Precautionary Imsak: {} (-10 mins buffer)\n", (fasting.imsak_time + tz_shift).format("%H:%M")));
            out.push_str(&format!("Suhoor Cutoff (Fajr):{}\n", (fasting.suhoor_cutoff + tz_shift).format("%H:%M")));
            out.push_str(&format!("Iftar Time (Maghrib):{}\n\n", (fasting.iftar_time + tz_shift).format("%H:%M")));
            out.push_str(&format!("Total Fasting Span: {:02}h {:02}m\n", f_dur_m / 60, f_dur_m % 60));
            out.push_str("------------------------------------------------------------\n");
            return out;
        }
        EdgeSubroute::Night => {
            out.push_str("============================================================\n");
            out.push_str("  munasabat - Night Division & Tahajjud Timing\n");
            out.push_str("============================================================\n");
            out.push_str(&format!("Location:           {} | Date: {}\n", mawaqit.location_name, mawaqit.date));
            out.push_str(&format!("Midnight (Nisf):    {}\n", (night.midnight + tz_shift).format("%H:%M")));
            out.push_str(&format!("Last Third (Tahajjud): {} - {}\n", (night.last_third.start + tz_shift).format("%H:%M"), (night.last_third.end + tz_shift).format("%H:%M")));
            out.push_str("------------------------------------------------------------\n");
            return out;
        }

        EdgeSubroute::WhiteDays => {
            out.push_str("============================================================\n");
            out.push_str("  munasabat - White Days (Ayyam al-Bid 13-15)\n");
            out.push_str("============================================================\n");
            out.push_str(&format!("Taqwim:  {} ({})\n", taqwim_date.format(), taqwim_date.weekday_name));
            out.push_str(&format!("13th:    {} ({})\n", white_days.day13_gregorian.format("%a, %d %b %Y"), white_days.day13_weekday));
            out.push_str(&format!("14th:    {} ({})\n", white_days.day14_gregorian.format("%a, %d %b %Y"), white_days.day14_weekday));
            out.push_str(&format!("15th:    {} ({})\n", white_days.day15_gregorian.format("%a, %d %b %Y"), white_days.day15_weekday));
            out.push_str("------------------------------------------------------------\n");
            return out;
        }
        EdgeSubroute::Prohibited => {
            out.push_str("============================================================\n");
            out.push_str("  munasabat - Prohibited Prayer Times & Duha Window\n");
            out.push_str("============================================================\n");
            out.push_str(&format!("Duha Window:        {} - {}\n", (prohibited.duha_start + tz_shift).format("%H:%M"), (prohibited.duha_end + tz_shift).format("%H:%M")));
            if let Some(reason) = prohibited.prohibited_reason {

                out.push_str(&format!("⚠️ CURRENT STATUS: IN PROHIBITED WINDOW ({})\n", reason));
            } else if prohibited.is_currently_duha {
                out.push_str("✨ CURRENT STATUS: Duha Window Active (Salat al-Duha permissible)\n");
            } else {
                out.push_str("Status: Standard prayer times permissible.\n");
            }
            out.push_str("------------------------------------------------------------\n");
            return out;
        }
        EdgeSubroute::Observances => {
            out.push_str("============================================================\n");
            out.push_str("  munasabat - Major Islamic Annual Stations & Sacred Days\n");
            out.push_str("============================================================\n");
            for ev in &annual_events {
                let status_str = if ev.is_today {
                    "TODAY ★"
                } else if ev.days_until > 0 {
                    &format!("in {}d", ev.days_until)
                } else {
                    "passed"
                };
                out.push_str(&format!("{:<28} {:<14} {}\n", ev.name, format!("{} {}", ev.hijri_day, ev.month_name), status_str));
            }
            out.push_str("------------------------------------------------------------\n");
            return out;
        }
        _ => {}
    }

    // 3. Default Formatted Table (wttr.in style full table)
    let countdown_str = format_duration(mawaqit.time_until_next.num_seconds());
    let moon_symbol = moon.glyph(MoonGlyphStyle::Geometric);
    let moon_pct = format!("{}%", (moon.illumination * 100.0).round() as u32);
    let style = ThemeStyle::default_style();

    out.push_str(&format!(
        "Location: {} | Date: {} ({} / {})\n\n",
        mawaqit.location_name,
        mawaqit.date,
        taqwim_date.format(),
        if now_local >= today_maghrib { "Eve of next day" } else { "Day" }
    ));

    let tz_shift = Duration::hours(params.timezone_offset_hours as i64);

    let total_items = mawaqit.schedule.len();
    for (idx, miqat) in mawaqit.schedule.iter().enumerate() {
        let is_last = idx + 1 == total_items;
        let shifted_time = miqat.time + tz_shift;
        let line = style.format_row(
            miqat.name,
            &shifted_time.format("%H:%M").to_string(),
            miqat.status,
            is_last,
        );
        out.push_str(&line);
        out.push('\n');
    }


    let shifted_next_time = mawaqit.next_prayer_time + tz_shift;
    out.push_str(&format!(
        "\nNext: {} at {} (-{})\n\n",
        mawaqit.next_prayer_name,
        shifted_next_time.format("%H:%M"),
        countdown_str
    ));

    out.push_str("Observances (Munasabat):\n");
    out.push_str(&format!("  Midnight (Nisf):     {}\n", (night.midnight + tz_shift).format("%H:%M")));
    out.push_str(&format!(
        "  Last Third of Night: {} - {}{}\n",
        (night.last_third.start + tz_shift).format("%H:%M"),
        (night.last_third.end + tz_shift).format("%H:%M"),
        if night.is_currently_last_third { " (ACTIVE NOW)" } else { "" }
    ));
    out.push_str(&format!(
        "  Fasting & Suhoor:    {:02}h {:02}m (Imsak: {}, Fajr: {}, Iftar: {})\n",
        fasting.fasting_duration.num_minutes() / 60,
        fasting.fasting_duration.num_minutes() % 60,
        (fasting.imsak_time + tz_shift).format("%H:%M"),
        (fasting.suhoor_cutoff + tz_shift).format("%H:%M"),
        (fasting.iftar_time + tz_shift).format("%H:%M")
    ));
    out.push_str(&format!(
        "  Duha Window:         {} - {}{}\n",
        (prohibited.duha_start + tz_shift).format("%H:%M"),
        (prohibited.duha_end + tz_shift).format("%H:%M"),
        if prohibited.is_currently_duha { " (ACTIVE NOW)" } else { "" }
    ));


    out.push_str("\nCelestial & Calendar:\n");
    out.push_str(&format!("  Calendar: {}\n", taqwim_date.format()));
    out.push_str(&format!("  Moon:     {} {} ({})\n", moon_symbol, moon.phase_name, moon_pct));
    out.push_str(&format!("  Sun:      {:.1}° {}\n\n", sun.altitude, sun.state_name));

    if params.is_terminal {
        out.push_str("\x1b[90mTip: curl mawaqit.rahmanr.com/:help for endpoint options\x1b[0m\n");
    } else {
        out.push_str("Tip: curl mawaqit.rahmanr.com/:help for endpoint options\n");
    }

    out
}

fn render_edge_help() -> String {
    let mut h = String::new();
    h.push_str("mawaqit.rahmanr.com (مواقيت) - Universal Terminal Prayer Service\n");
    h.push_str("Inspired by wttr.in | Designed for curl, httpie, tmux, and status bars\n\n");
    h.push_str("USAGE:\n");
    h.push_str("  curl mawaqit.rahmanr.com                   # Auto-geolocated by Cloudflare edge IP\n");
    h.push_str("  curl mawaqit.rahmanr.com/London            # Named city lookup\n");
    h.push_str("  curl mawaqit.rahmanr.com/1.35,103.82       # Latitude, Longitude coordinates\n");
    h.push_str("  curl mawaqit.rahmanr.com/:help             # This manual\n\n");
    h.push_str("SPECIAL ENDPOINTS:\n");
    h.push_str("  curl mawaqit.rahmanr.com/fasting           # Imsak, Suhoor cutoff & fasting span\n");
    h.push_str("  curl mawaqit.rahmanr.com/night             # Midnight (Nisf) & Tahajjud window\n");
    h.push_str("  curl mawaqit.rahmanr.com/white-days        # Ayyam al-Bid (13-15) monthly dates\n");
    h.push_str("  curl mawaqit.rahmanr.com/prohibited        # Awqat al-Nahy & Duha window\n");
    h.push_str("  curl mawaqit.rahmanr.com/observances       # Canonical Islamic annual sacred days\n\n");
    h.push_str("QUERY PARAMETERS:\n");
    h.push_str("  ?format=...                                # Custom template string\n");
    h.push_str("  ?method=<singapore|mwl|isna|...>           # Jurisprudential calculation method\n");
    h.push_str("  ?madhab=<shafi|hanafi>                     # Asr shadow factor\n\n");
    h.push_str("EXAMPLE STATUS BAR / TMUX INTEGRATION:\n");
    h.push_str("  curl -s \"mawaqit.rahmanr.com?format=%next+in+%remaining+%moon_symbol\"\n");
    h.push_str("  -> Asr in 01h 45m ◑\n");
    h
}
