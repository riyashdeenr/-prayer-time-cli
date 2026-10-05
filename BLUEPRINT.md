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

- [x] **Phase 4: Web-Enabled Edge Microservice (`shabaka` / `wttr.in`-style)**
  - [x] Decouple core calculation library from filesystem/OS dependencies
  - [x] Compile pure WebAssembly module for Cloudflare Workers edge deployment
  - [x] Implement Cloudflare edge header geolocation resolution (`cf-iplatitude`, `cf-iplongitude`, `cf-ipcity`, `cf-timezone`)
  - [x] Implement routing: city endpoints (`/London`), coordinate endpoints (`/1.35,103.82`), special subroutes (`/fasting`, `/night`, `/white-days`, `/prohibited`, `/observances`)
  - [x] Client negotiation: terminal clients (`curl`, `wget`, `httpie`, `powershell`, `invoke-restmethod`) receive pure UTF-8 plain text; browsers receive HTML
  - [x] Support status-bar and tmux format strings (`?format=...`)
  - [x] Deployed and verified live at `https://mawaqit.rahmanr.com`
  - [ ] Complete browser HTML/CSS rendering overhaul (See [EDGE_MICROSERVICE.md](EDGE_MICROSERVICE.md))

- [ ] **Phase 5: Live Interactive Terminal UI (`shasha`)**
  - [ ] Integrate `ratatui` and `crossterm`
  - [ ] Design terminal layout: Header, Prayer Schedule Grid, Celestial Widget (Moon + Sun), Progress Bar
  - [ ] Implement 1-second tick loop with dynamic progress bar showing time elapsed/remaining
  - [ ] Hotkeys: `q` to quit, `m` to toggle format, `r` to refresh

