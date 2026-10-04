use salah::prelude::{Configuration, Coordinates, Madhab, Method, Parameters};

/// Calculation and location settings (Verbal Noun: tawqit)
#[derive(Debug, Clone)]
pub struct Tawqit {
    pub location_name: String,
    pub coordinates: Coordinates,
    pub method: Method,
    pub madhab: Madhab,
    pub custom_fajr_angle: Option<f64>,
    pub custom_isha_angle: Option<f64>,
}

impl Tawqit {
    /// Default configuration for Singapore using MUIS parameters
    #[allow(dead_code)]
    pub fn singapore() -> Self {
        Self {
            location_name: "Singapore".to_string(),
            coordinates: Coordinates::new(1.3521, 103.8198),
            method: Method::Singapore,
            madhab: Madhab::Shafi,
            custom_fajr_angle: None,
            custom_isha_angle: None,
        }
    }

    /// Custom location, coordinates, method, and madhab
    #[allow(dead_code)]
    pub fn custom(
        name: impl Into<String>,
        latitude: f64,
        longitude: f64,
        method: Method,
        madhab: Madhab,
    ) -> Self {
        Self {
            location_name: name.into(),
            coordinates: Coordinates::new(latitude, longitude),
            method,
            madhab,
            custom_fajr_angle: None,
            custom_isha_angle: None,
        }
    }

    /// Lookup well-known international cities to lat, lon, standard name, and standard UTC offset (hours)
    pub fn lookup_city(name: &str) -> Option<(&'static str, f64, f64, i32)> {
        match name.to_lowercase().replace([' ', '-', '_', '.'], "").as_str() {
            "singapore" | "sg" => Some(("Singapore", 1.3521, 103.8198, 8)),
            "london" | "uk" => Some(("London", 51.5074, -0.1278, 1)), // BST in Oct / GMT
            "makkah" | "mecca" => Some(("Makkah", 21.4225, 39.8262, 3)),
            "madinah" | "medina" => Some(("Madinah", 24.5247, 39.5692, 3)),
            "kualalumpur" | "kl" => Some(("Kuala Lumpur", 3.1390, 101.6869, 8)),
            "jakarta" => Some(("Jakarta", -6.2088, 106.8456, 7)),
            "cairo" => Some(("Cairo", 30.0444, 31.2357, 3)),
            "istanbul" => Some(("Istanbul", 41.0082, 28.9784, 3)),
            "dubai" => Some(("Dubai", 25.2048, 55.2708, 4)),
            "karachi" => Some(("Karachi", 24.8607, 67.0011, 5)),
            "tokyo" => Some(("Tokyo", 35.6762, 139.6503, 9)),
            "newyork" | "nyc" => Some(("New York", 40.7128, -74.0060, -4)), // EDT in Oct
            "toronto" => Some(("Toronto", 43.6532, -79.3832, -4)),
            "sydney" => Some(("Sydney", -33.8688, 151.2093, 11)), // AEDT in Oct
            "paris" => Some(("Paris", 48.8566, 2.3522, 2)), // CEST in Oct
            "berlin" => Some(("Berlin", 52.5200, 13.4050, 2)), // CEST in Oct
            "oslo" => Some(("Oslo", 59.9139, 10.7522, 2)), // CEST in Oct
            "reykjavik" => Some(("Reykjavik", 64.1466, -21.9426, 0)),
            "johannesburg" => Some(("Johannesburg", -26.2041, 28.0473, 2)),
            _ => None,
        }
    }

    /// Parse calculation method from string name
    pub fn parse_method(s: &str) -> Option<Method> {
        match s.to_lowercase().replace(['-', ' ', '_'], "").as_str() {
            "singapore" | "muis" | "sg" => Some(Method::Singapore),
            "muslimworldleague" | "mwl" => Some(Method::MuslimWorldLeague),
            "egypt" | "egyptian" => Some(Method::Egyptian),
            "karachi" => Some(Method::Karachi),
            "ummalqura" | "makkah" | "saudi" => Some(Method::UmmAlQura),
            "dubai" | "uae" => Some(Method::Dubai),
            "qatar" => Some(Method::Qatar),
            "kuwait" => Some(Method::Kuwait),
            "moonsighting" | "moonsightingcommittee" => Some(Method::MoonsightingCommittee),
            "northamerica" | "isna" | "us" => Some(Method::NorthAmerica),
            "tehran" => Some(Method::Tehran),
            "turkey" | "diyanet" => Some(Method::Turkey),
            _ => None,
        }
    }

    /// Parse madhab from string name (defaults to Shafi if not specified or unrecognized)
    pub fn parse_madhab(s: &str) -> Madhab {
        match s.to_lowercase().as_str() {
            "hanafi" | "h" => Madhab::Hanafi,
            _ => Madhab::Shafi,
        }
    }

    /// Auto-resolve the best default method based on coordinates
    pub fn resolve_regional_method(lat: f64, lon: f64) -> Method {
        // Singapore / Malaysia / Indonesia bounding box approx
        if lat >= -11.0 && lat <= 7.0 && lon >= 95.0 && lon <= 141.0 {
            Method::Singapore
        }
        // Arabian Peninsula / Gulf
        else if lat >= 12.0 && lat <= 32.0 && lon >= 34.0 && lon <= 60.0 {
            Method::UmmAlQura
        }
        // North America (US & Canada)
        else if lat >= 24.0 && lat <= 70.0 && lon >= -170.0 && lon <= -50.0 {
            Method::NorthAmerica
        }
        // Egypt / North Africa
        else if lat >= 20.0 && lat <= 32.0 && lon >= 24.0 && lon <= 37.0 {
            Method::Egyptian
        }
        // Pakistan / India
        else if lat >= 23.0 && lat <= 37.0 && lon >= 60.0 && lon <= 78.0 {
            Method::Karachi
        }
        // Default international standard fallback: Muslim World League
        else {
            Method::MuslimWorldLeague
        }
    }

    /// Convert into salah Parameters, applying madhab and optional custom twilight angle overrides
    pub fn to_salah_params(&self) -> Parameters {
        let mut params = Configuration::with(self.method, self.madhab);

        if let Some(f_angle) = self.custom_fajr_angle {
            params.fajr_angle = f_angle;
        }

        if let Some(i_angle) = self.custom_isha_angle {
            params.isha_angle = i_angle;
        }

        params
    }
}
