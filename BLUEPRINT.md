# Project Blueprint & Implementation Phases

**Binary Name**: `mawaqit`  
**Repository**: `prayer-time-cli`

---

## 🗺️ Roadmap & Phase Tracker

- [x] **Phase 1: The Core Timekeeper Engine (`muwaqqit`)**
  - [x] Configure `Cargo.toml` with binary name `mawaqit` and dependencies (`salah`, `chrono`)
  - [x] Implement `Tawqit` (configuration for coordinates, method e.g. Singapore/MUIS, Madhab)
  - [x] Implement `Miqat` (individual prayer time representation) and `MawqutStatus`
  - [x] Implement `Muwaqqit` (the timekeeper engine calculating daily schedules and next prayer countdown)
  - [x] Build & run initial CLI verification: output today's prayer times for Singapore

- [x] **Phase 2: Celestial & Ephemeris Module (`falak` / `qamar`)**
  - [x] Implement offline Julian Day and Meeus lunar ephemeris calculation in `src/falak.rs`
  - [x] Calculate real-time Moon phase angle, illumination fraction, and synodic age
  - [x] Map Moon phases to clean geometric circle glyphs (`○`, `◔`, `◑`, `◕`, `●`, `◐`)
  - [x] Integrate Moon phase into default single-line output (`| ◐ 46%`)
  - [x] Add dedicated Celestial section in `--verbose` table output

- [x] **Phase 3: The Template & CLI Formatter (`bayan`)**
  - [x] Implement `wttr.in`-style format string token replacement (`src/bayan.rs`)
  - [x] Support compound tokens (`%remaining`, `%moon`, `%moon_pct`, `%city`, etc.) and short single-letter tokens (`%f`, `%d`, `%a`, `%m`, `%i`)
  - [x] Prevent token collision with layered precedence replacement
  - [x] Verified binary execution in Windows PowerShell (< 30ms cold-start)

- [ ] **Phase 4: Live Interactive Terminal UI (`shasha`)**
  - [ ] Integrate `ratatui` and `crossterm`
  - [ ] Design terminal layout: Header, Prayer Schedule Grid, Celestial Widget (Moon + Sun), Progress Bar
  - [ ] Implement 1-second tick loop with dynamic progress bar showing time elapsed/remaining
  - [ ] Hotkeys: `q` to quit, `m` to toggle format, `r` to refresh

- [ ] **Phase 5: Web-Enabled Microservice (`shabaka`)**
  - [ ] Integrate `axum` + `tokio`
  - [ ] Implement User-Agent negotiation: terminal clients (`curl`, `Invoke-RestMethod`) receive ANSI text; browsers receive clean page
  - [ ] Add IP-based geolocation fallback for `curl pray.in`
  - [ ] In-memory caching for zero-latency response
