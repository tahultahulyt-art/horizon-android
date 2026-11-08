//! Requested launch defaults are created only for a genuinely new installation.
//! migrate() merges the same launcher defaults once into existing installs.
use std::path::Path;

pub fn seed(home: &Path) -> Result<bool, String> {
    let path = home.join("android-launcher.json");
    // Existing or unreadable data is never treated as a new install.
    match std::fs::symlink_metadata(&path) {
        Ok(_) => return Ok(false),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    std::fs::create_dir_all(home).map_err(|e| e.to_string())?;
    let touch = home.join("horizon-touch.json");
    // Keep existing layouts even in installations with a missing launcher file.
    match std::fs::symlink_metadata(&touch) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&touch)
                .map_err(|e| e.to_string())?;
            file.write_all(b"{\"sensitivity\":8.75}")
                .map_err(|e| e.to_string())?;
        }
        Err(e) => return Err(e.to_string()),
    }
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(br#"{"renderer":"open-gl","worker_limit":7,"cache_budget_mib":64,"display_hz":120,"performance_hints":true}"#).map_err(|e|e.to_string())?;
    Ok(true)
}

/// One-time merge of the launcher defaults into an existing installation.
/// Keys the defaults do not set keep the player's values; a malformed or
/// missing launcher file is replaced with just the defaults. The touch
/// layout (horizon-touch.json) is personal and is never rewritten here.
pub fn migrate(home: &Path) -> Result<bool, String> {
    let marker = home.join("risiya-style-launcher-defaults-v1");
    if marker.try_exists().map_err(|e| e.to_string())? {
        return Ok(false);
    }
    std::fs::create_dir_all(home).map_err(|e| e.to_string())?;
    let path = home.join("android-launcher.json");
    let mut json: serde_json::Map<String, serde_json::Value> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    for (key, value) in [
        ("renderer", serde_json::json!("open-gl")),
        ("worker_limit", serde_json::json!(7)),
        ("cache_budget_mib", serde_json::json!(64)),
        ("display_hz", serde_json::json!(120)),
        ("performance_hints", serde_json::json!(true)),
    ] {
        json.insert(key.to_string(), value);
    }
    let text = serde_json::to_string(&serde_json::Value::Object(json)).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    std::fs::write(marker, b"1").map_err(|e| e.to_string())?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Temp(std::path::PathBuf);
    impl Temp {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!(
                    "fresh-defaults-{}-{}",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                )))
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn fresh_values_are_seeded_once() {
        let t = Temp::new();
        assert_eq!(seed(&t.0), Ok(true));
        let c: serde_json::Value =
            serde_json::from_slice(&std::fs::read(t.0.join("android-launcher.json")).unwrap())
                .unwrap();
        assert_eq!(c["renderer"], "open-gl");
        assert_eq!(c["worker_limit"], 7);
        assert_eq!(c["cache_budget_mib"], 64);
        assert_eq!(c["display_hz"], 120);
        assert_eq!(c["performance_hints"], true);
        let p = t.0.join("horizon-touch.json");
        let touch: serde_json::Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        assert_eq!(touch["sensitivity"], 8.75);
        std::fs::write(&p, b"user changed").unwrap();
        assert_eq!(seed(&t.0), Ok(false));
        assert_eq!(std::fs::read(&p).unwrap(), b"user changed");
    }
    #[test]
    fn migrate_merges_once_and_keeps_other_keys() {
        let t = Temp::new();
        std::fs::create_dir_all(&t.0).unwrap();
        let config = t.0.join("android-launcher.json");
        std::fs::write(&config, br#"{"renderer":"auto","custom":"kept"}"#).unwrap();
        assert_eq!(migrate(&t.0), Ok(true));
        let c: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&config).unwrap()).unwrap();
        assert_eq!(c["renderer"], "open-gl");
        assert_eq!(c["display_hz"], 120);
        assert_eq!(c["custom"], "kept");
        // The marker makes a second run a no-op.
        std::fs::write(&config, br#"{"renderer":"auto"}"#).unwrap();
        assert_eq!(migrate(&t.0), Ok(false));
        assert_eq!(
            std::fs::read(&config).unwrap(),
            br#"{"renderer":"auto"}"#.to_vec()
        );
        // Touch layout is untouched.
        assert!(!t.0.join("horizon-touch.json").exists());
    }
    #[test]
    fn existing_even_malformed_config_and_touch_are_never_replaced() {
        let t = Temp::new();
        std::fs::create_dir_all(&t.0).unwrap();
        let config = t.0.join("android-launcher.json");
        std::fs::write(&config, b"not json").unwrap();
        assert_eq!(seed(&t.0), Ok(false));
        assert_eq!(std::fs::read(&config).unwrap(), b"not json");
        assert!(!t.0.join("horizon-touch.json").exists());
        std::fs::remove_file(config).unwrap();
        let p = t.0.join("horizon-touch.json");
        std::fs::write(&p, b"existing layout").unwrap();
        assert_eq!(seed(&t.0), Ok(true));
        assert_eq!(std::fs::read(&p).unwrap(), b"existing layout");
    }
}
