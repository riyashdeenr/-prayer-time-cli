use prayer_time_cli::{render_edge_response, EdgeRenderParams, EdgeSubroute, Tawqit};
use salah::prelude::{Madhab, Method};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int};

#[no_mangle]
pub extern "C" fn mawaqit_alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn mawaqit_free(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        let _ = Vec::from_raw_parts(ptr, 0, size);
    }
}

#[no_mangle]
pub unsafe extern "C" fn mawaqit_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

#[no_mangle]
pub unsafe extern "C" fn process_edge_request(
    pathname_ptr: *const c_char,
    search_params_ptr: *const c_char,
    user_agent_ptr: *const c_char,
    cf_city_ptr: *const c_char,
    has_coords: c_int,
    cf_lat: c_double,
    cf_lon: c_double,
    cf_tz_offset_hours: c_int,
    utc_timestamp_sec: i64,
) -> *mut c_char {

    let pathname = if pathname_ptr.is_null() { "" } else { CStr::from_ptr(pathname_ptr).to_str().unwrap_or("") };
    let search_params = if search_params_ptr.is_null() { "" } else { CStr::from_ptr(search_params_ptr).to_str().unwrap_or("") };
    let user_agent = if user_agent_ptr.is_null() { "" } else { CStr::from_ptr(user_agent_ptr).to_str().unwrap_or("") };
    let cf_city = if cf_city_ptr.is_null() { None } else { CStr::from_ptr(cf_city_ptr).to_str().ok().map(|s| s.to_string()) };

    let is_terminal = is_terminal_client(user_agent);
    let clean_path = pathname.trim_matches('/');

    if clean_path == ":help" || clean_path == "help" {
        let params = EdgeRenderParams {
            location_name: "Universal".to_string(),
            latitude: 0.0,
            longitude: 0.0,
            timezone_offset_hours: 0,
            method: Method::MuslimWorldLeague,
            madhab: Madhab::Shafi,
            custom_fajr_angle: None,
            custom_isha_angle: None,
            format_template: None,
            subroute: EdgeSubroute::Help,
            is_terminal,
        };
        let body = render_edge_response(&params, utc_timestamp_sec as i64);
        let res = format_client_output(&body, is_terminal, "mawaqit - Help");
        return CString::new(res).unwrap_or_default().into_raw();
    }

    let (subroute, target_location) = match clean_path {
        "fasting" | "suhoor" | "imsak" => (EdgeSubroute::Fasting, None),
        "night" | "tahajjud" => (EdgeSubroute::Night, None),
        "white-days" | "bid" => (EdgeSubroute::WhiteDays, None),
        "prohibited" | "duha" => (EdgeSubroute::Prohibited, None),
        "observances" | "annual" | "events" => (EdgeSubroute::Observances, None),
        path if !path.is_empty() => (EdgeSubroute::DefaultTable, Some(path)),
        _ => (EdgeSubroute::DefaultTable, None),
    };

    let (location_name, latitude, longitude, tz_offset) = if let Some(loc_arg) = target_location {
        if loc_arg.contains(',') {
            let parts: Vec<&str> = loc_arg.split(',').collect();
            let lat = parts.get(0).and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.3521);
            let lon = parts.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(103.8198);
            let tz = if cf_tz_offset_hours != -999 { cf_tz_offset_hours } else { 8 };
            (format!("{:.2},{:.2}", lat, lon), lat, lon, tz)
        } else if let Some((std_name, lat, lon, city_tz)) = Tawqit::lookup_city(loc_arg) {
            (std_name.to_string(), lat, lon, city_tz)
        } else {
            let city = cf_city.unwrap_or_else(|| "Current Location".to_string());
            let lat = if has_coords == 1 { cf_lat } else { 1.3521 };
            let lon = if has_coords == 1 { cf_lon } else { 103.8198 };
            let tz = if cf_tz_offset_hours != -999 { cf_tz_offset_hours } else { 8 };
            (city, lat, lon, tz)
        }
    } else {
        let city = cf_city.unwrap_or_else(|| "Singapore".to_string());
        let lat = if has_coords == 1 { cf_lat } else { 1.3521 };
        let lon = if has_coords == 1 { cf_lon } else { 103.8198 };
        let tz = if cf_tz_offset_hours != -999 { cf_tz_offset_hours } else { 8 };
        (city, lat, lon, tz)
    };

    let query_map = parse_query_string(search_params);
    let format_template = query_map.get("format").cloned();
    let method = match query_map.get("method").map(|s| s.to_lowercase()).as_deref() {
        Some("singapore") | Some("muis") => Method::Singapore,
        Some("mwl") | Some("muslimworldleague") => Method::MuslimWorldLeague,
        Some("ummalqura") | Some("makkah") => Method::UmmAlQura,
        Some("isna") | Some("northamerica") => Method::NorthAmerica,
        Some("egyptian") | Some("egypt") => Method::Egyptian,
        Some("karachi") => Method::Karachi,
        Some("tehran") => Method::Tehran,
        Some("turkey") => Method::Turkey,
        Some("dubai") => Method::Dubai,
        Some("qatar") => Method::Qatar,
        Some("kuwait") => Method::Kuwait,
        Some("moonsighting") => Method::MoonsightingCommittee,
        _ => Tawqit::resolve_regional_method(latitude, longitude),
    };

    let madhab = match query_map.get("madhab").map(|s| s.to_lowercase()).as_deref() {
        Some("hanafi") | Some("2") => Madhab::Hanafi,
        _ => Madhab::Shafi,
    };

    let custom_fajr_angle = query_map.get("fajr-angle").and_then(|s| s.parse::<f64>().ok());
    let custom_isha_angle = query_map.get("isha-angle").and_then(|s| s.parse::<f64>().ok());

    let params = EdgeRenderParams {
        location_name: location_name.clone(),
        latitude,
        longitude,
        timezone_offset_hours: tz_offset,
        method,
        madhab,
        custom_fajr_angle,
        custom_isha_angle,
        format_template,
        subroute,
        is_terminal,
    };

    let raw_output = render_edge_response(&params, utc_timestamp_sec);
    let formatted = format_client_output(&raw_output, is_terminal, &format!("mawaqit - {}", location_name));
    CString::new(formatted).unwrap_or_default().into_raw()
}


fn is_terminal_client(user_agent: &str) -> bool {
    let ua = user_agent.to_lowercase();
    ua.contains("curl")
        || ua.contains("httpie")
        || ua.contains("wget")
        || ua.contains("fetch")
        || ua.contains("aria2")
}

fn parse_query_string(qs: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let clean = qs.trim_start_matches('?');
    for pair in clean.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut kv = pair.splitn(2, '=');
        if let Some(key) = kv.next() {
            let val = kv.next().unwrap_or("");
            let decoded_val = val.replace('+', " ").replace("%20", " ");
            map.insert(key.to_string(), decoded_val);
        }
    }
    map
}

fn format_client_output(body: &str, is_terminal: bool, title: &str) -> String {
    if is_terminal {
        body.to_string()
    } else {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{}</title>
    <style>
        body {{
            background-color: #0d1117;
            color: #c9d1d9;
            font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            padding: 1.5rem;
            box-sizing: border-box;
        }}
        pre {{
            background-color: #161b22;
            border: 1px solid #30363d;
            border-radius: 8px;
            padding: 1.5rem 2rem;
            line-height: 1.5;
            font-size: 0.95rem;
            box-shadow: 0 10px 30px rgba(0,0,0,0.5);
            max-width: 100%;
            overflow-x: auto;
            white-space: pre-wrap;
            word-wrap: break-word;
        }}
        .tip {{
            color: #8b949e;
            margin-top: 1rem;
            font-size: 0.85rem;
            text-align: center;
        }}
    </style>
</head>
<body>
    <div>
        <pre>{}</pre>
        <div class="tip">Designed for curl: <code>curl mawaqit.rahmanr.com</code></div>
    </div>
</body>
</html>"#,
            title,
            body.replace('<', "&lt;").replace('>', "&gt;")
        )
    }
}
