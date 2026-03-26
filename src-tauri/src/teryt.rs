use rusqlite::Connection;
use serde::Serialize;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize)]
pub struct TerytCity {
    pub voivodeship: String,
    pub district: String,
    pub commune: String,
    pub city: String,
    pub city_id: u64,
}

#[derive(Debug, Serialize)]
pub struct TerytStreet {
    pub full_street_name: String,
    pub city_id: u64,
    pub street_id: u64,
    pub street_name_1: String,
    pub street_name_2: Option<String>,
}

fn db_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    // Use app_data_dir for the working copy of the database
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {}", e))?;
    std::fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
    let db = app_data.join("teryt");

    if db.exists() {
        return Ok(db);
    }

    // Copy from resource dir to app data dir on first use
    if let Ok(resource_path) = app.path().resource_dir() {
        let candidates = [
            resource_path.join("assets").join("teryt"),
            resource_path.join("assets").join("assets").join("teryt"),
            resource_path.join("teryt"),
        ];
        for src in &candidates {
            if src.exists() {
                if std::fs::copy(src, &db).is_ok() {
                    return Ok(db);
                }
            }
        }
    }

    // Fallback: try the raw asset path (Android may return asset:// URLs)
    let asset_candidates = [
        std::path::PathBuf::from("/data/data/xyz.eremef.awaria/files/assets/assets/teryt"),
        std::path::PathBuf::from("/data/data/xyz.eremef.awaria/files/assets/teryt"),
        std::path::PathBuf::from("/data/data/xyz.eremef.awaria/files/teryt"),
    ];
    for src in &asset_candidates {
        if src.exists() {
            if std::fs::copy(src, &db).is_ok() {
                return Ok(db);
            }
        }
    }

    // Fallback: dev mode relative to Cargo.toml
    let dev_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("teryt");
    if dev_path.exists() {
        if std::fs::copy(&dev_path, &db).is_ok() {
            return Ok(db);
        }
    }

    Err(format!(
        "Teryt database not found in any location. Target: {:?}",
        db
    ))
}

pub fn lookup_cities(app: &AppHandle, city_name: &str) -> Result<Vec<TerytCity>, String> {
    let path = db_path(app)?;
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Failed to open teryt DB: {}", e))?;

    let sql = "SELECT voivodeship.nazwa, district.nazwa, commune.nazwa, city.nazwa, city.sym \
               FROM simc city \
               LEFT JOIN terc voivodeship ON city.woj = voivodeship.woj \
                   AND voivodeship.pow IS NULL AND voivodeship.gmi IS NULL \
               LEFT JOIN terc district ON city.woj = district.woj \
                   AND city.pow = district.pow AND district.gmi IS NULL \
               LEFT JOIN terc commune ON city.woj = commune.woj \
                   AND city.pow = commune.pow AND city.gmi = commune.gmi \
                   AND city.rodz_gmi = commune.rodz \
               WHERE city.sym = city.sympod \
                   AND city.nazwa = ?1 COLLATE NOCASE \
               ORDER BY city.nazwa \
               LIMIT 20";

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Failed to prepare city query: {}", e))?;
    let rows = stmt
        .query_map([city_name.to_string()], |row| {
            Ok(TerytCity {
                voivodeship: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                district: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                commune: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                city: row.get(3)?,
                city_id: row.get::<_, i64>(4)? as u64,
            })
        })
        .map_err(|e| format!("City query failed: {}", e))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("City row error: {}", e))?);
    }
    Ok(results)
}

pub fn lookup_streets(
    app: &AppHandle,
    city_id: u64,
    street_name: &str,
) -> Result<Vec<TerytStreet>, String> {
    let path = db_path(app)?;
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Failed to open teryt DB: {}", e))?;

    let sql = "SELECT street.cecha || IFNULL(' ' || street.nazwa_2, '') || ' ' || street.nazwa_1 AS full_name,
                       street.sym, street.sym_ul, street.nazwa_1, street.nazwa_2 \
               FROM simc city \
               LEFT JOIN simc city_part ON city.sym = city_part.sympod \
               LEFT JOIN ulic street ON city.sym = street.sym \
                   OR city_part.sym = street.sym \
               WHERE city.sym = city.sympod \
                   AND street.sym_ul IS NOT NULL \
                   AND city.sym = ?1 \
                   AND full_name LIKE ?2 COLLATE NOCASE \
               ORDER BY full_name \
               LIMIT 30";

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Failed to prepare street query: {}", e))?;
    let pattern = format!("%{}%", street_name);
    let city_id_i64 = city_id as i64;
    let rows = stmt
        .query_map([&city_id_i64 as &dyn rusqlite::ToSql, &pattern], |row| {
            Ok(TerytStreet {
                full_street_name: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                city_id: row.get::<_, i64>(1)? as u64,
                street_id: row.get::<_, i64>(2)? as u64,
                street_name_1: row.get(3)?,
                street_name_2: row.get(4)?,
            })
        })
        .map_err(|e| format!("Street query failed: {}", e))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Street row error: {}", e))?);
    }
    Ok(results)
}
