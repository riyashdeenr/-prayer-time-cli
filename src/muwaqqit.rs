use chrono::{DateTime, Duration, Local};
use salah::prelude::{Prayer, PrayerSchedule};

use crate::miqat::{MawqutStatus, Miqat};
use crate::tawqit::Tawqit;

/// The Timekeeper Engine (Active Participle: muwaqqit)
pub struct Muwaqqit {
    pub tawqit: Tawqit,
}

/// The daily prayer collection results (Plural Noun: mawaqit)
#[derive(Debug)]
pub struct MawaqitDaily {
    pub location_name: String,
    pub date: chrono::NaiveDate,
    pub schedule: Vec<Miqat>,
    pub current_name: String,
    pub next_prayer_name: String,
    pub next_prayer_time: DateTime<Local>,
    pub time_until_next: Duration,
}

impl Muwaqqit {
    pub fn new(tawqit: Tawqit) -> Self {
        Self { tawqit }
    }

    /// Calculate the full prayer schedule for a given point in time
    pub fn calculate(&self, now: DateTime<Local>) -> Result<MawaqitDaily, String> {
        let date = now.date_naive();
        let salah_params = self.tawqit.to_salah_params();

        let schedule = PrayerSchedule::new()
            .on(date)
            .for_location(self.tawqit.coordinates)
            .with_configuration(salah_params)
            .calculate()
            .map_err(|e| format!("Failed to calculate prayer schedule: {:?}", e))?;

        let prayers = [
            (Prayer::Fajr, "Fajr"),
            (Prayer::Sunrise, "Sunrise"),
            (Prayer::Dhuhr, "Dhuhr"),
            (Prayer::Asr, "Asr"),
            (Prayer::Maghrib, "Maghrib"),
            (Prayer::Isha, "Isha"),
        ];

        // Convert all today's prayer times to Local
        let times: Vec<(Prayer, &'static str, DateTime<Local>)> = prayers
            .iter()
            .map(|(p, name)| (*p, *name, DateTime::from(schedule.time(*p))))
            .collect();

        // Determine current and next prayer safely without relying on salah's panicking .current()
        // Case 1: Before Fajr today (e.g. 05:25 AM before 05:40 AM Fajr)
        let fajr_time = times[0].2;
        let isha_time = times[5].2;

        let (current_idx, next_idx, is_before_fajr, is_after_isha) = if now < fajr_time {
            (None, Some(0), true, false)
        } else if now >= isha_time {
            (Some(5), None, false, true)
        } else {
            // It's between Fajr and Isha today
            let mut curr = 0;
            for i in 0..times.len() {
                if now >= times[i].2 {
                    curr = i;
                }
            }
            let next = (curr + 1).min(times.len() - 1);
            (Some(curr), Some(next), false, false)
        };

        let mut miqat_list = Vec::new();
        for (i, (prayer, _name, dt_local)) in times.iter().enumerate() {
            let status = if Some(i) == current_idx {
                MawqutStatus::Current
            } else if Some(i) == next_idx {
                MawqutStatus::Upcoming
            } else if *dt_local < now {
                MawqutStatus::Passed
            } else {
                MawqutStatus::Upcoming
            };

            miqat_list.push(Miqat::new(*prayer, *dt_local, status));
        }

        let (current_name, next_prayer_name, next_prayer_time) = if is_before_fajr {
            ("Qiyam / Night".to_string(), "Fajr".to_string(), fajr_time)
        } else if is_after_isha {
            // Next is tomorrow's Fajr
            let tomorrow = date + chrono::Duration::days(1);
            let next_day_schedule = PrayerSchedule::new()
                .on(tomorrow)
                .for_location(self.tawqit.coordinates)
                .with_configuration(salah_params)
                .calculate()
                .map_err(|e| format!("Failed to calculate tomorrow's schedule: {:?}", e))?;
            let tomorrow_fajr = DateTime::from(next_day_schedule.time(Prayer::Fajr));
            ("Isha".to_string(), "Fajr".to_string(), tomorrow_fajr)
        } else {
            let n_idx = next_idx.unwrap();
            let c_idx = current_idx.unwrap();
            (times[c_idx].1.to_string(), times[n_idx].1.to_string(), times[n_idx].2)
        };

        let time_until_next = if next_prayer_time > now {
            next_prayer_time - now
        } else {
            Duration::zero()
        };

        Ok(MawaqitDaily {
            location_name: self.tawqit.location_name.clone(),
            date,
            schedule: miqat_list,
            current_name,
            next_prayer_name,
            next_prayer_time,
            time_until_next,
        })
    }
}
