use tauri::command;
use tauri::AppHandle;
use tauri::Manager;
use chrono::{Utc, SecondsFormat};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const BASE_URL: &str = "https://www.tauron-dystrybucja.pl/waapi";

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct GeoItem {
    pub GAID: u64,
    pub Name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct Settings {
    pub cityName: String,
    pub streetName: String,
    pub houseNo: String,
    pub cityGAID: u64,
    pub streetGAID: u64,
    #[serde(default)]
    pub theme: Option<String>,
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    Ok(data_dir.join("settings.json"))
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .build()
        .map_err(|e| e.to_string())
}

#[command]
async fn lookup_city(city_name: String) -> Result<Vec<GeoItem>, String> {
    let client = build_client()?;
    let cache_bust = Utc::now().timestamp_millis().to_string();

    let res = client
        .get(&format!("{}/enum/geo/cities", BASE_URL))
        .query(&[("partName", &city_name), ("_", &cache_bust)])
        .header("accept", "application/json")
        .header("x-requested-with", "XMLHttpRequest")
        .header("Referer", "https://www.tauron-dystrybucja.pl/wylaczenia")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("HTTP error: {}", res.status()));
    }

    res.json().await.map_err(|e| e.to_string())
}

#[command]
async fn lookup_street(street_name: String, city_gaid: u64) -> Result<Vec<GeoItem>, String> {
    let client = build_client()?;
    let cache_bust = Utc::now().timestamp_millis().to_string();
    let owner = city_gaid.to_string();

    let res = client
        .get(&format!("{}/enum/geo/streets", BASE_URL))
        .query(&[
            ("partName", street_name.as_str()),
            ("ownerGAID", &owner),
            ("_", &cache_bust),
        ])
        .header("accept", "application/json")
        .header("x-requested-with", "XMLHttpRequest")
        .header("Referer", "https://www.tauron-dystrybucja.pl/wylaczenia")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("HTTP error: {}", res.status()));
    }

    res.json().await.map_err(|e| e.to_string())
}

#[command]
async fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let path = settings_path(&app)?;
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
async fn load_settings(app: AppHandle) -> Result<Option<Settings>, String> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let settings: Settings = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(Some(settings))
}

#[command]
async fn fetch_outages(app: AppHandle) -> Result<serde_json::Value, String> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Err("No settings configured. Please set up your location first.".into());
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let settings: Settings = serde_json::from_str(&data).map_err(|e| e.to_string())?;

    let now = Utc::now();
    let from_date = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    let cache_bust = now.timestamp_millis().to_string();
    let city_str = settings.cityGAID.to_string();
    let street_str = settings.streetGAID.to_string();

    let client = build_client()?;
    let query_params: Vec<(&str, &str)> = vec![
        ("cityGAID", &city_str),
        ("streetGAID", &street_str),
        ("houseNo", &settings.houseNo),
        ("fromDate", &from_date),
        ("getLightingSupport", "false"),
        ("getServicedSwitchingoff", "true"),
        ("_", &cache_bust),
    ];

    let res = client.get(&format!("{}/outages/address", BASE_URL))
        .query(&query_params)
        .header("accept", "application/json")
        .header("x-requested-with", "XMLHttpRequest")
        .header("Referer", "https://www.tauron-dystrybucja.pl/wylaczenia")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("HTTP error! status: {}", res.status()));
    }

    let data = res.json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(data)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        fetch_outages,
        lookup_city,
        lookup_street,
        save_settings,
        load_settings
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
