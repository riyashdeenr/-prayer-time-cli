use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Persistent configuration file handler (~/.config/mawaqit/config.toml or AppData\Roaming\mawaqit\config.toml)
pub struct ConfigManager;

#[derive(Debug, Clone, Default)]
pub struct SavedConfig {
    pub location_name: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub method: Option<String>,
    pub madhab: Option<String>,
    pub fajr_angle: Option<f64>,
    pub isha_angle: Option<f64>,
    pub style: Option<String>,
    pub moon_style: Option<String>,
    pub taqwim_default_offset: i32,
    pub taqwim_adjustments: HashMap<String, i32>,
    pub night_basis: Option<String>,
    pub imsak_buffer_minutes: Option<i64>,
}

impl ConfigManager {
    /// Get the path to mawaqit's config.toml
    pub fn config_path() -> PathBuf {
        let base_dir = if let Ok(app_data) = std::env::var("APPDATA") {
            PathBuf::from(app_data).join("mawaqit")
        } else if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            PathBuf::from(home).join(".config").join("mawaqit")
        } else {
            PathBuf::from(".").join(".mawaqit")
        };

        base_dir.join("config.toml")
    }

    /// Load saved configuration if file exists
    pub fn load() -> Option<SavedConfig> {
        let path = Self::config_path();
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).ok()?;
        let mut config = SavedConfig::default();

        let mut current_section = "";

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = &line[1..line.len() - 1];
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim().trim_matches('"').trim_matches('\'');
                let val = val.trim().trim_matches('"').trim_matches('\'');

                if current_section == "taqwim.adjustments" {
                    if let Ok(offset_val) = val.parse::<i32>() {
                        config.taqwim_adjustments.insert(key.to_string(), offset_val);
                    }
                    continue;
                }

                match key {
                    "location" | "city" => config.location_name = Some(val.to_string()),
                    "latitude" | "lat" => config.latitude = val.parse().ok(),
                    "longitude" | "lon" => config.longitude = val.parse().ok(),
                    "method" => config.method = Some(val.to_string()),
                    "madhab" => config.madhab = Some(val.to_string()),
                    "fajr_angle" => config.fajr_angle = val.parse().ok(),
                    "isha_angle" => config.isha_angle = val.parse().ok(),
                    "style" | "theme" => config.style = Some(val.to_string()),
                    "moon" | "moon_style" => config.moon_style = Some(val.to_string()),
                    "default_offset" => config.taqwim_default_offset = val.parse().unwrap_or(0),
                    "night_basis" | "night" => config.night_basis = Some(val.to_string()),
                    "imsak_buffer" | "imsak_mins" | "imsak_minutes" => config.imsak_buffer_minutes = val.parse().ok(),
                    _ => {}
                }
            }
        }

        Some(config)
    }

    /// Save configuration to disk
    pub fn save(config: &SavedConfig) -> Result<PathBuf, String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let mut toml = String::new();
        toml.push_str("# mawaqit persistent user configuration\n\n[location]\n");
        if let Some(name) = &config.location_name {
            toml.push_str(&format!("location = \"{}\"\n", name));
        }
        if let Some(lat) = config.latitude {
            toml.push_str(&format!("latitude = {}\n", lat));
        }
        if let Some(lon) = config.longitude {
            toml.push_str(&format!("longitude = {}\n", lon));
        }

        toml.push_str("\n[calculation]\n");
        if let Some(m) = &config.method {
            toml.push_str(&format!("method = \"{}\"\n", m));
        }
        if let Some(madhab) = &config.madhab {
            toml.push_str(&format!("madhab = \"{}\"\n", madhab));
        }
        if let Some(fa) = config.fajr_angle {
            toml.push_str(&format!("fajr_angle = {}\n", fa));
        }
        if let Some(ia) = config.isha_angle {
            toml.push_str(&format!("isha_angle = {}\n", ia));
        }
        if let Some(nb) = &config.night_basis {
            toml.push_str(&format!("night_basis = \"{}\"\n", nb));
        }
        if let Some(imsak) = config.imsak_buffer_minutes {
            toml.push_str(&format!("imsak_buffer = {}\n", imsak));
        }

        toml.push_str("\n[ui]\n");
        if let Some(s) = &config.style {
            toml.push_str(&format!("style = \"{}\"\n", s));
        }
        if let Some(ms) = &config.moon_style {
            toml.push_str(&format!("moon = \"{}\"\n", ms));
        }

        toml.push_str("\n[taqwim]\n");
        toml.push_str(&format!("default_offset = {}\n", config.taqwim_default_offset));

        if !config.taqwim_adjustments.is_empty() {
            toml.push_str("\n[taqwim.adjustments]\n");
            for (month_key, offset_val) in &config.taqwim_adjustments {
                toml.push_str(&format!("\"{}\" = {:+}\n", month_key, offset_val));
            }
        }

        fs::write(&path, toml).map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(path)
    }
}
