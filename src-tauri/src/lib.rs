use tauri::command;
use chrono::{Utc, SecondsFormat};

#[command]
async fn fetch_outages() -> Result<serde_json::Value, String> {
    let now = Utc::now();
    let from_date = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    
    let url = "https://www.tauron-dystrybucja.pl/waapi/outages/address";
    
    let client = reqwest::Client::new();
    let query_params: Vec<(&str, &str)> = vec![
        ("cityGAID", "119431"),
        ("streetGAID", "897300"),
        ("houseNo", "8"),
        ("fromDate", &from_date),
        ("getLightingSupport", "true"),
        ("getServicedSwitchingoff", "true"),
        ("_", &now.timestamp_millis().to_string()),
    ];

    let res = client.get(url)
        .query(&query_params)
        .header("accept", "application/json, text/javascript, */*; q=0.01")
        .header("accept-language", "pl;q=0.8")
        .header("sec-ch-ua", "\"Not(A:Brand\";v=\"8\", \"Chromium\";v=\"144\", \"Brave\";v=\"144\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"Windows\"")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-origin")
        .header("sec-gpc", "1")
        .header("x-requested-with", "XMLHttpRequest")
        .header("cookie", "shell#lang=en; ASP.NET_SessionId=wuyz0t43k0scrn2hg0fntzrd; SERVERID=var02; tauron-load-balancer-cookie-w=!kFDkC4WWkqO/xcVy4hYKIFHezWcwD86Bzl+/yyVijjteif/LyRKpOfQq7Lx71FyJHZG0n3F/Fni7l1q0VEOqGWoRGIrW/yoXG0PqM8f+iygl; SC_ANALYTICS_GLOBAL_COOKIE=2bed67dd00d34312bcaa72f4eb145fc1|True; 2effbf1d689e5e222f6296ba4e6dada6=59b43dd83da3cbdea52930250fa39faa; TS018bf492=015f73abc6682f350f940cc5eae57d9b98f432548c1e0ea6b44c2de0a29439283143b6f406bd4bb088970bdd3779e1d45835e418f02318a72d0dd94c6cd94c93dcc073bb2139ee27c52394c96a32313bf0c53a4bd9234d4b728acb470c8c74526170c7ed9a00b55e5866bb5309a040a20ad7a5b49fb369fbf0174cd1f15a062be2bcfd91d4")
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
    .invoke_handler(tauri::generate_handler![fetch_outages])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
