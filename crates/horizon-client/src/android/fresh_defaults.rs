//! Requested launch defaults are created only for a genuinely new installation.
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
