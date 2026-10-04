# mawaqit (مواقيت) - Terminal Prayer & Celestial Engine

An offline, zero-network-latency CLI tool written in pure Rust providing prayer schedules, Jean Meeus astronomical celestial tracking (Moon/Sun), custom `wttr.in`-style format templating, month-indexed Hijri adjustments (`taqwim`), and Islamic calendar observances (`munasabat`).

---

## Architecture & Domain Model

The internal domain architecture maps directly to the Arabic root *w-q-t* (و-ق-ت):

```
mawaqit (Root Application / Collection)
 ├── miqat        -> Individual prayer milestone (struct Miqat)
 ├── mawqut       -> Designated state: upcoming, active, passed (enum MawqutStatus)
 ├── muwaqqit     -> Calculation engine & timekeeper (struct Muwaqqit)
 ├── tawqit       -> Regional coordinates & method configuration (struct Tawqit)
 ├── falak        -> Pure Rust astronomical ephemeris (struct Falak)
 ├── taqwim       -> Islamic lunar calendar & month-indexed ledger (struct Taqwim)
 ├── munasabat    -> Non-prayer stations, night thirds, fasting, & prohibited times
 ├── bayan        -> Template token interpolation engine (struct Bayan)
 └── theme        -> Terminal formatting styles (enum ThemeStyle)
```

---

## Quick Start & Basic Usage

### 1. Default Single-Line Output (for tmux / status bars / polybar)
```bash
cargo run
# Singapore: Dhuhr 12:56 -> Asr 16:04 (-01:12:45) | ◔ 37% | Sunday, 21 Rabi' al-Thani 1448 AH
```

### 2. Verbose Table View
```bash
cargo run -- -v
```

### 3. Custom Format String (`-f` / `--format`)
```bash
cargo run -- -f "%location: %next in %remaining | %moon %moon_pct | Fajr: %fajr, Maghrib: %maghrib"
# Singapore: Asr in 01:12:45 | ◔ 37% | Fajr: 05:35, Maghrib: 18:57
```

### 4. Fasting Schedule & Live State Machine (`--fasting` / `--suhoor` / `--imsak`)
```bash
cargo run -- --fasting
# Displays Imsak (with configurable buffer), Suhoor cutoff (Fajr), Iftar (Maghrib),
# total fasting duration, night eating window, and live state (Suhoor permitted / Imsak warning / Fasting active).
```

### 5. Multi-City Fasting Comparison (`--compare-fasting`)
Compare fasting duration across different latitudes and regional calculation methods:
```bash
cargo run -- --compare-fasting "London,Tokyo,Makkah,Oslo"
```

### 6. Night Divisions & Last Third Breakdown (`--night`)
```bash
cargo run -- --night
# Breaks down 1st Third, 2nd Third, Midnight (Nisf), and Last Third (Thuluth al-Akhir / Tahajjud).
```

### 7. White Days (*Ayyam al-Bid*) & Local Moonsighting Verification (`--white-days`)
```bash
cargo run -- --white-days
# Shows Day 1 of the month (for moonsighting verification), active offset, and Gregorian dates of the 13th, 14th, and 15th.
```

### 8. Hijri to Gregorian Date Converter (`--convert-hijri`)
Converts any Hijri day/month into its exact Gregorian date taking into account all active month ledger offsets:
```bash
cargo run -- --convert-hijri 13               # 13th of current month
cargo run -- --convert-hijri 1448-05-13        # 13th Jumada al-Ula 1448 AH
```

### 9. Prohibited Prayer Times & Duha Window (`--prohibited` / `--duha`)
```bash
cargo run -- --prohibited
# Tracks the 3 forbidden times (Sunrise, Solar Zenith, Sunset) and forenoon Duha window.
```

### 10. Major Annual Islamic Sacred Stations (`--observances`)
```bash
cargo run -- --observances
# Displays canonical stations: Tasu'a, 'Ashura, Mid-Sha'ban, Ramadan, Laylat al-Qadr, Eid al-Fitr, 'Arafah, Eid al-Adha, Tashriq.
```

### 11. Per-Month Islamic Calendar Ledger Override (`--taqwim-adjust`)
```bash
cargo run -- --taqwim-adjust +1               # Adjust current month by +1 day
cargo run -- --taqwim-adjust -1 --taqwim-month 1448-05  # Adjust specific month
# Automatically appends audit record to taqwim_adjustments.log and updates config.toml
```

---

## CLI Flags & Options Table

| Flag / Option | Description | Example / Values |
| :--- | :--- | :--- |
| `-v`, `--verbose` | Full multi-section table display | `mawaqit -v` |
| `-f`, `--format <STR>` | Custom `wttr.in`-style output template | `mawaqit -f "%next in %remaining"` |
| `--city <NAME>` | City name (auto-populates coordinates) | `mawaqit --city London` |
| `--lat <F64>`, `--lon <F64>` | Custom geographical coordinates | `mawaqit --lat 51.5074 --lon -0.1278` |
| `--method <METHOD>` | Juristic calculation method | `singapore`, `mwl`, `ummalqura`, `isna`, `egyptian`, `karachi`, `turkey`, `dubai` |
| `--madhab <MADHAB>` | Asr shadow calculation | `shafi` (1x shadow, default), `hanafi` (2x shadow) |
| `--fajr-angle <DEG>` | Custom Fajr twilight depression angle | `mawaqit --fajr-angle 19.5` |
| `--isha-angle <DEG>` | Custom Isha twilight depression angle | `mawaqit --isha-angle 17.5` |
| `--night-basis <BASIS>` | Basis for night calculation | `sunset` (Maghrib to Fajr, default), `isha` (Isha to Fajr) |
| `--night` | Dedicated breakdown of the 3 thirds of night | `mawaqit --night` |
| `--imsak-mins <N>` | Precautionary Imsak buffer in minutes | `mawaqit --imsak-mins 15` (default: `10`) |
| `--fasting`, `--suhoor` | Dedicated Suhoor, Imsak & Fasting schedule | `mawaqit --fasting` |
| `--compare-fasting <LIST>` | Multi-city fasting comparison | `mawaqit --compare-fasting "London,Tokyo,Makkah"` |
| `--white-days` | White Days (13th-15th) & Month 1st check | `mawaqit --white-days` |
| `--convert-hijri <SPEC>` | Convert Hijri date to Gregorian | `mawaqit --convert-hijri 13` or `1448-05-13` |
| `--prohibited`, `--duha` | Prohibited times (*Awqat al-Nahy*) & Duha | `mawaqit --prohibited` |
| `--observances` | Major annual Islamic sacred stations | `mawaqit --observances` |
| `--taqwim-adjust <+/-N>` | Apply month-indexed calendar offset | `mawaqit --taqwim-adjust +1` |
| `--taqwim-month <KEY>` | Target Hijri month for adjustment | `mawaqit --taqwim-month 1448-05` |
| `--taqwim-offset <N>` | Global fallback Hijri day offset | `mawaqit --taqwim-offset +1` |
| `--style`, `--theme <NAME>` | Prayer row styling | `dots` (default), `timeline`, `blocks`, `radio`, `minimal` |
| `--moon <STYLE>` | Moon phase glyph style | `geometric` (`○ ◔ ◑ ◕ ● ◐`), `lunar`, `classic`, `text` |
| `--save` | Persist current configuration to disk | `mawaqit --city London --save` |

---

## Template Tokens Table (`-f` / `--format`)

| Token | Description | Example Output |
| :--- | :--- | :--- |
| **`%location`**, **`%city`** | Configured location name | `Singapore` |
| **`%date`** | Gregorian calendar date | `2026-10-04` |
| **`%current`** | Current active prayer name | `Dhuhr` |
| **`%next`** | Next upcoming prayer name | `Asr` |
| **`%next_time`** | Next prayer time | `16:04` |
| **`%remaining`** | Time until next prayer (`HH:MM:SS`) | `01:12:45` |
| **`%remaining_hm`** | Time until next prayer (`HHh MMm`) | `01h 12m` |
| **`%fajr`** (or **`%f`**) | Fajr prayer time | `05:35` |
| **`%sunrise`** | Sunrise milestone time | `06:52` |
| **`%dhuhr`** (or **`%d`**) | Dhuhr prayer time | `12:56` |
| **`%asr`** (or **`%a`**) | Asr prayer time | `16:04` |
| **`%maghrib`** (or **`%m`**) | Maghrib prayer time | `18:57` |
| **`%isha`** (or **`%i`**) | Isha prayer time | `20:06` |
| **`%moon`** | Moon phase glyph | `◔` |
| **`%moon_pct`** | Moon illumination percentage | `37%` |
| **`%moon_name`** | Moon phase name | `Waning Crescent` |
| **`%sun_alt`** | Sun altitude in degrees | `+60.4°` |
| **`%sun_state`** | Solar twilight/elevation state | `Daylight` |
| **`%sun_dir`** | Compass heading of sun azimuth | `W` |
| **`%hijri`**, **`%taqwim`** | Full textual Hijri date | `Sunday, 21 Rabi' al-Thani 1448 AH` |
| **`%hijri_short`** | Short Hijri date | `21 Rabi' al-Thani 1448 AH` |
| **`%hijri_day`** | Current Hijri day number | `21` |
| **`%hijri_month`** | Current Hijri month name | `Rabi' al-Thani` |
| **`%hijri_year`** | Current Hijri year | `1448` |
| **`%hijri_weekday`** | Hijri day of week | `Sunday` |
| **`%hijri_offset`** | Active ledger offset | `+0` |
| **`%midnight`** | Middle of night (*Nisf al-Layl*) | `00:15` |
| **`%last_third`** | Start of Last Third (*Tahajjud*) | `02:01` |
| **`%last_third_range`**| Full Last Third time window | `02:01-05:34` |
| **`%white_days`** | Summary of current month White Days | `13-15 Rabi' al-Thani (Fri 25 Sep - Sun 27 Sep)` |
| **`%imsak`** | Precautionary Imsak cutoff time | `05:25` |
| **`%suhoor`** / **`%suhoor_cutoff`** | Final Suhoor cutoff (Fajr) | `05:35` |
| **`%iftar`** | Fast-breaking time (Maghrib) | `18:57` |
| **`%fasting_duration`** | Today's fasting duration | `13h 22m` |
| **`%duha`** | Start of Duha / Ishraq window | `07:07` |
| **`%duha_range`** | Full permissible Duha window | `07:07-12:46` |
| **`%zawal`** | Midday zenith prohibited start | `12:46` |
| **`%istijabah`** | Friday Hour of Response window | `17:57-18:57` |

---

## Configuration & Audit Ledger

Configuration is persisted across runs in standard TOML format:
* **Linux / macOS**: `~/.config/mawaqit/config.toml`
* **Windows**: `%APPDATA%\mawaqit\config.toml`

### Example `config.toml`:
```toml
# mawaqit persistent user configuration

[location]
location = "Singapore"
latitude = 1.3521
longitude = 103.8198

[calculation]
method = "singapore"
madhab = "shafi"
night_basis = "sunset"
imsak_buffer = 10

[ui]
style = "dots"
moon = "geometric"

[taqwim]
default_offset = 0

[taqwim.adjustments]
"1448-03" = +1
"1448-04" = 0
```

Each adjustment is logged with a human-readable audit trail in `taqwim_adjustments.log`:
```text
2026-10-04 14:10:00 | Month: 1448-03 | Offset: +1 | Reason: User CLI adjustment
```
