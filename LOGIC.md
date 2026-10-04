# Architectural Logic & Boundary Specifications

This document outlines the core business and astronomical logic implemented in `mawaqit`, specifically documenting edge cases, boundary conditions, and rationale for future modifications.

---

## 1. Daily Prayer Boundary & State Transition Logic (`src/muwaqqit.rs`)

The Islamic prayer day does not align with a standard calendar day (00:00:00 to 23:59:59). Instead, it spans three distinct astronomical zones:

```
[ 00:00:00 ] ───────► [ Fajr ] ───────► [ Isha ] ───────► [ 23:59:59 ]
  Zone 1: Pre-Fajr        Zone 2: Daily Prayers       Zone 3: Post-Isha
 (Qiyam / Suhoor)           (Standard Salah)         (Late Night / Qiyam)
```

### Zone 1: Pre-Fajr Window (Midnight until Today's Fajr)
* **Time Range**: `00:00:00 <= now < today.fajr` (e.g. 05:25 AM when Fajr is 05:37 AM).
* **Upstream Bug Identified**: Upstream `salah-0.7.6`'s `schedule.current()` panics with `Out of bounds` because it assumes a prayer from *today* must already have arrived.
* **Our Engine Logic**:
  - `current_name`: Identified as `"Qiyam / Night"`.
  - `next_prayer`: Guaranteed to be **today's Fajr**.
  - `time_until_next`: `today.fajr - now`.
  - All prayers in today's schedule table are marked as upcoming (`MawqutStatus::Upcoming` / `○`).

### Zone 2: Daytime & Evening Prayers (`today.fajr <= now < today.isha`)
* **Time Range**: Standard prayer progression:
  `Fajr -> Sunrise -> Dhuhr -> Asr -> Maghrib -> Isha`.
* **State Identification**:
  - Linear scan across the day's time milestones.
  - The latest prayer milestone where `now >= milestone.time` is marked as active (`MawqutStatus::Current` / `◉` emerald green).
  - The immediately succeeding milestone is marked as `MawqutStatus::Upcoming` (`○`).
  - All earlier milestones are marked as `MawqutStatus::Passed` (`●` dim gray).

### Zone 3: Post-Isha Window (`now >= today.isha` until `23:59:59`)
* **Time Range**: From Isha entry until midnight.
* **Engine Logic**:
  - `current_name`: `"Isha"`.
  - `next_prayer`: Cannot be found in today's prayer list.
  - **Lookahead Resolution**: The engine invokes `PrayerSchedule` for `today + 1 day` to compute **tomorrow's Fajr**.
  - `time_until_next`: `tomorrow.fajr - now`.

---

## 2. Formatting & Presentation Logic (`src/main.rs`, `src/theme.rs`)

### Default Single-Line Format
* **Design Philosophy**: Minimal, non-distracting, zero emojis, suitable for `tmux` status bars, `polybar`, `waybar`, or shell prompts.
* **Pattern**: `<Location>: <Current> -> <Next> <Time> (-<Remaining>)`
* **Examples**:
  - Daytime: `Singapore: Asr 16:01 -> Maghrib 19:03 (-00:13:15)`
  - Pre-Fajr: `Singapore: Qiyam / Night -> Fajr 05:37 (-00:10:35)`

### Verbose Table Mode (`--verbose` / `-v`)
* Driven by the Theme Engine (`ThemeStyle`), providing 5 UI/UX styles.
* **Default Theme**: `ThemeStyle::Dots` (`●` dim gray for passed, `◉` emerald green `#2ecc71` for current, `○` white for upcoming).
* **Styles Available via Flag**:
  - `--style dots` (Pattern 1)
  - `--style timeline` (Pattern 2)
  - `--style blocks` (Pattern 3)
  - `--style radio` (Pattern 4)
  - `--style minimal` (Pattern 5)

---

## 3. Anticipated Future Logic Changes

When refactoring or expanding this system in upcoming phases, consider:

1. **Sunrise Status**:
   - Currently, `Sunrise` is listed in the schedule array as a milestone between Fajr and Dhuhr.
   - *Consideration*: In Islamic jurisprudence, Sunrise (*Shuruq*) is not a prayer itself, but marks the expiration of Fajr and the start of the prohibited prayer time (*Duha* comes later).
2. **Qiyam Calculation**:
   - `salah` computes `qiyam` as the start of the last third of the night (`sunset + (night_duration * 2 / 3)`).
   - *Future upgrade*: Expose precise Qiyam time when in Zone 1 or Zone 3.
3. **Location & Tawqit Customization (`src/tawqit.rs`, `src/config.rs`)**:
   - Resolution hierarchy: CLI Flag -> Persistent `config.toml` -> Regional Auto-Resolution (MUIS for SG/MY, UmmAlQura for Gulf, ISNA for NA, Egyptian for North Africa, Karachi for Pakistan/India, MWL default elsewhere).
   - Madhab: Default is **Shafi'i** (1x shadow ratio for Shafi/Maliki/Hanbali). `--madhab hanafi` selects 2x shadow.
   - Power-user angle overrides: `--fajr-angle <deg>` and `--isha-angle <deg>`.
   - Persistence: `--save` stores current configuration in `~/.config/mawaqit/config.toml` (or `AppData\Roaming\mawaqit\config.toml`).
4. **Islamic Lunar Calendar Module (`src/taqwim.rs`)**:
   - Computes Islamic date from Julian Day using the Astronomical Tabular algorithm.
   - **Sunset Day Transition**: Checks current time against local today `Maghrib`. If `now >= today_maghrib`, the Islamic date automatically transitions to the eve of the next day.
   - **Month-Indexed Adjustment Ledger**: Supports per-month overrides (`[taqwim.adjustments]`) keyed by `"YYYY-MM"` (e.g. `"1448-03" = +1`). If no month override exists, falls back to `default_offset`.
   - **Audit Trail**: Every adjustment is appended to `taqwim_adjustments.log` with timestamp, target month, offset, and rationale.
   - **Template Tokens**: Exposes `%hijri`, `%taqwim`, `%hijri_day`, `%hijri_month`, and `%hijri_year` in `Bayan`.
5. **Observances & Night Breakdown Module (`src/munasabat.rs`)**:
   - Computes astronomical divisions of the night (First Third, Second Third, Midnight, and Last Third / *Thuluth al-Akhir*).
   - **Selectable Basis**:
     - `SunsetToDawn` (Default Majority): Night spans from **Maghrib to Fajr**.
     - `IshaToDawn` (Alternative): Night spans from **Isha to Fajr**.
   - Accessible via `--night` flag, `--night-basis <sunset|isha>`, and template tokens `%last_third`, `%last_third_range`, and `%midnight`.
6. **Suhoor, Imsak & Fasting Schedule (`src/munasabat.rs`, `src/main.rs`)**:
   - **Boundary Definition**: Fasting strictly spans from True Dawn (**Fajr**) to Sunset (**Maghrib**). Eating window spans from Maghrib to next day's Fajr.
   - **Configurable Imsak Buffer**: Configurable via `--imsak-mins <N>` (defaults to 10 minutes) and persistent config `imsak_buffer = N`.
   - **Fasting Status Engine**:
     - `SuhoorPermitted`: `now < imsak_time` (countdown to Imsak and Fajr cutoff).
     - `ImsakBuffer`: `imsak_time <= now < fajr` (warning state to prepare to stop eating).
     - `FastingActive`: `fajr <= now < maghrib` (tracks elapsed fasting time and countdown to Iftar).
     - `FastCompleted`: `now >= maghrib` (fast concluded).
   - **Multi-City Fasting Comparison**:
     - Accessible via `--compare-fasting <city1,city2,...>` (e.g. `--compare-fasting London,Tokyo,Makkah,Oslo`).
     - Auto-resolves city coordinates and regional calculation parameters.
     - Displays local wall clock times for each city and calculates duration differences relative to the user's primary location.
   - **Template Tokens**: Exposes `%imsak`, `%suhoor_cutoff` / `%suhoor`, `%iftar`, and `%fasting_duration`.
7. **Prohibited Prayer Times (Awqat al-Nahy) & Duha Window (`src/munasabat.rs`)**:
   - **The 3 Forbidden Times**:
     1. *Sunrise (Tulu')*: From Sunrise until ~15 mins after (ascent of sun above horizon).
     2. *Midday (Zawal / Istiwa')*: ~10 mins before Dhuhr until Dhuhr enters (sun at exact zenith before inclining).
     3. *Sunset (Ghurub)*: ~15 mins before Maghrib until Maghrib enters (pale yellowing of sun).
   - **Forenoon Prayer (Salat al-Duha / Ishraq)**: Permissible window spanning from Ishraq (`Sunrise + 15m`) until Zawal (`Dhuhr - 10m`).
   - Accessible via `--prohibited` / `--duha` flag and template tokens `%duha`, `%duha_range`, and `%zawal`.
8. **Friday Hour of Response & Annual Sacred Stations (`src/munasabat.rs`)**:
   - **Friday Istijabah**: Tracks the last hour before Maghrib on Fridays where du'a is answered (`%istijabah`).
   - **Annual Sacred Stations**:
     - Canonical calendar events computed using the `taqwim` month offset engine: Tasu'a, 'Ashura, Mid-Sha'ban, 1st Ramadan, Laylat al-Qadr (27th), Eid al-Fitr, Day of 'Arafah, Eid al-Adha, and Days of Tashriq.
     - Accessible via `--observances` / `--annual`.
