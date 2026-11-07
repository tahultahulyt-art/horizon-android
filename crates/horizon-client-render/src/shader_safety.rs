//! Fail-closed shader activation journal. It contains a local pack filename, never account data.
use std::path::{Path, PathBuf};
fn journal(options: &Path) -> PathBuf {
    options.with_file_name("horizon-shader-active.json")
}
/// Arm before any native GPU allocation. A hard driver/process exit leaves this behind.
pub fn arm(options: &Path, pack: &str) -> Result<(), String> {
    if pack.is_empty() {
        return Err("No shader selected".into());
    }
    let path = journal(options);
    let stage = path.with_extension("partial");
    let bytes = serde_json::to_vec(&pack).map_err(|e| e.to_string())?;
    use std::io::Write;
    let mut f = std::fs::File::create(&stage).map_err(|e| e.to_string())?;
    f.write_all(&bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    std::fs::rename(stage, path).map_err(|e| e.to_string())
}
pub fn disarm(options: &Path) -> Result<(), String> {
    match std::fs::remove_file(journal(options)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
/// Only a matching saved choice is reset. A process kill while shaders ran is conservatively
/// treated like a crash. Corrupt journals also fail closed, but never change other settings.
pub fn recover(options: &Path, selected: &mut String) -> Result<bool, String> {
    let path = journal(options);
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    let saved = serde_json::from_slice::<String>(&bytes).ok();
    let reset = !selected.is_empty() && saved.as_ref().is_none_or(|p| p == selected);
    if reset {
        selected.clear();
    }
    // Caller must persist a reset BEFORE disarming, so a failed save cannot re-arm a crash loop.
    if !reset {
        disarm(options)?;
    }
    Ok(reset)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn temp(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("shader-journal-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&p).unwrap();
        p.join("options.txt")
    }
    #[test]
    fn crash_resets_only_failed_choice() {
        let p = temp("crash");
        arm(&p, "warm.hshader").unwrap();
        let mut s = "warm.hshader".into();
        assert!(recover(&p, &mut s).unwrap());
        assert!(s.is_empty());
        assert!(journal(&p).exists());
        disarm(&p).unwrap();
        assert!(!recover(&p, &mut s).unwrap());
        std::fs::remove_dir_all(p.parent().unwrap()).unwrap();
    }
    #[test]
    fn other_selection_survives_and_clean_session_does_not_reset() {
        let p = temp("other");
        arm(&p, "warm").unwrap();
        let mut s = "other".into();
        assert!(!recover(&p, &mut s).unwrap());
        assert_eq!(s, "other");
        arm(&p, "other").unwrap();
        disarm(&p).unwrap();
        assert!(!recover(&p, &mut s).unwrap());
        std::fs::remove_dir_all(p.parent().unwrap()).unwrap();
    }
    #[test]
    fn corrupt_journal_fails_closed() {
        let p = temp("corrupt");
        std::fs::write(journal(&p), b"{").unwrap();
        let mut s = "warm".into();
        assert!(recover(&p, &mut s).unwrap());
        assert!(s.is_empty());
        disarm(&p).unwrap();
        std::fs::remove_dir_all(p.parent().unwrap()).unwrap();
    }
}

/// Persist OFF without resetting any other graphics preference or unknown line.
pub fn persist_off(options: &Path) -> Result<(), String> {
    let path = options.with_file_name("horizon-graphics.txt");
    let original = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.to_string()),
    };
    let mut out = String::new();
    let mut found = false;
    for line in original.split_inclusive('\n') {
        let text = line.trim_end_matches(['\r', '\n']);
        if text.starts_with("pickedShaderPack:") {
            out.push_str("pickedShaderPack:");
            out.push_str(&line[text.len()..]);
            found = true;
        } else {
            out.push_str(line);
        }
    }
    if !found {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("pickedShaderPack:\n");
    }
    let stage = path.with_extension("shader-off.partial");
    use std::io::Write;
    let mut f = std::fs::File::create(&stage).map_err(|e| e.to_string())?;
    f.write_all(out.as_bytes())
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    std::fs::rename(stage, path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod persist_tests {
    use super::*;
    #[test]
    fn off_changes_only_selection_including_crlf() {
        let p = std::env::temp_dir().join(format!("shader-off-{}", std::process::id()));
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(
            p.join("horizon-graphics.txt"),
            b"unknown:value\r\npickedShaderPack:warm\r\nother:keep",
        )
        .unwrap();
        persist_off(&p.join("options.txt")).unwrap();
        assert_eq!(
            std::fs::read(p.join("horizon-graphics.txt")).unwrap(),
            b"unknown:value\r\npickedShaderPack:\r\nother:keep"
        );
        std::fs::remove_dir_all(p).unwrap();
    }
}
