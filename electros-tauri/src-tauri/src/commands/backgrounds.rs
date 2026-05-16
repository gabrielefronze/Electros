use crate::paths::{self, backgrounds_dir, ensure_elemento_dirs};
use base64::Engine;
use image::ImageFormat;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

const SUPPORTED: &[&str] = &[".jpg", ".jpeg", ".png", ".gif", ".webp", ".bmp"];

fn is_image(name: &str) -> bool {
    let lower = name.to_lowercase();
    SUPPORTED.iter().any(|ext| lower.ends_with(ext))
}

pub async fn list_backgrounds() -> Value {
    ensure_elemento_dirs();
    let dir = backgrounds_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return json!([]);
    };

    let mut files: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|f| is_image(f))
        .collect();

    let mut file_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for file in &files {
        let path = dir.join(file);
        let ext = Path::new(file)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let base = Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file)
            .to_string();
        if ext != "webp" {
            let webp = dir.join(format!("{base}.webp"));
            if !webp.exists() {
                file_map.insert(base, file.clone());
            }
        }
    }
    for file in &files {
        if file.to_lowercase().ends_with(".webp") {
            let base = Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(file)
                .to_string();
            file_map.insert(base, file.clone());
        }
    }

    let list: Vec<Value> = file_map
        .values()
        .filter_map(|file| {
            let file_path = dir.join(file);
            let file_url = paths::elemento_bg_url_for_path(&file_path).ok()?;
            Some(json!({
                "name": file,
                "path": file_path.to_string_lossy(),
                "fileUrl": file_url,
            }))
        })
        .collect();

    json!(list)
}

pub async fn get_background_data(image_path: String, for_thumbnail: bool) -> Value {
    let path = Path::new(&image_path);
    if !path.exists() {
        return Value::Null;
    }
    if for_thumbnail {
        if let Ok(data) = fs::read(path) {
            let b64 = base64::engine::general_purpose::STANDARD.encode(data);
            let mime = match path.extension().and_then(|e| e.to_str()) {
                Some("png") => "image/png",
                Some("gif") => "image/gif",
                Some("webp") => "image/webp",
                _ => "image/jpeg",
            };
            return json!({
                "dataUrl": format!("data:{mime};base64,{b64}")
            });
        }
        return Value::Null;
    }
    paths::elemento_bg_url_for_path(path)
        .map(|u| json!({ "fileUrl": u }))
        .unwrap_or(Value::Null)
}

pub async fn import_background() -> Value {
    // Dialog handled in invoke via plugin from lib - fallback error
    json!({ "success": false, "error": "Use native import dialog via Tauri" })
}

pub async fn import_background_from_path(source: String) -> Value {
    ensure_elemento_dirs();
    let file_name = Path::new(&source)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "wallpaper.jpg".into());
    let dest = backgrounds_dir().join(&file_name);
    if let Err(e) = fs::copy(&source, &dest) {
        return json!({ "success": false, "error": e.to_string() });
    }
    let final_path = convert_to_webp(&dest).await.unwrap_or(dest);
    let final_name = final_path.file_name().unwrap().to_string_lossy();
    let file_url = paths::elemento_bg_url_for_path(&final_path).unwrap_or_default();
    json!({
        "success": true,
        "file": {
            "name": final_name,
            "path": final_path.to_string_lossy(),
            "fileUrl": file_url,
        }
    })
}

async fn convert_to_webp(source: &Path) -> Option<std::path::PathBuf> {
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ext == "webp" {
        return Some(source.to_path_buf());
    }
    let stem = source.file_stem()?.to_str()?;
    let webp_path = source.with_file_name(format!("{stem}.webp"));
    let img = image::open(source).ok()?;
    img.save_with_format(&webp_path, ImageFormat::WebP).ok()?;
    let _ = fs::remove_file(source);
    Some(webp_path)
}

pub async fn save_background_from_url(
    image_url: String,
    filename: Option<String>,
    subfolder: Option<String>,
) -> Value {
    ensure_elemento_dirs();
    let client = reqwest::Client::builder()
        .user_agent("Electros/3.1 (https://elemento.cloud/electros; hello@elemento.cloud)")
        .build()
        .unwrap();
    let mut url = image_url.clone();
    if url.contains("upload.wikimedia.org") && url.contains("/thumb/") {
        if let Ok(parsed) = url::Url::parse(&url) {
            let parts: Vec<&str> = parsed.path().split('/').filter(|p| !p.is_empty()).collect();
            if let Some(ti) = parts.iter().position(|p| *p == "thumb") {
                if parts.len() > ti + 3 {
                    let mut new_parts: Vec<&str> = parts[..ti].to_vec();
                    new_parts.extend_from_slice(&parts[ti + 1..parts.len() - 1]);
                    let new_path = format!("/{}", new_parts.join("/"));
                    if let Ok(mut u) = url::Url::parse(&url) {
                        u.set_path(&new_path);
                        url = u.to_string();
                    }
                }
            }
        }
    }

    let mut dest_dir = backgrounds_dir();
    if let Some(ref sub) = subfolder {
        dest_dir = dest_dir.join(sub);
        let _ = fs::create_dir_all(&dest_dir);
    }

    let fname = filename.unwrap_or_else(|| {
        url.split('/')
            .last()
            .unwrap_or("image.jpg")
            .split('?')
            .next()
            .unwrap_or("image.jpg")
            .replace(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-', "_")
    });

    let mut dest_path = dest_dir.join(&fname);
    let mut counter = 1;
    while dest_path.exists() && subfolder.as_deref() != Some("cache") {
        let stem = Path::new(&fname)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("img");
        let ext = Path::new(&fname).extension().and_then(|e| e.to_str()).unwrap_or("jpg");
        dest_path = dest_dir.join(format!("{stem}_{counter}.{ext}"));
        counter += 1;
    }

    match client.get(&url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return json!({ "success": false, "error": format!("HTTP {}", resp.status()) });
            }
            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => return json!({ "success": false, "error": e.to_string() }),
            };
            if let Err(e) = fs::write(&dest_path, &bytes) {
                return json!({ "success": false, "error": e.to_string() });
            }
            let final_path = convert_to_webp(&dest_path).await.unwrap_or(dest_path.clone());
            let file_url = paths::elemento_bg_url_for_path(&final_path).unwrap_or_default();
            json!({
                "success": true,
                "file": {
                    "name": final_path.file_name().unwrap().to_string_lossy(),
                    "path": final_path.to_string_lossy(),
                    "fileUrl": file_url,
                }
            })
        }
        Err(e) => json!({ "success": false, "error": e.to_string() }),
    }
}

pub async fn delete_background(image_path: String) -> Value {
    match fs::remove_file(&image_path) {
        Ok(()) => json!({ "success": true }),
        Err(e) => json!({ "success": false, "error": e.to_string() }),
    }
}

pub async fn convert_existing_backgrounds() -> Value {
    ensure_elemento_dirs();
    let dir = backgrounds_dir();
    let mut converted = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let Ok(entries) = fs::read_dir(&dir) else {
        return json!({ "success": false, "error": "Cannot read backgrounds dir" });
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext == "webp" {
            skipped += 1;
            continue;
        }
        if convert_to_webp(&path).await.is_some() {
            converted += 1;
        } else {
            failed += 1;
        }
    }
    json!({ "success": true, "converted": converted, "failed": failed, "skipped": skipped })
}

pub fn serve_elemento_bg(uri: &str) -> Result<tauri::http::Response<Vec<u8>>, String> {
    let parsed = url::Url::parse(uri).map_err(|e| e.to_string())?;
    if parsed.host_str() != Some("bg") {
        return Err("Not found".into());
    }
    let encoded = parsed.path().trim_start_matches('/');
    let rel = percent_encoding::percent_decode_str(encoded)
        .decode_utf8()
        .map_err(|e| e.to_string())?;
    if rel.contains("..") {
        return Err("Forbidden".into());
    }
    let root = backgrounds_dir();
    let file_path = root.join(rel.as_ref());
    let canonical = file_path.canonicalize().map_err(|_| "Not found")?;
    let root_canon = root.canonicalize().map_err(|_| "Not found")?;
    if !canonical.starts_with(&root_canon) {
        return Err("Forbidden".into());
    }
    let data = fs::read(&canonical).map_err(|_| "Not found")?;
    let mime = match canonical.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        _ => "image/jpeg",
    };
    Ok(tauri::http::Response::builder()
        .header("Content-Type", mime)
        .body(data)
        .unwrap())
}
