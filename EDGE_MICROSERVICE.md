# Phase 4: `wttr.in`-Style Edge Microservice Architecture & Fix Tracker

**Service Domain**: `https://mawaqit.rahmanr.com`  
**Deployment Infrastructure**: Cloudflare Workers (WebAssembly `wasm32-unknown-unknown` + ES Module entrypoint)  
**Tracking Repository**: [`riyashdeenr/-prayer-time-cli`](https://github.com/riyashdeenr/-prayer-time-cli.git)

---

## 🎯 Architecture Summary

- **Core Engine Preservation**: The local machine CLI application (`mawaqit`) remains the primary, zero-regression target. All calculations (astronomy, Hijri Taqwim, prayer boundaries, fasting schedules) reside in the decoupled `prayer_time_cli` core library (`src/lib.rs`).
- **Edge Deployment**: Cloudflare Workers executes the pure Rust engine compiled to WebAssembly with zero filesystem or OS system calls.
- **Client Negotiation**:
  - Terminal Clients (`curl`, `httpie`, `wget`, `powershell`, `invoke-webrequest`, `invoke-restmethod`, `irm`, `iwr`) receive pure `text/plain; charset=utf-8` with UTF-8/ANSI terminal tables.
  - Web Browsers receive HTML/CSS presentation.

---

## 🛠️ Issues Resolved & Verified

- [x] **Cloudflare Git CI/CD Build Setup**: Configured root `wrangler.toml` with `[[rules]] type = "CompiledWasm"` and committed precompiled `worker/mawaqit_worker.wasm` so Cloudflare deploys immediately on git push.
- [x] **WebAssembly FFI Runtime Compatibility**: Replaced `wasm-bindgen` JS dependencies with a clean C-ABI FFI layer (`mawaqit_alloc`, `mawaqit_free`, `mawaqit_free_string`) and universal proxy resolver to eliminate `__wbindgen_placeholder__` errors.
- [x] **Live Unix Epoch Passing**: Passed BigInt epoch seconds (`Math.floor(Date.now() / 1000)`) from Cloudflare edge into Rust to prevent 1970 fallback.
- [x] **Timezone Display Synchronization**: Adjusted display prayer times, fasting boundaries, and observances by Cloudflare edge timezone offset (`cf.timezone` / `tz_shift`) to match local wall-clock time.
- [x] **PowerShell / `Invoke-WebRequest` Identification**: Added `powershell`, `invoke-webrequest`, `invoke-restmethod`, `irm`, and `iwr` to the terminal client detector list. PowerShell commands now receive pure `text/plain` UTF-8 output with zero security risk prompts and zero Cloudflare HTML/script injections.
- [x] **Token Enhancements**: Added `%moon_symbol` token support across both CLI and edge template engines.

---

## 📋 Open Problem: Browser (HTML) Rendering

### Issue Description
When a web browser (Chrome, Safari, Edge, Firefox) visits `https://mawaqit.rahmanr.com`, the current placeholder `<pre>` wrapper suffers from:
1. **ANSI Code Artifacts / Glitches**: Raw terminal color escape sequences (`\x1b[90m`, `\x1b[38;2;...`) distort browser text or show up as control artifacts.
2. **Glyph & Monospace Misalignment**: Default browser fonts lack consistent tabular numbers and wide Unicode glyph fallbacks for moon icons (`◔`, `●`, `○`), causing table columns to become jagged and uneven.
3. **Line Wrapping Distortions**: On smaller screens or mobile viewports, pre-wrapped lines break mid-table.
4. **Third-Party Script Injections**: Cloudflare injects analytics and challenge scripts into HTML responses, making raw DOM dumps messy.

---

## 💡 Evaluation of Solutions for Browser Visitors

### Option 1: High-Fidelity Web Terminal (True `wttr.in` Model)
- **Concept**: Treat the web browser as an authentic, crisp hacker terminal.
- **Implementation**:
  - Parse ANSI color escapes into semantic CSS classes (`.active`, `.dim`, `.bright`).
  - Use modern developer fonts (`JetBrains Mono` / `Cascadia Code` / `Fira Code`) with `font-feature-settings: "tnum" 1` and explicit Unicode symbol fallbacks.
  - Present inside a macOS/Unix-styled dark terminal card with header controls, click-to-copy curl snippets, and horizontal scrolling for mobile.
- **Pros**: 100% unified experience with CLI; lightweight (< 25 KB); iconic developer aesthetic.
- **Cons**: Still renders as a terminal interface, which non-technical mobile users might find raw.

---

### Option 2: Dual Personality (Modern Card Dashboard for Browsers)
- **Concept**: Split rendering logic: terminals get ASCII/ANSI; browsers get a modern responsive web dashboard.
- **Implementation**:
  - Hero section: Active prayer status, live countdown ring/progress bar.
  - 6-card prayer grid (Fajr, Sunrise, Dhuhr, Asr, Maghrib, Isha) highlighting current vs upcoming.
  - Lunar & Solar cards: Real SVG moon phase illustration with illuminated fraction and sun altitude angle.
  - Fasting & Observances section: Imsak, Suhoor cutoff, Iftar, next White Days.
  - Developer bar: Prominent "Copy CLI Command" (`curl mawaqit.rahmanr.com`) section for terminal users.
- **Pros**: Completely eliminates monospace/font width alignment issues; looks stunning and mobile-friendly on phones.
- **Cons**: Requires writing and maintaining a standalone HTML/CSS template within the Worker subcrate.

---

### Option 3: Terminal-First Developer Landing Page
- **Concept**: Dedicate the web page purely to CLI documentation and quick-reference usage.
- **Implementation**:
  - Explain what `mawaqit` is.
  - Provide interactive copy blocks for all endpoints (`/London`, `/fasting`, `/night`, `?format=...`).
  - Include an interactive preview terminal showing sample live output.
- **Pros**: Clear identity as a developer tool; sets the right expectations.
- **Cons**: Casual visitors cannot view prayer times without copying a terminal command.

---

### Option 4: Pure `text/plain` Everywhere (Zero HTML)
- **Concept**: Eliminate HTML entirely.
- **Implementation**:
  - Return `Content-Type: text/plain; charset=utf-8` to all requests.
  - Strip ANSI color escape codes when the client is a browser to ensure clean text.
- **Pros**: Zero HTML/CSS code; zero Cloudflare script injections.
- **Cons**: Relies on browser default monospace font rendering; no styling or branding.

---

## 🏆 Selected Architecture: Option 5 — The Interactive Web TUI Simulator
*(Modeled after the terminal aesthetic of `bio.rahmanr.com`)*

### Design Philosophy
- **Aesthetic**: Pitch Black (`#000000`), Zinc 950 (`#09090b`), Dark Border (`rgba(255,255,255,0.08)` / `border-zinc-800`), Amber Gold accents (`#f59e0b`), and Emerald Green indicators (`#10b981`).
- **Terminal Chrome**: Traffic light window controls (🔴 🟡 🟢), title bar `mawaqit - universal terminal prayer engine`, and active pulse badge (`ONLINE`).
- **Interactive Shell**:
  - Live command input with `$ ` prompt.
  - Auto-scrollable terminal output buffer (`overflow-y-auto`).
  - **Tab auto-completion** (`help`, `London`, `fasting`, `night`, `white-days`, `prohibited`, `observances`, `clear`).
  - **`↑ / ↓` Command history navigation**.
  - One-click shortcut pills for rapid execution (`[help]`, `[fasting]`, `[night]`, `[white-days]`, `[London]`, `[Makkah]`).
  - **Dynamic "Copy curl Command" pill** that automatically generates the exact curl command corresponding to the user's active query.

### Functional Boundary: What is Included vs Excluded

| Capability | In Web TUI Simulator? | Rationale |
| :--- | :---: | :--- |
| **City Lookup (`London`, `Tokyo`, `Makkah`)** | ✅ YES | Pure Wasm lookup, instant response. |
| **Coordinates Lookup (`1.35,103.82`)** | ✅ YES | Evaluated directly by Wasm engine. |
| **Fasting & Imsak (`fasting`, `suhoor`)** | ✅ YES | Full schedule calculated live. |
| **Night Thirds & Tahajjud (`night`)** | ✅ YES | Midnight & last third calculation. |
| **White Days (`white-days`)** | ✅ YES | 13-15 Ayyam al-Bid dates. |
| **Prohibited Times & Duha (`prohibited`)** | ✅ YES | Awqat al-Nahy + Duha window. |
| **Observances (`observances`)** | ✅ YES | Canonical sacred annual events. |
| **Method / Madhab switches (`method ...`)** | ✅ YES | Regional calculation configuration. |
| **Custom Format Playground (`format ...`)** | ✅ YES | Instant status-bar format tester. |
| **Persistent Config File (`--save`)** | ❌ NO | Cloudflare Workers is stateless. Explains: *"Config saving is a local CLI feature. Run 'mawaqit --save' on your machine."* |
| **1-Second Loop / Progress Bar** | ❌ NO | Reserved for Phase 5 (`ratatui`). Web TUI is a request/response shell to avoid websocket spam. |

---

## 🛡️ Zero-Regression Impact Assessment

| File | Status | Regression Risk |
| :--- | :---: | :--- |
| `src/main.rs` | **Untouched** | 0% (Local offline CLI remains identical) |
| `src/lib.rs` | **Untouched** | 0% (Core library remains identical) |
| `tests/core_tests.rs` | **Untouched** | 0% (All 12 unit/integration tests passing) |
| `src/edge.rs` | **Untouched** | 0% (Existing Wasm engine handles queries) |
| `worker/src/lib.rs` | **Untouched** | 0% (FFI exports remain unchanged) |
| `worker/worker.js` | **Modified** | 0% (Only affects browser HTML responses; terminal curl bypasses HTML completely) |

---

## 🚀 Execution Checklist for Next Session
1. [ ] Update `worker/worker.js` to render the `bio.rahmanr.com`-style interactive TUI simulator.
2. [ ] Embed the lightweight JavaScript shell (Tab completion, history buffer, fetch bridge to `mawaqit_worker.wasm`).
3. [ ] Test in desktop and mobile browsers.
4. [ ] Run `cargo test` and local `mawaqit --verbose` to confirm zero regressions.
5. [ ] Commit and push to deploy live.

