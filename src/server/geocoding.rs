use crate::models::LocationSuggestion;
use leptos::server_fn::ServerFnError;

#[derive(serde::Deserialize)]
struct NominatimResult {
    lat: String,
    lon: String,
    display_name: Option<String>,
    address: Option<NominatimAddress>,
}

#[derive(serde::Deserialize)]
struct NominatimAddress {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    country: Option<String>,
}

/// Geocode a free-text location query to (latitude, longitude) via Nominatim.
/// Returns Ok(None) if no results found.
pub async fn geocode_query(query: &str) -> Result<Option<(f64, f64)>, ServerFnError> {
    if query.trim().is_empty() {
        return Ok(None);
    }

    let url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1",
        urlencoding::encode(query.trim())
    );

    let client = reqwest::Client::new();
    let results: Vec<NominatimResult> = client
        .get(&url)
        .header("User-Agent", "mybd/1.0")
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Geocoding request failed: {e}")))?
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Geocoding parse failed: {e}")))?;

    let coords = results.first().and_then(|r| {
        let lat = r.lat.parse::<f64>().ok()?;
        let lon = r.lon.parse::<f64>().ok()?;
        Some((lat, lon))
    });

    Ok(coords)
}

/// Search for locations matching a query, returning structured suggestions.
pub async fn search_locations(query: &str) -> Result<Vec<LocationSuggestion>, ServerFnError> {
    if query.trim().len() < 2 {
        return Ok(vec![]);
    }

    let url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=json&addressdetails=1&limit=5",
        urlencoding::encode(query.trim())
    );

    let client = reqwest::Client::new();
    let results: Vec<NominatimResult> = client
        .get(&url)
        .header("User-Agent", "mybd/1.0")
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Geocoding request failed: {e}")))?
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Geocoding parse failed: {e}")))?;

    Ok(results
        .into_iter()
        .filter_map(|r| {
            let addr = r.address.as_ref()?;
            let country = addr.country.clone().unwrap_or_default();
            let city = addr
                .city
                .clone()
                .or_else(|| addr.town.clone())
                .or_else(|| addr.village.clone())
                .unwrap_or_default();
            Some(LocationSuggestion {
                display_name: r.display_name.unwrap_or_default(),
                country,
                city,
            })
        })
        .collect())
}
