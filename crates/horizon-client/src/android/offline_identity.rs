//! One-time requested offline identity. No credentials or game folders are touched.
use serde_json::{Map, Value};
use std::path::Path;

const MARKER: &str = "risiya_offline_applied_v1";
fn read_object(path: &Path) -> Result<Map<String, Value>, String> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice::<Value>(&bytes)
            .map_err(|e| format!("Cannot read saved account settings: {e}"))?
            .as_object()
            .cloned()
            .ok_or_else(|| "Saved account settings are not an object".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Map::new()),
        Err(e) => Err(e.to_string()),
    }
}
fn stage(path: &Path, value: &Map<String, Value>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
/// Both files are staged before changing either. Marker is committed last;
/// an interrupted migration retries safely at the next launcher startup.
pub fn apply(home: &Path, accounts_path: &Path) -> Result<bool, String> {
    let config_path = home.join("android-launcher.json");
    let mut config = read_object(&config_path)?;
    if config.get(MARKER).and_then(Value::as_bool) == Some(true) {
        return Ok(false);
    }
    let mut accounts = read_object(accounts_path)?;
    if accounts.get("accounts").is_some_and(|v| !v.is_array()) {
        return Err("Saved accounts list is invalid; no account was changed".into());
    }
    accounts
        .entry("accounts")
        .or_insert_with(|| Value::Array(vec![]));
    let mut offline = match accounts.get("offline") {
        None => vec![],
        Some(Value::Array(names)) => names.clone(),
        Some(_) => return Err("Saved offline names are invalid; no account was changed".into()),
    };
    if !offline.iter().any(|v| v.as_str() == Some("risiya")) {
        offline.push(Value::String("risiya".into()));
    }
    accounts.insert("offline".into(), Value::Array(offline));
    accounts.insert("selected".into(), Value::Null);
    accounts.insert("offline_selected".into(), Value::String("risiya".into()));
    config.insert("name".into(), Value::String("risiya".into()));
    config.insert(MARKER.into(), Value::Bool(true));
    let account_part = accounts_path.with_extension("risiya-migration.partial");
    let config_part = home.join("android-launcher.risiya-migration.partial");
    stage(&account_part, &accounts)?;
    stage(&config_part, &config)?;
    std::fs::rename(account_part, accounts_path).map_err(|e| e.to_string())?;
    std::fs::rename(config_part, config_path).map_err(|e| e.to_string())?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    struct Temp(std::path::PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "risiya-migration-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn accounts(&self) -> std::path::PathBuf {
            self.0.join("accounts.json")
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn first_install_selects_offline() {
        let t = Temp::new();
        assert_eq!(apply(&t.0, &t.accounts()), Ok(true));
        let config = read_object(&t.0.join("android-launcher.json")).unwrap();
        assert_eq!(config["name"], "risiya");
        let a = read_object(&t.accounts()).unwrap();
        assert_eq!(a["selected"], Value::Null);
        assert_eq!(a["offline_selected"], "risiya");
        assert_eq!(a["offline"], json!(["risiya"]));
    }
    #[test]
    fn update_preserves_accounts_looks_unknown_settings_and_game_files() {
        let t = Temp::new();
        let original = json!({"accounts":[{"name":"Online","uuid":"saved-id","future":7}],"selected":"saved-id","offline":["Old"],"looks":{"old":{"skin":"keep"}},"future_field":23});
        std::fs::write(t.accounts(), original.to_string()).unwrap();
        let config = json!({"name":"Old","profile":4,"renderer":"open-gl","richer_future":{"x":1}});
        std::fs::write(t.0.join("android-launcher.json"), config.to_string()).unwrap();
        std::fs::create_dir_all(t.0.join("profiles/profile-4/saves/world")).unwrap();
        std::fs::write(
            t.0.join("profiles/profile-4/saves/world/level.dat"),
            b"untouched",
        )
        .unwrap();
        apply(&t.0, &t.accounts()).unwrap();
        let a = read_object(&t.accounts()).unwrap();
        for k in ["accounts", "looks", "future_field"] {
            assert_eq!(a[k], original[k]);
        }
        assert_eq!(a["offline"], json!(["Old", "risiya"]));
        let c = read_object(&t.0.join("android-launcher.json")).unwrap();
        for k in ["profile", "renderer", "richer_future"] {
            assert_eq!(c[k], config[k]);
        }
        assert_eq!(
            std::fs::read(t.0.join("profiles/profile-4/saves/world/level.dat")).unwrap(),
            b"untouched"
        );
    }
    #[test]
    fn later_user_choice_is_never_reset() {
        let t = Temp::new();
        apply(&t.0, &t.accounts()).unwrap();
        let p = t.0.join("android-launcher.json");
        let mut c = read_object(&p).unwrap();
        c.insert("name".into(), json!("Changed"));
        stage(&p, &c).unwrap();
        let mut a = read_object(&t.accounts()).unwrap();
        a.insert("selected".into(), json!("online-again"));
        stage(&t.accounts(), &a).unwrap();
        assert_eq!(apply(&t.0, &t.accounts()), Ok(false));
        assert_eq!(read_object(&p).unwrap()["name"], "Changed");
        assert_eq!(
            read_object(&t.accounts()).unwrap()["selected"],
            "online-again"
        );
    }
    #[test]
    fn malformed_saved_data_is_not_overwritten() {
        for bad in ["not json", "[]", "{\"accounts\":{}}", "{\"offline\":false}"] {
            let t = Temp::new();
            std::fs::write(t.accounts(), bad).unwrap();
            assert!(apply(&t.0, &t.accounts()).is_err());
            assert_eq!(std::fs::read_to_string(t.accounts()).unwrap(), bad);
            assert!(!t.0.join("android-launcher.json").exists());
        }
    }
    #[test]
    fn interrupted_before_marker_retries_without_duplicates() {
        let t = Temp::new();
        std::fs::write(
            t.accounts(),
            json!({"accounts":[],"selected":null,"offline":["risiya"],"offline_selected":"risiya"})
                .to_string(),
        )
        .unwrap();
        assert_eq!(apply(&t.0, &t.accounts()), Ok(true));
        assert_eq!(
            read_object(&t.accounts()).unwrap()["offline"],
            json!(["risiya"])
        );
    }
}
