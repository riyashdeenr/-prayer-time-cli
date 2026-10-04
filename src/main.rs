use std::collections::HashMap;
use std::env;
use chrono::{Duration, Local, Utc};
use prayer_time_cli::{
    bayan::Bayan,
    config::{ConfigManager, SavedConfig},
    falak::{Falak, MoonGlyphStyle},
    miqat,
    munasabat::{self, Munasabat, NightCalculationBasis},
    muwaqqit::Muwaqqit,
    taqwim::{self, Taqwim},
    tawqit::Tawqit,
    theme::ThemeStyle,
};
use salah::prelude::{Coordinates, Prayer, PrayerSchedule};

fn format_duration(total_secs: i64) -> String {
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let is_help = args.iter().any(|arg| arg == "--help" || arg == "-h");
    let is_verbose = args.iter().any(|arg| arg == "--verbose" || arg == "-v");
    let is_save = args.iter().any(|arg| arg == "--save");
    let is_night_breakdown = args.iter().any(|arg| arg == "--night");
    let is_white_days = args.iter().any(|arg| arg == "--white-days");
    let is_fasting = args.iter().any(|arg| arg == "--fasting" || arg == "--suhoor" || arg == "--imsak");
    let is_prohibited = args.iter().any(|arg| arg == "--prohibited" || arg == "--duha" || arg == "--nahy");
    let is_annual = args.iter().any(|arg| arg == "--observances" || arg == "--annual" || arg == "--events");

    if is_help {
        println!("mawaqit (مواقيت) - Universal Terminal Prayer & Celestial Engine");
        println!("Usage: mawaqit [OPTIONS]\n");
        println!("GENERAL OPTIONS:");
        println!("  -v, --verbose               Full multi-section schedule, astronomical & observances table");
        println!("  -f, --format <STR>          Custom wttr.in-style format template (e.g. \"%next in %remaining\")");
        println!("  --city, --location <NAME>   Select city by name (e.g. Singapore, London, Makkah, Tokyo)");
        println!("  --lat <F64>, --lon <F64>    Custom geographic coordinates (e.g. --lat 1.35 --lon 103.82)");
        println!("  --save                      Persist active flags and location to config.toml\n");
        println!("CALCULATION & METHOD OPTIONS:");
        println!("  --method <NAME>             Calculation method: singapore, mwl, ummalqura, isna, egyptian,");
        println!("                              karachi, tehran, turkey, dubai, qatar, kuwait, moonsighting");
        println!("  --madhab <shafi|hanafi>     Asr shadow ratio (default: shafi [1x], hanafi [2x])");
        println!("  --fajr-angle <DEG>          Custom Fajr twilight depression angle");
        println!("  --isha-angle <DEG>          Custom Isha twilight depression angle\n");
        println!("OBSERVANCES & CELESTIAL OPTIONS:");
        println!("  --night                     Detailed breakdown of night thirds, midnight & Tahajjud window");
        println!("  --night-basis <sunset|isha> Juristic basis for night bounds (default: sunset [Maghrib to Fajr])");
        println!("  --fasting, --suhoor         Dedicated Imsak, Suhoor cutoff, Iftar & fasting duration schedule");
        println!("  --imsak-mins <N>            Precautionary Imsak buffer in minutes (default: 10)");
        println!("  --compare-fasting <CITIES>  Compare fasting duration across global cities (e.g. London,Tokyo)");
        println!("  --white-days                White Days (Ayyam al-Bid 13-15) and Month 1st verification");
        println!("  --convert-hijri <SPEC>      Convert Hijri date to Gregorian (e.g. 13 or 1448-05-13)");
        println!("  --prohibited, --duha        The 3 prohibited times (Awqat al-Nahy) and permissible Duha window");
        println!("  --observances, --annual     Major canonical Islamic annual stations and sacred days\n");
        println!("TAQWIM (HIJRI CALENDAR) ADJUSTMENTS:");
        println!("  --taqwim-adjust <+/-N>      Apply offset to target month (e.g. +1 or -1) with audit logging");
        println!("  --taqwim-month <YYYY-MM>    Target Hijri month key for adjustment (default: current month)");
        println!("  --taqwim-offset <+/-N>      Global fallback Hijri day offset\n");
        println!("UI & STYLING OPTIONS:");
        println!("  --style, --theme <NAME>     Row style: dots (default), timeline, blocks, radio, minimal");
        println!("  --moon <STYLE>              Moon glyph style: geometric (default), lunar, classic, text");
        println!("  -h, --help                  Print this help guidance\n");
        println!("TEMPLATE TOKENS (-f / --format):");
        println!("  %location, %date, %current, %next, %next_time, %remaining, %remaining_hm");
        println!("  %fajr (%f), %sunrise, %dhuhr (%d), %asr (%a), %maghrib (%m), %isha (%i)");
        println!("  %moon, %moon_pct, %moon_name, %sun_alt, %sun_state, %sun_dir");
        println!("  %hijri, %hijri_short, %hijri_day, %hijri_month, %hijri_year, %hijri_weekday, %hijri_offset");
        println!("  %midnight, %last_third, %last_third_range, %white_days");
        println!("  %imsak, %suhoor, %iftar, %fasting_duration, %duha, %duha_range, %zawal, %istijabah");
        return;
    }

    // Fasting comparison flag: --compare-fasting <city1,city2,...> or --compare <city1> <city2>
    let compare_fasting_arg = args.iter().position(|a| a == "--compare-fasting" || a == "--compare")
        .and_then(|i| args.get(i + 1).cloned());

    // Hijri to Gregorian date converter flag: --convert-hijri <D> [M] [Y] or <Y-M-D>
    let convert_hijri_arg = args.iter().position(|a| a == "--convert-hijri" || a == "--convert")
        .and_then(|i| args.get(i + 1).cloned());

    // Format string template support (--format "%next in %remaining")
    let custom_format = args.iter().position(|a| a == "--format" || a == "-f")
        .and_then(|i| args.get(i + 1).cloned());

    // 1. Load saved config if available
    let saved = ConfigManager::load();

    // 2. Parse CLI flags (overriding saved config)
    let city_cli = args.iter().position(|a| a == "--city" || a == "--location")
        .and_then(|i| args.get(i + 1).cloned());

    let mut location_name = city_cli.clone()
        .or_else(|| saved.as_ref().and_then(|s| s.location_name.clone()))
        .unwrap_or_else(|| "Singapore".to_string());

    // If city name is recognized in our global directory, auto-populate lat/lon
    let (city_lat, city_lon) = city_cli.as_deref()
        .and_then(Tawqit::lookup_city)
        .map(|(name, lat, lon, _)| {
            location_name = name.to_string();
            (Some(lat), Some(lon))
        })
        .unwrap_or((None, None));

    let latitude: f64 = args.iter().position(|a| a == "--lat" || a == "--latitude")
        .and_then(|i| args.get(i + 1).and_then(|v| v.parse().ok()))
        .or(city_lat)
        .or_else(|| saved.as_ref().and_then(|s| s.latitude))
        .unwrap_or(1.3521);

    let longitude: f64 = args.iter().position(|a| a == "--lon" || a == "--longitude")
        .and_then(|i| args.get(i + 1).and_then(|v| v.parse().ok()))
        .or(city_lon)
        .or_else(|| saved.as_ref().and_then(|s| s.longitude))
        .unwrap_or(103.8198);

    // Calculation method: CLI flag -> Saved config -> Regional auto-detection based on lat/lon
    let method = args.iter().position(|a| a == "--method")
        .and_then(|i| args.get(i + 1).and_then(|v| Tawqit::parse_method(v)))
        .or_else(|| saved.as_ref().and_then(|s| s.method.as_deref().and_then(Tawqit::parse_method)))
        .unwrap_or_else(|| Tawqit::resolve_regional_method(latitude, longitude));

    // Madhab: CLI flag -> Saved config -> Default Shafi (1x shadow)
    let madhab = args.iter().position(|a| a == "--madhab")
        .and_then(|i| args.get(i + 1).map(|v| Tawqit::parse_madhab(v)))
        .or_else(|| saved.as_ref().and_then(|s| s.madhab.as_deref().map(Tawqit::parse_madhab)))
        .unwrap_or(salah::prelude::Madhab::Shafi);

    // Custom twilight angles
    let fajr_angle: Option<f64> = args.iter().position(|a| a == "--fajr-angle")
        .and_then(|i| args.get(i + 1).and_then(|v| v.parse().ok()))
        .or_else(|| saved.as_ref().and_then(|s| s.fajr_angle));

    let isha_angle: Option<f64> = args.iter().position(|a| a == "--isha-angle")
        .and_then(|i| args.get(i + 1).and_then(|v| v.parse().ok()))
        .or_else(|| saved.as_ref().and_then(|s| s.isha_angle));

    // Night Calculation Basis: CLI flag -> Saved config -> Default SunsetToDawn (Maghrib to Fajr)
    let night_basis = args.iter().position(|a| a == "--night-basis" || a == "--night-method")
        .and_then(|i| args.get(i + 1).and_then(|v| NightCalculationBasis::parse(v)))
        .or_else(|| saved.as_ref().and_then(|s| s.night_basis.as_deref().and_then(NightCalculationBasis::parse)))
        .unwrap_or_else(NightCalculationBasis::default_basis);

    // Imsak Buffer (minutes before Fajr): CLI flag -> Saved config -> Default 10 mins
    let imsak_buffer_minutes: i64 = args.iter().position(|a| a == "--imsak-mins" || a == "--imsak-buffer" || a == "--imsak")
        .and_then(|i| args.get(i + 1).and_then(|v| v.parse().ok()))
        .or_else(|| saved.as_ref().and_then(|s| s.imsak_buffer_minutes))
        .unwrap_or(10);

    // UI Styles
    let style = args.iter().position(|a| a == "--style" || a == "--theme")
        .and_then(|i| args.get(i + 1).and_then(|v| ThemeStyle::parse(v)))
        .or_else(|| saved.as_ref().and_then(|s| s.style.as_deref().and_then(ThemeStyle::parse)))
        .unwrap_or_else(ThemeStyle::default_style);

    let moon_style = args.iter().position(|a| a == "--moon" || a == "--moon-style")
        .and_then(|i| args.get(i + 1).and_then(|v| MoonGlyphStyle::parse(v)))
        .or_else(|| saved.as_ref().and_then(|s| s.moon_style.as_deref().and_then(MoonGlyphStyle::parse)))
        .unwrap_or_else(MoonGlyphStyle::default_style);

    // Taqwim adjustments
    let taqwim_default_offset = args.iter().position(|a| a == "--taqwim-offset" || a == "--hijri-offset")
        .and_then(|i| args.get(i + 1).and_then(|v| v.parse::<i32>().ok()))
        .unwrap_or_else(|| saved.as_ref().map(|s| s.taqwim_default_offset).unwrap_or(0));

    let mut taqwim_adjustments: HashMap<String, i32> = saved
        .as_ref()
        .map(|s| s.taqwim_adjustments.clone())
        .unwrap_or_default();

    // Check if user is performing an adjustment via CLI (--taqwim-adjust +1 [--taqwim-month 1448-03])
    let taqwim_adjust_cli = args.iter().position(|a| a == "--taqwim-adjust" || a == "--hijri-adjust")
        .and_then(|i| args.get(i + 1).and_then(|v| v.parse::<i32>().ok()));

    let taqwim_month_target = args.iter().position(|a| a == "--taqwim-month" || a == "--hijri-month")
        .and_then(|i| args.get(i + 1).cloned());

    // If user passed custom coordinates without city, display the coords
    if (latitude != 1.3521 || longitude != 103.8198) && location_name == "Singapore" {
        location_name = format!("{:.2}°, {:.2}°", latitude, longitude);
    }

    // 3. Build Tawqit configuration
    let tawqit = Tawqit {
        location_name: location_name.clone(),
        coordinates: Coordinates::new(latitude, longitude),
        method,
        madhab,
        custom_fajr_angle: fajr_angle,
        custom_isha_angle: isha_angle,
    };

    let muwaqqit = Muwaqqit::new(tawqit.clone());

    let now_local = Local::now();
    let now_utc = Utc::now();

    // Celestial Calculations (Moon & Sun)
    let moon = Falak::calculate_moon(now_utc);
    let moon_pct = (moon.illumination * 100.0).round() as u32;
    let moon_symbol = moon.glyph(moon_style);

    let sun = Falak::calculate_sun(now_utc, tawqit.coordinates.latitude, tawqit.coordinates.longitude);

    match muwaqqit.calculate(now_local) {
        Ok(mawaqit) => {
            // Find today's Maghrib and Isha
            let today_maghrib = mawaqit
                .schedule
                .iter()
                .find(|m| m.prayer == Prayer::Maghrib)
                .map(|m| m.time)
                .unwrap_or(now_local);

            let today_isha = mawaqit
                .schedule
                .iter()
                .find(|m| m.prayer == Prayer::Isha)
                .map(|m| m.time)
                .unwrap_or(now_local);

            // Compute tomorrow's Fajr for exact night bounds
            let tomorrow = now_local.date_naive() + Duration::days(1);
            let next_day_schedule = PrayerSchedule::new()
                .on(tomorrow)
                .for_location(tawqit.coordinates)
                .with_configuration(tawqit.to_salah_params())
                .calculate()
                .expect("Failed to calculate tomorrow schedule");
            let tomorrow_fajr: chrono::DateTime<Local> = chrono::DateTime::from(next_day_schedule.time(Prayer::Fajr));

            // Calculate Munasabat (Night Divisions)
            let night = Munasabat::calculate_night(
                now_local,
                today_maghrib,
                today_isha,
                tomorrow_fajr,
                night_basis,
            );

            // Calculate Taqwim (Hijri Date)
            let mut taqwim_date = Taqwim::calculate(
                now_local,
                today_maghrib,
                taqwim_default_offset,
                &taqwim_adjustments,
            );

            // Handle CLI monthly adjustment request
            if let Some(adjust_offset) = taqwim_adjust_cli {
                let month_key = taqwim_month_target.unwrap_or_else(|| taqwim_date.month_key());
                taqwim_adjustments.insert(month_key.clone(), adjust_offset);
                let _ = Taqwim::record_audit_log(&month_key, adjust_offset, Some("User CLI adjustment"));

                // Recalculate with new adjustment
                taqwim_date = Taqwim::calculate(
                    now_local,
                    today_maghrib,
                    taqwim_default_offset,
                    &taqwim_adjustments,
                );

                println!("Applied Taqwim adjustment: Month {} set to {:+}", month_key, adjust_offset);
                println!("Audit logged to: {}\n", Taqwim::log_path().display());
            }

            // Handle --save flag if requested
            if is_save || taqwim_adjust_cli.is_some() {
                let to_save = SavedConfig {
                    location_name: Some(location_name.clone()),
                    latitude: Some(latitude),
                    longitude: Some(longitude),
                    method: Some(format!("{:?}", method).to_lowercase()),
                    madhab: Some(format!("{:?}", madhab).to_lowercase()),
                    fajr_angle,
                    isha_angle,
                    style: Some(format!("{:?}", style).to_lowercase()),
                    moon_style: Some(format!("{:?}", moon_style).to_lowercase()),
                    taqwim_default_offset,
                    taqwim_adjustments: taqwim_adjustments.clone(),
                    night_basis: Some(match night_basis {
                        NightCalculationBasis::SunsetToDawn => "sunset".to_string(),
                        NightCalculationBasis::IshaToDawn => "isha".to_string(),
                    }),
                    imsak_buffer_minutes: Some(imsak_buffer_minutes),
                };
                match ConfigManager::save(&to_save) {
                    Ok(p) => {
                        if is_save {
                            println!("Saved configuration to: {}\n", p.display());
                        }
                    }
                    Err(e) => eprintln!("Error saving configuration: {}\n", e),
                }
            }

            // Dedicated --night flag output
            if is_night_breakdown {
                let total_m = night.total_duration.num_minutes();
                let dur_h = total_m / 60;
                let dur_m = total_m % 60;

                println!("============================================================");
                println!("  munasabat - Night Breakdown ({})", night.basis.name());
                println!("============================================================");
                println!(
                    "Window:   {} -> {} (Duration: {:02}h {:02}m)",
                    night.night_start.format("%H:%M"),
                    night.night_end.format("%H:%M"),
                    dur_h,
                    dur_m
                );
                println!("Midnight: {} (Nisf al-Layl)\n", night.midnight.format("%H:%M"));

                println!("Segments:");
                println!(
                    "  1st Third: {} - {}",
                    night.first_third.start.format("%H:%M"),
                    night.first_third.end.format("%H:%M")
                );
                println!(
                    "  2nd Third: {} - {}",
                    night.second_third.start.format("%H:%M"),
                    night.second_third.end.format("%H:%M")
                );
                let active_marker = if night.is_currently_last_third { "  <-- ACTIVE NOW (Tahajjud / Du'a)" } else { "" };
                println!(
                    "  3rd Third: {} - {}{}",
                    night.last_third.start.format("%H:%M"),
                    night.last_third.end.format("%H:%M"),
                    active_marker
                );

                if let Some(until) = night.time_until_last_third {
                    println!("\n⏳ Last Third starts in: {}", format_duration(until.num_seconds()));
                } else if night.is_currently_last_third {
                    let left = night.last_third.end - now_local;
                    println!("\n✨ Currently in the Last Third! Ends in: {}", format_duration(left.num_seconds()));
                }

                println!("------------------------------------------------------------");
                return;
            }

            // Calculate Month Start & White Days (Munasabat)
            let month_start = Taqwim::find_month_start(
                taqwim_date.year,
                taqwim_date.month,
                now_local.date_naive(),
                taqwim_default_offset,
                &taqwim_adjustments,
            );

            let white_days = Taqwim::calculate_white_days(
                &taqwim_date,
                now_local.date_naive(),
                taqwim_default_offset,
                &taqwim_adjustments,
            );

            // Handle --convert-hijri <D> [M] [Y] or <Y-M-D>
            if let Some(target_spec) = convert_hijri_arg {
                let (target_year, target_month, target_day) = if target_spec.contains('-') {
                    let parts: Vec<&str> = target_spec.split('-').collect();
                    if parts.len() == 3 {
                        let y = parts[0].parse::<i32>().unwrap_or(taqwim_date.year);
                        let m = parts[1].parse::<u32>().unwrap_or(taqwim_date.month);
                        let d = parts[2].parse::<u32>().unwrap_or(1);
                        (y, m, d)
                    } else {
                        (taqwim_date.year, taqwim_date.month, 1)
                    }
                } else {
                    let d = target_spec.parse::<u32>().unwrap_or(1);
                    // Check if month or year were passed as subsequent arguments
                    let conv_idx = args.iter().position(|a| a == "--convert-hijri" || a == "--convert").unwrap();
                    let m = args.get(conv_idx + 2).and_then(|v| v.parse::<u32>().ok()).unwrap_or(taqwim_date.month);
                    let y = args.get(conv_idx + 3).and_then(|v| v.parse::<i32>().ok()).unwrap_or(taqwim_date.year);
                    (y, m, d)
                };

                let (greg_date, greg_weekday, applied_offset) = Taqwim::convert_hijri_to_gregorian(
                    target_year,
                    target_month,
                    target_day,
                    now_local.date_naive(),
                    taqwim_default_offset,
                    &taqwim_adjustments,
                );

                let month_names = taqwim::TaqwimDate::month_names();
                let m_idx = (target_month.saturating_sub(1) as usize).min(11);
                let m_name = month_names[m_idx];

                println!("============================================================");
                println!("  taqwim - Hijri to Gregorian Converter");
                println!("============================================================");
                println!(
                    "Hijri Input:       {} {} {} AH",
                    target_day, m_name, target_year
                );
                println!(
                    "Gregorian Result:  {} ({})",
                    greg_date.format("%A, %d %B %Y"),
                    greg_weekday
                );
                println!(
                    "Applied Offset:    {:+ } day(s) (Month: {:04}-{:02})",
                    applied_offset, target_year, target_month
                );
                println!("------------------------------------------------------------");
                return;
            }

            // Dedicated --white-days flag output
            if is_white_days {
                println!("============================================================");
                println!("  munasabat - White Days (Ayyam al-Bid) & Month Verification");
                println!("============================================================");
                println!(
                    "Current Taqwim:    {} (Day: {})",
                    taqwim_date.format(),
                    taqwim_date.weekday_name
                );
                println!(
                    "Month 1st:         {} ({})",
                    month_start.first_gregorian.format("%a, %d %b %Y"),
                    month_start.first_weekday
                );
                println!(
                    "Active Offset:     {:+ } day(s) (Ledger key: {})",
                    month_start.applied_offset,
                    month_start.month_key
                );
                println!();
                println!("White Days (13th, 14th, 15th):");
                println!(
                    "  13th {}:  {} ({})",
                    white_days.month_name,
                    white_days.day13_gregorian.format("%a, %d %b %Y"),
                    white_days.day13_weekday
                );
                println!(
                    "  14th {}:  {} ({})",
                    white_days.month_name,
                    white_days.day14_gregorian.format("%a, %d %b %Y"),
                    white_days.day14_weekday
                );
                println!(
                    "  15th {}:  {} ({})",
                    white_days.month_name,
                    white_days.day15_gregorian.format("%a, %d %b %Y"),
                    white_days.day15_weekday
                );
                println!();
                if white_days.is_currently_white_day {
                    println!("✨ TODAY is White Day {} of {}! (Recommended voluntary fast)",
                        white_days.current_white_day_number.unwrap_or(taqwim_date.day),
                        white_days.month_name
                    );
                } else if white_days.days_until_start > 0 {
                    println!("⏳ {} day(s) until White Days begin.", white_days.days_until_start);
                } else {
                    println!("White Days for this month have concluded.");
                }
                println!("------------------------------------------------------------");
                return;
            }

            // Calculate Fasting (Munasabat / Suhoor / Imsak)
            let today_fajr = mawaqit
                .schedule
                .iter()
                .find(|m| m.prayer == Prayer::Fajr)
                .map(|m| m.time)
                .unwrap_or(now_local);

            let fasting = Munasabat::calculate_fasting(
                now_local,
                today_fajr,
                today_maghrib,
                tomorrow_fajr,
                imsak_buffer_minutes,
            );

            // Dedicated --compare-fasting <city1,city2,...> or --compare
            if let Some(cities_str) = compare_fasting_arg {
                let city_list: Vec<&str> = if cities_str.contains(',') {
                    cities_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
                } else {
                    let mut list = vec![cities_str.as_str()];
                    let comp_idx = args.iter().position(|a| a == "--compare-fasting" || a == "--compare").unwrap();
                    for next_arg in args.iter().skip(comp_idx + 2) {
                        if next_arg.starts_with('-') {
                            break;
                        }
                        list.push(next_arg.as_str());
                    }
                    list
                };

                println!("============================================================");
                println!("  munasabat - Global Fasting Duration Comparison");
                println!("============================================================");
                println!("Date: {}\n", now_local.format("%A, %d %B %Y"));
                println!("{:<16} {:<10} {:<10} {:<10} {:<16}", "Location", "Imsak", "Fajr (Cut)", "Maghrib", "Fasting Duration");
                println!("{:-<16} {:-<10} {:-<10} {:-<10} {:-<16}", "", "", "", "", "");

                // Include current location first
                let curr_dur_m = fasting.fasting_duration.num_minutes();
                println!(
                    "{:<16} {:<10} {:<10} {:<10} {:02}h {:02}m (Local)",
                    mawaqit.location_name,
                    fasting.imsak_time.format("%H:%M"),
                    fasting.suhoor_cutoff.format("%H:%M"),
                    fasting.iftar_time.format("%H:%M"),
                    curr_dur_m / 60,
                    curr_dur_m % 60
                );

                for city_name in city_list {
                    if city_name.eq_ignore_ascii_case(&mawaqit.location_name) {
                        continue;
                    }
                    if let Some((std_name, lat, lon, city_utc_offset)) = Tawqit::lookup_city(city_name) {
                        let city_method = Tawqit::resolve_regional_method(lat, lon);
                        let city_tawqit = Tawqit {
                            location_name: std_name.to_string(),
                            coordinates: Coordinates::new(lat, lon),
                            method: city_method,
                            madhab,
                            custom_fajr_angle: None,
                            custom_isha_angle: None,
                        };
                        let city_muwaqqit = Muwaqqit::new(city_tawqit.clone());
                        if let Ok(city_mawaqit) = city_muwaqqit.calculate(now_local) {
                            let c_fajr = city_mawaqit.schedule.iter().find(|m| m.prayer == Prayer::Fajr).map(|m| m.time).unwrap_or(now_local);
                            let c_maghrib = city_mawaqit.schedule.iter().find(|m| m.prayer == Prayer::Maghrib).map(|m| m.time).unwrap_or(now_local);
                            let c_tomorrow = now_local.date_naive() + Duration::days(1);
                            let c_tomorrow_fajr = PrayerSchedule::new()
                                .on(c_tomorrow)
                                .for_location(city_tawqit.coordinates)
                                .with_configuration(city_tawqit.to_salah_params())
                                .calculate()
                                .map(|s| chrono::DateTime::from(s.time(Prayer::Fajr)))
                                .unwrap_or(c_fajr + Duration::days(1));
                            let c_fasting = Munasabat::calculate_fasting(now_local, c_fajr, c_maghrib, c_tomorrow_fajr, imsak_buffer_minutes);
                            let dur_m = c_fasting.fasting_duration.num_minutes();
                            let diff_m = dur_m - curr_dur_m;
                            let diff_str = if diff_m > 0 {
                                format!(" (+{:02}h {:02}m)", diff_m / 60, diff_m % 60)
                            } else if diff_m < 0 {
                                let abs_m = diff_m.abs();
                                format!(" (-{:02}h {:02}m)", abs_m / 60, abs_m % 60)
                            } else {
                                " (same)".to_string()
                            };

                            // Convert Fajr, Imsak, Maghrib times from local machine timezone to target city's wall clock time
                            let local_offset_hours = now_local.offset().local_minus_utc() / 3600;
                            let tz_shift = Duration::hours((city_utc_offset - local_offset_hours) as i64);

                            let disp_imsak = c_fasting.imsak_time + tz_shift;
                            let disp_fajr = c_fasting.suhoor_cutoff + tz_shift;
                            let disp_iftar = c_fasting.iftar_time + tz_shift;

                            println!(
                                "{:<16} {:<10} {:<10} {:<10} {:02}h {:02}m{}",
                                std_name,
                                disp_imsak.format("%H:%M"),
                                disp_fajr.format("%H:%M"),
                                disp_iftar.format("%H:%M"),
                                dur_m / 60,
                                dur_m % 60,
                                diff_str
                            );
                        }
                    } else {
                        println!("{:<16} (Unknown city - use --lat/--lon or choose from known list)", city_name);
                    }
                }
                println!("------------------------------------------------------------");
                return;
            }

            // Dedicated --fasting / --suhoor / --imsak flag output
            if is_fasting {
                let f_dur_m = fasting.fasting_duration.num_minutes();
                let e_dur_m = fasting.eating_window_duration.num_minutes();

                println!("============================================================");
                println!("  munasabat - Suhoor, Imsak & Fasting Schedule");
                println!("============================================================");
                println!("Location:           {} ({:.2}°, {:.2}°)", mawaqit.location_name, latitude, longitude);
                println!("Date:               {} ({} / {})", mawaqit.date, taqwim_date.format(), taqwim_date.weekday_name);
                println!();
                println!("Schedule:");
                println!("  Imsak (Buffer {}m):   {}", fasting.imsak_buffer_minutes, fasting.imsak_time.format("%H:%M"));
                println!("  Suhoor Cutoff / Fajr: {}", fasting.suhoor_cutoff.format("%H:%M"));
                println!("  Iftar / Maghrib:      {}", fasting.iftar_time.format("%H:%M"));
                println!();
                println!("Durations:");
                println!("  Fasting Duration:     {:02}h {:02}m (Fajr -> Maghrib)", f_dur_m / 60, f_dur_m % 60);
                println!("  Eating / Night Window: {:02}h {:02}m (Maghrib -> Fajr)", e_dur_m / 60, e_dur_m % 60);
                println!();
                match fasting.status {
                    munasabat::FastingStatus::SuhoorPermitted => {
                        if let Some(left) = fasting.time_until_imsak {
                            println!("Status: Suhoor Permitted (Imsak in {}, Fajr cutoff in {})",
                                format_duration(left.num_seconds()),
                                format_duration(fasting.time_until_suhoor_cutoff.unwrap().num_seconds())
                            );
                        }
                    }
                    munasabat::FastingStatus::ImsakBuffer => {
                        let left = fasting.time_until_suhoor_cutoff.unwrap();
                        println!("Status: ⚠️ IMSAK ACTIVE! Prepare to stop eating. Suhoor cutoff in {}",
                            format_duration(left.num_seconds())
                        );
                    }
                    munasabat::FastingStatus::FastingActive => {
                        let elapsed = fasting.elapsed_fasting_time.unwrap();
                        let left = fasting.time_until_iftar.unwrap();
                        println!("Status: ✨ FASTING ACTIVE (Elapsed: {}, Iftar in: {})",
                            format_duration(elapsed.num_seconds()),
                            format_duration(left.num_seconds())
                        );
                    }
                    munasabat::FastingStatus::FastCompleted => {
                        println!("Status: Fast Completed (Iftar was at {})", fasting.iftar_time.format("%H:%M"));
                    }
                }
                println!("------------------------------------------------------------");
                return;
            }

            // Extract Sunrise and Dhuhr for prohibited times and Duha window
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

            let prohibited = Munasabat::calculate_prohibited_times(
                now_local,
                today_sunrise,
                today_dhuhr,
                today_maghrib,
            );

            let friday = Munasabat::calculate_friday_istijabah(
                now_local,
                today_maghrib,
            );

            let annual_events = Munasabat::calculate_annual_observances(
                taqwim_date.year,
                now_local.date_naive(),
                taqwim_default_offset,
                &taqwim_adjustments,
            );

            // Dedicated --prohibited / --duha flag output
            if is_prohibited {
                println!("============================================================");
                println!("  munasabat - Prohibited Prayer Times & Duha Window");
                println!("============================================================");
                println!("Location:           {}", mawaqit.location_name);
                println!("Date:               {} ({})", mawaqit.date, taqwim_date.format());
                println!();
                println!("The 3 Prohibited / Disliked Times (Awqat al-Nahy):");
                println!(
                    "  1. Sunrise (Tulu'):   {} - {} (Ascent of sun)",
                    prohibited.sunrise_prohibited_start.format("%H:%M"),
                    prohibited.sunrise_prohibited_end.format("%H:%M")
                );
                println!(
                    "  2. Midday (Zawal):    {} - {} (Exact zenith / Istiwa')",
                    prohibited.zawal_prohibited_start.format("%H:%M"),
                    prohibited.zawal_prohibited_end.format("%H:%M")
                );
                println!(
                    "  3. Sunset (Ghurub):   {} - {} (Pale yellowing before Maghrib)",
                    prohibited.sunset_prohibited_start.format("%H:%M"),
                    prohibited.sunset_prohibited_end.format("%H:%M")
                );
                println!();
                println!("Permissible Forenoon Prayer (Salat al-Duha / Ishraq):");
                println!(
                    "  Duha Window:          {} - {} (Begins at Ishraq until Zawal)",
                    prohibited.duha_start.format("%H:%M"),
                    prohibited.duha_end.format("%H:%M")
                );
                println!();
                if let Some(reason) = prohibited.prohibited_reason {
                    println!("⚠️ CURRENTLY IN PROHIBITED PRAYER TIME: {}", reason);
                } else if prohibited.is_currently_duha {
                    println!("✨ Currently within Duha Window! (Recommended voluntary prayer)");
                } else {
                    println!("Status: Standard prayer times permissible.");
                }
                println!("------------------------------------------------------------");
                return;
            }

            // Dedicated --observances / --annual flag output
            if is_annual {
                println!("============================================================");
                println!("  munasabat - Major Islamic Annual Stations & Sacred Days");
                println!("============================================================");
                println!("Hijri Year: {} AH (Ledger month offset applied)\n", taqwim_date.year);
                println!("{:<28} {:<18} {:<24} {:<10}", "Observance", "Hijri Date", "Gregorian Date", "Status");
                println!("{:-<28} {:-<18} {:-<24} {:-<10}", "", "", "", "");

                for ev in &annual_events {
                    let h_str = format!("{} {}", ev.hijri_day, ev.month_name);
                    let g_str = format!("{} ({})", ev.greg_date.format("%d %b %Y"), ev.greg_weekday);
                    let status_str = if ev.is_today {
                        "TODAY ★"
                    } else if ev.days_until > 0 {
                        &format!("in {}d", ev.days_until)
                    } else {
                        "passed"
                    };

                    println!("{:<28} {:<18} {:<24} {:<10}", ev.name, h_str, g_str, status_str);
                }
                println!("------------------------------------------------------------");
                return;
            }

            let countdown_str = format_duration(mawaqit.time_until_next.num_seconds());

            if let Some(template) = custom_format {
                // Mode A: Custom template string (wttr.in style)
                let rendered = Bayan::format(
                    &template,
                    &mawaqit,
                    &moon,
                    &sun,
                    moon_style,
                    &taqwim_date,
                    &night,
                    &white_days,
                    &fasting,
                    &prohibited,
                    &friday,
                );
                println!("{}", rendered);
            } else if is_verbose {
                // Mode B: Verbose table mode
                println!(
                    "Location: {} | Date: {} ({} / {})\n",
                    mawaqit.location_name,
                    mawaqit.date,
                    taqwim_date.format(),
                    if now_local >= today_maghrib { "Eve of next day" } else { "Day" }
                );

                let total_items = mawaqit.schedule.len();
                for (idx, miqat) in mawaqit.schedule.iter().enumerate() {
                    let is_last = idx + 1 == total_items;
                    let line = style.format_row(
                        miqat.name,
                        &miqat.time.format("%H:%M").to_string(),
                        miqat.status,
                        is_last,
                    );
                    println!("{}", line);
                }

                println!(
                    "\nNext: {} at {} (-{})",
                    mawaqit.next_prayer_name,
                    mawaqit.next_prayer_time.format("%H:%M"),
                    countdown_str
                );

                // Method & Madhab detail in verbose mode
                println!("\nConfiguration:");
                println!(
                    "  Method: {:?} | Madhab: {:?} | Night Basis: {}",
                    tawqit.method, tawqit.madhab, night.basis.name()
                );
                if let Some(fa) = tawqit.custom_fajr_angle {
                    print!("  Custom Fajr Angle: {:.1}°", fa);
                }
                if let Some(ia) = tawqit.custom_isha_angle {
                    print!(" | Custom Isha Angle: {:.1}°", ia);
                }
                if tawqit.custom_fajr_angle.is_some() || tawqit.custom_isha_angle.is_some() {
                    println!();
                }

                // Dedicated Observances / Munasabat section in verbose mode
                println!("\nObservances (Munasabat):");
                println!(
                    "  Midnight (Nisf):     {}",
                    night.midnight.format("%H:%M")
                );
                let last_third_status = if night.is_currently_last_third {
                    " (ACTIVE NOW)"
                } else if let Some(until) = night.time_until_last_third {
                    &format!(" (in {})", format_duration(until.num_seconds()))
                } else {
                    ""
                };
                println!(
                    "  Last Third of Night: {} - {}{}",
                    night.last_third.start.format("%H:%M"),
                    night.last_third.end.format("%H:%M"),
                    last_third_status
                );
                let white_day_status = if white_days.is_currently_white_day {
                    " (ACTIVE TODAY)"
                } else if white_days.days_until_start > 0 {
                    &format!(" (in {} days)", white_days.days_until_start)
                } else {
                    " (passed)"
                };
                println!(
                    "  White Days (Bid):    {} - {} (13-15 {}){}",
                    white_days.day13_gregorian.format("%d %b"),
                    white_days.day15_gregorian.format("%d %b"),
                    white_days.month_name,
                    white_day_status
                );
                let f_dur = fasting.fasting_duration.num_minutes();
                println!(
                    "  Fasting & Suhoor:    {:02}h {:02}m (Imsak: {}, Fajr: {}, Iftar: {})",
                    f_dur / 60,
                    f_dur % 60,
                    fasting.imsak_time.format("%H:%M"),
                    fasting.suhoor_cutoff.format("%H:%M"),
                    fasting.iftar_time.format("%H:%M")
                );
                let duha_status = if prohibited.is_currently_duha { " (ACTIVE NOW)" } else { "" };
                println!(
                    "  Duha Window:         {} - {}{}",
                    prohibited.duha_start.format("%H:%M"),
                    prohibited.duha_end.format("%H:%M"),
                    duha_status
                );
                if friday.is_friday {
                    let istijabah_status = if friday.is_currently_active { " (ACTIVE NOW)" } else { "" };
                    println!(
                        "  Friday Istijabah:    {} - {}{}",
                        friday.window_start.format("%H:%M"),
                        friday.window_end.format("%H:%M"),
                        istijabah_status
                    );
                }

                // Dedicated Celestial section in verbose mode
                println!("\nCelestial & Calendar:");
                println!("  Calendar: {}", taqwim_date.format());
                println!(
                    "  Moon:     {} {} ({}% illuminated, Day {:.1})",
                    moon_symbol,
                    moon.phase_name,
                    moon_pct,
                    moon.age_days
                );
                let alt_sign = if sun.altitude >= 0.0 { "+" } else { "" };
                println!(
                    "  Sun:      {}{:.1}° {} (Azimuth {:.0}° {})",
                    alt_sign,
                    sun.altitude,
                    sun.state_name,
                    sun.azimuth,
                    sun.compass_direction
                );

                println!("\x1b[90mTip: Run mawaqit --help for all flags | Use -f for custom tmux templates\x1b[0m");
            } else {
                // Mode C: Default single-line format
                let current_display = if let Some(m) = mawaqit
                    .schedule
                    .iter()
                    .find(|m| m.status == miqat::MawqutStatus::Current)
                {
                    format!("{} {}", m.name, m.time.format("%H:%M"))
                } else {
                    mawaqit.current_name
                };

                println!(
                    "{}: {} -> {} {} (-{}) | {} {}% | {}",
                    mawaqit.location_name,
                    current_display,
                    mawaqit.next_prayer_name,
                    mawaqit.next_prayer_time.format("%H:%M"),
                    countdown_str,
                    moon_symbol,
                    moon_pct,
                    taqwim_date.format()
                );
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
