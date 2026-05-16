use std::path::{Path, PathBuf};

pub fn elemento_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".elemento")
}

pub fn config_path() -> PathBuf {
    elemento_dir().join("settings")
}

pub fn hosts_path() -> PathBuf {
    elemento_dir().join("hosts")
}

pub fn backgrounds_dir() -> PathBuf {
    elemento_dir().join("backgrounds")
}

pub fn ensure_elemento_dirs() {
    let _ = std::fs::create_dir_all(elemento_dir());
    let _ = std::fs::create_dir_all(backgrounds_dir());
}

pub fn elemento_bg_url_for_path(absolute: &Path) -> Result<String, String> {
    let root = backgrounds_dir().canonicalize().map_err(|e| e.to_string())?;
    let resolved = absolute.canonicalize().map_err(|e| e.to_string())?;
    if !resolved.starts_with(&root) {
        return Err("Invalid background path".into());
    }
    let rel = resolved
        .strip_prefix(&root)
        .map_err(|_| "Invalid background path")?;
    let posix = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    Ok(format!(
        "elemento-bg://bg/{}",
        percent_encoding::utf8_percent_encode(&posix, percent_encoding::NON_ALPHANUMERIC)
    ))
}
