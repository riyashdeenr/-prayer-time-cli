use crate::falak::{MoonGlyphStyle, MoonPhase, SunPosition};
use crate::munasabat::{FastingInfo, FridayIstijabahInfo, NightBreakdown, ProhibitedTimesInfo};
use crate::muwaqqit::MawaqitDaily;
use crate::taqwim::{TaqwimDate, WhiteDaysInfo};
use salah::prelude::Prayer;

/// The Template & Presentation Engine (Domain: bayan / بيان)
pub struct Bayan;

impl Bayan {
    /// Format string template replacement engine supporting wttr.in-style tokens
    pub fn format(
        template: &str,
        mawaqit: &MawaqitDaily,
        moon: &MoonPhase,
        sun: &SunPosition,
        moon_style: MoonGlyphStyle,
        taqwim: &TaqwimDate,
        night: &NightBreakdown,
        white_days: &WhiteDaysInfo,
        fasting: &FastingInfo,
        prohibited: &ProhibitedTimesInfo,
        friday: &FridayIstijabahInfo,
    ) -> String {
        let total_secs = mawaqit.time_until_next.num_seconds();
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;

        let remaining_hms = format!("{:02}:{:02}:{:02}", hours, mins, secs);
        let remaining_hm = format!("{:02}h {:02}m", hours, mins);

        // Extract individual prayer times
        let fajr_time = Self::find_time(mawaqit, Prayer::Fajr);
        let sunrise_time = Self::find_time(mawaqit, Prayer::Sunrise);
        let dhuhr_time = Self::find_time(mawaqit, Prayer::Dhuhr);
        let asr_time = Self::find_time(mawaqit, Prayer::Asr);
        let maghrib_time = Self::find_time(mawaqit, Prayer::Maghrib);
        let isha_time = Self::find_time(mawaqit, Prayer::Isha);

        let moon_pct = format!("{}%", (moon.illumination * 100.0).round() as u32);
        let moon_symbol = moon.glyph(moon_style);
        let sun_alt = format!("{:.1}°", sun.altitude);
        let hijri_full = taqwim.format();
        let hijri_short = taqwim.format_short();
        let hijri_day = taqwim.day.to_string();
        let hijri_month = taqwim.month_name;
        let hijri_year = taqwim.year.to_string();
        let hijri_weekday = taqwim.weekday_name;
        let hijri_offset = format!("{:+}", taqwim.applied_offset);

        let midnight_time = night.midnight.format("%H:%M").to_string();
        let last_third_start = night.last_third.start.format("%H:%M").to_string();
        let last_third_range = format!(
            "{}-{}",
            night.last_third.start.format("%H:%M"),
            night.last_third.end.format("%H:%M")
        );

        let white_days_summary = format!(
            "13-15 {} ({} {} - {} {})",
            white_days.month_name,
            white_days.day13_weekday,
            white_days.day13_gregorian.format("%d %b"),
            white_days.day15_weekday,
            white_days.day15_gregorian.format("%d %b")
        );

        let imsak_str = fasting.imsak_time.format("%H:%M").to_string();
        let suhoor_cutoff_str = fasting.suhoor_cutoff.format("%H:%M").to_string();
        let iftar_str = fasting.iftar_time.format("%H:%M").to_string();
        let f_mins = fasting.fasting_duration.num_minutes();
        let fasting_duration_str = format!("{:02}h {:02}m", f_mins / 60, f_mins % 60);

        let duha_range = format!("{}-{}", prohibited.duha_start.format("%H:%M"), prohibited.duha_end.format("%H:%M"));
        let duha_start_str = prohibited.duha_start.format("%H:%M").to_string();
        let zawal_start_str = prohibited.zawal_prohibited_start.format("%H:%M").to_string();
        let istijabah_range = if friday.is_friday {
            format!("{}-{}", friday.window_start.format("%H:%M"), friday.window_end.format("%H:%M"))
        } else {
            "Fridays only".to_string()
        };

        let mut output = template.to_string();

        // 1. Longest compound tokens first
        output = output.replace("%fasting_duration", &fasting_duration_str);
        output = output.replace("%suhoor_cutoff", &suhoor_cutoff_str);
        output = output.replace("%suhoor", &suhoor_cutoff_str);
        output = output.replace("%imsak", &imsak_str);
        output = output.replace("%iftar", &iftar_str);

        output = output.replace("%istijabah", &istijabah_range);
        output = output.replace("%duha_range", &duha_range);
        output = output.replace("%duha", &duha_start_str);
        output = output.replace("%zawal", &zawal_start_str);

        output = output.replace("%white_days", &white_days_summary);
        output = output.replace("%last_third_range", &last_third_range);
        output = output.replace("%last_third", &last_third_start);
        output = output.replace("%midnight", &midnight_time);

        output = output.replace("%hijri_offset", &hijri_offset);
        output = output.replace("%hijri_weekday", hijri_weekday);
        output = output.replace("%hijri_month", hijri_month);
        output = output.replace("%hijri_year", &hijri_year);
        output = output.replace("%hijri_short", &hijri_short);
        output = output.replace("%hijri_day", &hijri_day);
        output = output.replace("%hijri", &hijri_full);
        output = output.replace("%taqwim", &hijri_full);

        output = output.replace("%moon_symbol", moon_symbol);
        output = output.replace("%moon_name", moon.phase_name);
        output = output.replace("%moon_pct", &moon_pct);
        output = output.replace("%moon", moon_symbol);


        output = output.replace("%sun_state", sun.state_name);
        output = output.replace("%sun_dir", sun.compass_direction);
        output = output.replace("%sun_alt", &sun_alt);

        output = output.replace("%remaining_hm", &remaining_hm);
        output = output.replace("%remaining", &remaining_hms);
        output = output.replace("%next_time", &mawaqit.next_prayer_time.format("%H:%M").to_string());
        output = output.replace("%next", &mawaqit.next_prayer_name);
        output = output.replace("%current", &mawaqit.current_name);

        output = output.replace("%location", &mawaqit.location_name);
        output = output.replace("%city", &mawaqit.location_name);
        output = output.replace("%date", &mawaqit.date.to_string());

        output = output.replace("%sunrise", &sunrise_time);
        output = output.replace("%maghrib", &maghrib_time);
        output = output.replace("%fajr", &fajr_time);
        output = output.replace("%dhuhr", &dhuhr_time);
        output = output.replace("%asr", &asr_time);
        output = output.replace("%isha", &isha_time);

        // 2. Single-letter short tokens last
        output = output.replace("%f", &fajr_time);
        output = output.replace("%d", &dhuhr_time);
        output = output.replace("%a", &asr_time);
        output = output.replace("%m", &maghrib_time);
        output = output.replace("%i", &isha_time);

        output
    }

    fn find_time(mawaqit: &MawaqitDaily, prayer: Prayer) -> String {
        mawaqit
            .schedule
            .iter()
            .find(|m| m.prayer == prayer)
            .map(|m| m.time.format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".to_string())
    }
}
