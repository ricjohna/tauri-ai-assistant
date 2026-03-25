use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeoResult>>,
}

#[derive(Debug, Deserialize)]
struct GeoResult {
    #[serde(rename = "latitude")]
    lat: f64,
    #[serde(rename = "longitude")]
    lon: f64,
    #[serde(rename = "name")]
    city_name: String,
    #[serde(rename = "country")]
    country: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    #[serde(rename = "current")]
    current: CurrentWeather,
    #[serde(rename = "current_units")]
    units: CurrentUnits,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    #[serde(rename = "temperature_2m")]
    temperature: f64,
    #[serde(rename = "relative_humidity_2m")]
    humidity: f64,
    #[serde(rename = "weather_code")]
    code: i32,
    #[serde(rename = "wind_speed_10m")]
    wind: f64,
}

#[derive(Debug, Deserialize)]
struct CurrentUnits {
    #[serde(rename = "temperature_2m")]
    temp_unit: String,
    #[serde(rename = "wind_speed_10m")]
    wind_unit: String,
}

fn weather_code_to_text(code: i32) -> &'static str {
    match code {
        0 => "Clear",
        1 | 2 | 3 => "Partly Cloudy",
        45 | 48 => "Foggy",
        51 | 53 | 55 => "Drizzle",
        56 | 57 => "Freezing Drizzle",
        61 | 63 | 65 => "Rain",
        66 | 67 => "Freezing Rain",
        71 | 73 | 75 => "Snow",
        77 => "Snow Grains",
        80 | 81 | 82 => "Rain Showers",
        85 | 86 => "Snow Showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm with Hail",
        _ => "Unknown",
    }
}

pub fn is_weather_command(input: &str) -> bool {
    is_explicit_weather_command(input) || is_conversational_weather_command(input)
}

pub fn is_explicit_weather_command(input: &str) -> bool {
    input.to_lowercase().starts_with("/weather")
}

pub fn is_conversational_weather_command(input: &str) -> bool {
    let input_lower = input.to_lowercase();
    !input_lower.starts_with("/weather") && extract_city(input).is_some()
}

pub fn extract_city(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    
    let patterns = [
        r"(?i)weather\s+(?:in|at|for)?\s*(.+?)(?:\?|$)",
        r"(?i)temperature\s+(?:in|at|for)?\s*(.+?)(?:\?|$)",
        r"(?i)forecast\s+(?:in|at|for)?\s*(.+?)(?:\?|$)",
        r"(?i)how(?:'s| is) (?:the )?weather\s+(?:in|at|for)?\s*(.+?)(?:\?|$)",
        r"(?i)what(?:'s| is) (?:the )?temperature\s+(?:in|at|for)?\s*(.+?)(?:\?|$)",
        r"(?i)is it\s+(?:hot|cold|raining|sunny)\s+(?:in|at|for)?\s*(.+?)(?:\?|$)",
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(&input_lower) {
                if let Some(city) = caps.get(1) {
                    let city = city.as_str().trim();
                    if !city.is_empty() && city.len() > 1 {
                        return Some(city.split(',').next().unwrap_or(city).trim().to_string());
                    }
                }
            }
        }
    }
    None
}

pub async fn get_weather(city: &str) -> Result<String, String> {
    let city_encoded = urlencoding::encode(city);
    let geocode_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city_encoded
    );

    let geocode_response = reqwest::get(&geocode_url)
        .await
        .map_err(|e| e.to_string())?;

    let geocode_json: GeocodingResponse = geocode_response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let geo = geocode_json
        .results
        .ok_or_else(|| format!("City '{}' not found", city))?
        .into_iter()
        .next()
        .ok_or_else(|| format!("City '{}' not found", city))?;

    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m",
        geo.lat, geo.lon
    );

    let weather_response = reqwest::get(&weather_url)
        .await
        .map_err(|e| e.to_string())?;

    let weather: WeatherResponse = weather_response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let condition = weather_code_to_text(weather.current.code);
    let location = match &geo.country {
        Some(c) => format!("{}, {}", geo.city_name, c),
        None => geo.city_name,
    };

    Ok(format!(
        "Weather in {}\nTemperature: {:.1}{}\nCondition: {}\nHumidity: {:.0}%\nWind: {:.1}{}",
        location,
        weather.current.temperature,
        weather.units.temp_unit,
        condition,
        weather.current.humidity,
        weather.current.wind,
        weather.units.wind_unit
    ))
}

pub async fn get_weather_for_ai(city: &str) -> Result<(String, String), String> {
    let city_encoded = urlencoding::encode(city);
    let geocode_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city_encoded
    );

    let geocode_response = reqwest::get(&geocode_url)
        .await
        .map_err(|e| e.to_string())?;

    let geocode_json: GeocodingResponse = geocode_response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let geo = geocode_json
        .results
        .ok_or_else(|| format!("City '{}' not found", city))?
        .into_iter()
        .next()
        .ok_or_else(|| format!("City '{}' not found", city))?;

    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m",
        geo.lat, geo.lon
    );

    let weather_response = reqwest::get(&weather_url)
        .await
        .map_err(|e| e.to_string())?;

    let weather: WeatherResponse = weather_response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let condition = weather_code_to_text(weather.current.code);
    
    let location = match &geo.country {
        Some(c) => format!("{}, {}", geo.city_name, c),
        None => geo.city_name,
    };

    let weather_summary = format!(
        "{}, {}°C, {}, {}% humidity, {} wind",
        location,
        weather.current.temperature,
        condition,
        weather.current.humidity as i32,
        weather.current.wind
    );

    Ok((location, weather_summary))
}
