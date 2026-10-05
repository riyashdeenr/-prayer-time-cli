# mawaqit (مواقيت) - Terminal Prayer & Celestial Engine

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Edge: Cloudflare Workers](https://img.shields.io/badge/Edge-Cloudflare_Workers_Wasm-F38020?logo=cloudflare)](https://mawaqit.rahmanr.com)
[![Rust: 2024 Edition](https://img.shields.io/badge/Rust-2024_Edition-black?logo=rust)](https://www.rust-lang.org)

An offline, zero-network-latency CLI tool and pure WebAssembly edge microservice written in Rust. Provides high-precision prayer schedules, Jean Meeus astronomical celestial tracking (Moon/Sun), custom `wttr.in`-style format templating, month-indexed Hijri calendar adjustments (`taqwim`), and Islamic calendar observances (`munasabat`).

Available both as a **standalone local CLI binary** (`mawaqit`) and an **interactive Web TUI / curl service** at **[mawaqit.rahmanr.com](https://mawaqit.rahmanr.com/)**.

---

## Live Interactive Web TUI & `curl` Service

`mawaqit` runs as a high-performance WebAssembly microservice distributed across Cloudflare's global edge network at **[mawaqit.rahmanr.com](https://mawaqit.rahmanr.com/)**.

### 1. In Your Browser: Interactive Web TUI Simulator
When visited in any modern browser, `mawaqit.rahmanr.com` presents an interactive terminal emulator modeled with an ultra-clean Pitch Black / Zinc aesthetic:
* **Interactive Command Prompt**: Type commands directly (`help`, `fasting`, `night`, `makkah`, `white-days`, `clear`).
* **One-Click Quick Pills**: Rapidly execute common queries (`[fasting]`, `[night]`, `[white-days]`, `[duha]`, `[convert-hijri]`, `[madhab: hanafi]`, `[method: muis]`, `[method: egyptian]`).
* **Live Copy `curl` Button**: Header automatically generates and syncs the exact `curl` command corresponding to your active browser view.
* **Shell Features**: Full command history (`Up`/`Down` arrows) and Tab auto-completion.

### 2. In Your Terminal: Zero-Install Instant `curl`
No Rust installation required—fetch instant calculations directly from your terminal:

```bash
# Default one-liner for your current geolocation (detected via CF IP-geocoding)
curl -s https://mawaqit.rahmanr.com

# Dedicated fasting, suhoor & imsak schedule
curl -s https://mawaqit.rahmanr.com/fasting

# Night divisions & Tahajjud last third calculation
curl -s https://mawaqit.rahmanr.com/night

# White days (13th, 14th, 15th) & Day 1 moonsighting verification
curl -s https://mawaqit.rahmanr.com/white-days

# Hijri to Gregorian date converter
curl -s https://mawaqit.rahmanr.com/convert
curl -s https://mawaqit.rahmanr.com/convert/1448-05-13

# Location override via subpath
curl -s https://mawaqit.rahmanr.com/makkah
curl -s https://mawaqit.rahmanr.com/london
curl -s https://mawaqit.rahmanr.com/tokyo
```

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
 ├── theme        -> Terminal formatting styles (enum ThemeStyle)
 └── edge         -> Edge renderer & WebAssembly C-ABI layer (worker/src/lib.rs)
```

---

## Local CLI Quick Start & Features

### 1. Default Single-Line Output (for tmux / Waybar / Polybar)
```bash
mawaqit
# Singapore: Dhuhr 12:56 -> Asr 16:04 (-01:12:45) | ◔ 37% | Sunday, 21 Rabi' al-Thani 1448 AH
```

### 2. Verbose Table View (`-v`)
```bash
mawaqit -v
```
Displays a complete multi-section ASCII dashboard with:
* Real-time prayer milestones with active/passed/upcoming state indicators.
* Solar elevations, dawn/dusk twilight angles, and compass azimuth directions.
* Moon phase glyph, percentage illumination, and lunar cycle name.
* Active Hijri calendar date and month ledger offsets.

### 3. Custom Format Templating (`-f` / `--format`)
Compose custom one-liners for your shell prompt or window manager:
```bash
mawaqit -f "%location: %next in %remaining | %moon %moon_pct | Fajr: %fajr, Maghrib: %maghrib"
# Singapore: Asr in 01:12:45 | ◔ 37% | Fajr: 05:35, Maghrib: 18:57
```

### 4. Fasting Schedule & Live State Machine (`--fasting` / `--suhoor` / `--imsak`)
```bash
mawaqit --fasting
```
Calculates:
* Precautionary Imsak (configurable buffer, default: 10 mins).
* Suhoor cutoff (Fajr time).
* Fast-breaking / Iftar (Maghrib time).
* Total fasting span and night eating window.
* Live state machine: *Suhoor permitted*, *Imsak precautionary cutoff*, or *Fasting active*.

### 5. Multi-City Fasting Comparison (`--compare-fasting`)
Compare fasting duration across different latitudes and regional calculation methods:
```bash
mawaqit --compare-fasting "London,Tokyo,Makkah,Oslo"
```

### 6. Night Divisions & Tahajjud Last Third (`--night`)
```bash
mawaqit --night
```
Breaks down the night into canonical intervals:
* **First Third**: Early evening window.
* **Second Third**: Mid-night window.
* **Midnight (*Nisf al-Layl*)**: Juristic midnight calculation (sunset to Fajr or sunset to sunrise).
* **Last Third (*Thuluth al-Akhir*)**: The designated window for Tahajjud and Qiyam al-Layl.

### 7. White Days (*Ayyam al-Bid*) & Moonsighting Check (`--white-days`)
```bash
mawaqit --white-days
```
Shows:
* Day 1 of the active Hijri month (for moonsighting audit and verification).
* Active month ledger offset.
* Exact Gregorian dates for the 13th, 14th, and 15th of the month.

### 8. Hijri to Gregorian Date Converter (`--convert-hijri`)
Converts any Hijri day or specific month into its corresponding Gregorian date while applying all active month ledger overrides:
```bash
mawaqit --convert-hijri 13               # 13th of current month
mawaqit --convert-hijri 1448-05-13        # 13th Jumada al-Ula 1448 AH
```

### 9. Prohibited Prayer Times & Duha Window (`--prohibited` / `--duha`)
```bash
mawaqit --prohibited
```
Tracks the 3 forbidden times (*Awqat al-Nahy*) where voluntary prayers are prohibited:
1. **Sunrise (*Tulu' al-Shams*)**: From sunrise until the sun rises a spear's height (~15 mins).
2. **Solar Zenith (*Zawal*)**: Pre-Dhuhr solar zenith window (~10 mins before Dhuhr).
3. **Sunset (*Ghurub al-Shams*)**: When the sun turns amber until complete sunset.
Also displays the permissible **Duha / Ishraq** window.

### 10. Major Annual Islamic Sacred Stations (`--observances`)
```bash
mawaqit --observances
```
Displays canonical annual stations including Tasu'a, 'Ashura, Mid-Sha'ban, Ramadan, Laylat al-Qadr, Eid al-Fitr, Day of 'Arafah, Eid al-Adha, and Days of Tashriq.

### 11. Per-Month Islamic Calendar Ledger Override (`--taqwim-adjust`)
Handle local moonsighting variations cleanly without modifying calculation source code:
```bash
mawaqit --taqwim-adjust +1                              # Adjust current month by +1 day
mawaqit --taqwim-adjust -1 --taqwim-month 1448-05         # Adjust specific month
```
Automatically appends an audit entry to `taqwim_adjustments.log` and updates `config.toml`.

### 12. Calculation Method Selection (`--method`)
Select from canonical global calculation bodies and regional authorities:
```bash
mawaqit --method mwl         # Muslim World League (Fajr 18°, Isha 17°)
mawaqit --method singapore   # MUIS Singapore (Fajr 20°, Isha 18°)
mawaqit --method egyptian    # Egyptian General Authority of Survey (Fajr 19.5°, Isha 17.5°)
mawaqit --method isna        # Islamic Society of North America (Fajr 15°, Isha 15°)
mawaqit --method ummalqura   # Umm al-Qura, Makkah (Fajr 18.5°, Isha +90 min interval)
mawaqit --method karachi     # Univ. of Islamic Sciences, Karachi (Fajr 18°, Isha 18°)
mawaqit --method turkey      # Diyanet İşleri Başkanlığı, Turkey
mawaqit --method dubai       # Dubai / UAE Awqaf
```

### 13. Juristic Asr School / Madhab (`--madhab`)
Toggle the shadow multiplier used for calculating Asr prayer entrance:
```bash
mawaqit --madhab shafi       # Standard/Majority: Shadow length equals object height (1x shadow factor, default)
mawaqit --madhab hanafi      # Hanafi: Shadow length equals twice object height (2x shadow factor)
```

### 14. Custom Astronomical Twilight Angles (`--fajr-angle` & `--isha-angle`)
For high-latitude or custom observational guidelines, override exact solar depression angles:
```bash
mawaqit --fajr-angle 19.5 --isha-angle 17.5
```

> [!TIP]
> For a full list of all available CLI flags, subroutes, and customisation parameters, refer to the [CLI Flags & Options Table](#cli-flags--options-table) below.

---

## Configuration & Local Persistent Storage (`--save`)

When you use `mawaqit` on your local machine, you can persist your preferred location, calculation method, madhab, and UI style permanently to disk using `--save`:

```bash
# Save your hometown parameters once to your local machine
mawaqit --city London --method mwl --madhab hanafi --save
```

Subsequent runs of `mawaqit` anywhere in your terminal will automatically load your saved configuration without needing to pass flags again.

> [!NOTE]
> **Why does `--save` only work in the local CLI?**  
> The web emulator at `mawaqit.rahmanr.com` runs on stateless Cloudflare Workers WebAssembly at the edge with zero disk storage access. Writing persistent user files to `config.toml` is exclusive to the local CLI binary on your machine where it safely manages your local user environment.

Configuration is persisted across runs in standard TOML format:
* **Linux / macOS**: `~/.config/mawaqit/config.toml`
* **Windows**: `%APPDATA%\mawaqit\config.toml`

### Example `config.toml`:
```toml
# mawaqit persistent user configuration

[location]
location = "London"
latitude = 51.5074
longitude = -0.1278

[calculation]
method = "mwl"
madhab = "hanafi"
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

Each adjustment is logged with an audit trail in `taqwim_adjustments.log`:
```text
2026-10-04 14:10:00 | Month: 1448-03 | Offset: +1 | Reason: User CLI adjustment
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

## Testing

Run the automated test suite locally:
```bash
cargo test
```
All unit tests verify:
* Solar elevation & astronomical ephemeris boundary conditions.
* Moon illumination percentage bounds.
* Prayer sequence progression and post-Isha / pre-Fajr boundaries.
* Month-specific Taqwim ledger overrides.
* Prohibited times (*Awqat al-Nahy*) and Duha calculations.
* Fasting and Suhoor state transitions.

