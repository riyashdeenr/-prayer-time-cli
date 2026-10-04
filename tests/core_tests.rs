use chrono::{Duration, TimeZone};
use prayer_time_cli::falak::{Falak, MoonGlyphStyle};
use prayer_time_cli::munasabat::{Munasabat, NightCalculationBasis};
use prayer_time_cli::muwaqqit::Muwaqqit;
use prayer_time_cli::taqwim::Taqwim;
use prayer_time_cli::tawqit::Tawqit;
use salah::prelude::{Madhab, Method};
use std::collections::HashMap;

#[test]
fn test_singapore_default_tawqit() {
    let tawqit = Tawqit::singapore();
    assert_eq!(tawqit.location_name, "Singapore");
    assert_eq!(tawqit.method, Method::Singapore);
    assert_eq!(tawqit.madhab, Madhab::Shafi);
}

#[test]
fn test_city_directory_lookup() {
    let (name, lat, lon, offset) = Tawqit::lookup_city("London").expect("City should exist");
    assert_eq!(name, "London");
    assert!((lat - 51.5074).abs() < 0.001);
    assert!((lon - (-0.1278)).abs() < 0.001);
    assert_eq!(offset, 1);

    let (makkah_name, _, _, m_offset) = Tawqit::lookup_city("Makkah").expect("Makkah should exist");
    assert_eq!(makkah_name, "Makkah");
    assert_eq!(m_offset, 3);
}

#[test]
fn test_muwaqqit_prayer_calculation_order() {
    let tawqit = Tawqit::singapore();
    let muwaqqit = Muwaqqit::new(tawqit);

    let test_time = chrono::Local.with_ymd_and_hms(2026, 10, 4, 12, 0, 0).unwrap();
    let mawaqit = muwaqqit.calculate(test_time).expect("Schedule calculation should succeed");

    assert_eq!(mawaqit.schedule.len(), 6);
    let names: Vec<&str> = mawaqit.schedule.iter().map(|m| m.name).collect();
    assert_eq!(names, vec!["Fajr", "Sunrise", "Dhuhr", "Asr", "Maghrib", "Isha"]);

    for i in 0..mawaqit.schedule.len() - 1 {
        assert!(mawaqit.schedule[i].time < mawaqit.schedule[i + 1].time);
    }
}

#[test]
fn test_muwaqqit_pre_fajr_boundary() {
    // 04:30 AM is before Fajr (~05:35 AM in Singapore)
    let tawqit = Tawqit::singapore();
    let muwaqqit = Muwaqqit::new(tawqit);

    let pre_fajr_time = chrono::Local.with_ymd_and_hms(2026, 10, 4, 4, 30, 0).unwrap();
    let mawaqit = muwaqqit.calculate(pre_fajr_time).expect("Pre-fajr calculation should succeed");

    assert_eq!(mawaqit.current_name, "Qiyam / Night");
    assert_eq!(mawaqit.next_prayer_name, "Fajr");
    assert!(mawaqit.time_until_next > Duration::zero());
}

#[test]
fn test_muwaqqit_post_isha_boundary() {
    // 22:30 PM is after Isha (~20:06 PM in Singapore)
    let tawqit = Tawqit::singapore();
    let muwaqqit = Muwaqqit::new(tawqit);

    let post_isha_time = chrono::Local.with_ymd_and_hms(2026, 10, 4, 22, 30, 0).unwrap();
    let mawaqit = muwaqqit.calculate(post_isha_time).expect("Post-isha calculation should succeed");

    assert_eq!(mawaqit.current_name, "Isha");
    assert_eq!(mawaqit.next_prayer_name, "Fajr");
    // Next Fajr should be tomorrow's Fajr
    assert!(mawaqit.next_prayer_time.date_naive() > post_isha_time.date_naive());
}

#[test]
fn test_falak_moon_illumination_bounds() {
    let now_utc = chrono::Utc::now();
    let moon = Falak::calculate_moon(now_utc);

    assert!(moon.illumination >= 0.0 && moon.illumination <= 1.0);
    assert!(moon.age_days >= 0.0 && moon.age_days <= 30.0);
    assert!(!moon.phase_name.is_empty());

    let glyph = moon.glyph(MoonGlyphStyle::Geometric);
    assert!(!glyph.is_empty());
}

#[test]
fn test_falak_solar_elevation() {
    let now_utc = chrono::Utc::now();
    let sun = Falak::calculate_sun(now_utc, 1.3521, 103.8198);

    assert!(sun.altitude >= -90.0 && sun.altitude <= 90.0);
    assert!(sun.azimuth >= 0.0 && sun.azimuth <= 360.0);
    assert!(!sun.compass_direction.is_empty());
    assert!(!sun.state_name.is_empty());
}

#[test]
fn test_taqwim_maghrib_day_transition() {
    let adjustments = HashMap::new();

    let maghrib_time = chrono::Local.with_ymd_and_hms(2026, 10, 4, 18, 57, 0).unwrap();
    let before_maghrib = chrono::Local.with_ymd_and_hms(2026, 10, 4, 17, 0, 0).unwrap();
    let after_maghrib = chrono::Local.with_ymd_and_hms(2026, 10, 4, 19, 30, 0).unwrap();

    let date_before = Taqwim::calculate(before_maghrib, maghrib_time, 0, &adjustments);
    let date_after = Taqwim::calculate(after_maghrib, maghrib_time, 0, &adjustments);

    // After Maghrib, the Islamic day should increment by 1
    assert_eq!(date_after.day, date_before.day + 1);
}

#[test]
fn test_taqwim_month_specific_ledger_override() {
    let maghrib_time = chrono::Local.with_ymd_and_hms(2026, 10, 4, 18, 57, 0).unwrap();
    let noon_time = chrono::Local.with_ymd_and_hms(2026, 10, 4, 12, 0, 0).unwrap();

    let mut adjustments = HashMap::new();
    let base_date = Taqwim::calculate(noon_time, maghrib_time, 0, &adjustments);

    // Override the specific month by +1
    let month_key = base_date.month_key();
    adjustments.insert(month_key, 1);

    let adjusted_date = Taqwim::calculate(noon_time, maghrib_time, 0, &adjustments);
    assert_eq!(adjusted_date.day, base_date.day + 1);
    assert_eq!(adjusted_date.applied_offset, 1);
}

#[test]
fn test_munasabat_night_divisions() {
    let now = chrono::Local.with_ymd_and_hms(2026, 10, 4, 21, 0, 0).unwrap();
    let maghrib = chrono::Local.with_ymd_and_hms(2026, 10, 4, 18, 57, 0).unwrap();
    let isha = chrono::Local.with_ymd_and_hms(2026, 10, 4, 20, 06, 0).unwrap();
    let tomorrow_fajr = chrono::Local.with_ymd_and_hms(2026, 10, 5, 5, 34, 0).unwrap();

    let night = Munasabat::calculate_night(now, maghrib, isha, tomorrow_fajr, NightCalculationBasis::SunsetToDawn);

    assert_eq!(night.night_start, maghrib);
    assert_eq!(night.night_end, tomorrow_fajr);
    assert_eq!(night.first_third.start, maghrib);
    assert_eq!(night.first_third.end, night.second_third.start);
    assert_eq!(night.second_third.end, night.last_third.start);
    assert_eq!(night.last_third.end, tomorrow_fajr);

    assert!(night.midnight > maghrib && night.midnight < tomorrow_fajr);
}

#[test]
fn test_munasabat_fasting_schedule_and_imsak() {
    let now = chrono::Local.with_ymd_and_hms(2026, 10, 4, 12, 0, 0).unwrap();
    let fajr = chrono::Local.with_ymd_and_hms(2026, 10, 4, 5, 35, 0).unwrap();
    let maghrib = chrono::Local.with_ymd_and_hms(2026, 10, 4, 18, 57, 0).unwrap();
    let tomorrow_fajr = chrono::Local.with_ymd_and_hms(2026, 10, 5, 5, 34, 0).unwrap();

    let fasting = Munasabat::calculate_fasting(now, fajr, maghrib, tomorrow_fajr, 10);

    // Imsak should be exactly 10 minutes before Fajr
    assert_eq!(fasting.imsak_time, fajr - Duration::minutes(10));
    assert_eq!(fasting.suhoor_cutoff, fajr);
    assert_eq!(fasting.iftar_time, maghrib);

    // Total fasting duration = Maghrib - Fajr (18:57 - 05:35 = 13h 22m)
    assert_eq!(fasting.fasting_duration.num_minutes(), 13 * 60 + 22);

    // At 12:00 PM, user is actively fasting
    assert_eq!(fasting.status, prayer_time_cli::munasabat::FastingStatus::FastingActive);
}

#[test]
fn test_munasabat_prohibited_times() {
    let now = chrono::Local.with_ymd_and_hms(2026, 10, 4, 10, 0, 0).unwrap();
    let sunrise = chrono::Local.with_ymd_and_hms(2026, 10, 4, 6, 52, 0).unwrap();
    let dhuhr = chrono::Local.with_ymd_and_hms(2026, 10, 4, 12, 56, 0).unwrap();
    let maghrib = chrono::Local.with_ymd_and_hms(2026, 10, 4, 18, 57, 0).unwrap();

    let prohibited = Munasabat::calculate_prohibited_times(now, sunrise, dhuhr, maghrib);

    assert_eq!(prohibited.sunrise_prohibited_start, sunrise);
    assert_eq!(prohibited.sunrise_prohibited_end, sunrise + Duration::minutes(15));
    assert_eq!(prohibited.zawal_prohibited_start, dhuhr - Duration::minutes(10));
    assert_eq!(prohibited.zawal_prohibited_end, dhuhr);
    assert_eq!(prohibited.sunset_prohibited_start, maghrib - Duration::minutes(15));
    assert_eq!(prohibited.sunset_prohibited_end, maghrib);

    // At 10:00 AM, user is in permissible Duha forenoon window
    assert!(prohibited.is_currently_duha);
    assert!(!prohibited.is_currently_prohibited);
}
