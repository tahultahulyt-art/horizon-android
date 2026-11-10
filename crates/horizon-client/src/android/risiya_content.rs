//! One-time Risiya server list and scoped withdrawal of the optional crashing pack.
use horizon_protocol::servers::{self, ServerEntry};
use std::path::Path;
const ADDRESS: &str = "delhi-791455.indernos.in:25565";
const ADDRESS2: &str = "rahukkez.aternos.me:46509";
pub fn seed(home: &Path) -> Result<(), String> {
    std::fs::create_dir_all(home).map_err(|e| e.to_string())?;
    let marker = home.join("risiya-server-offered-v1");
    if !marker.try_exists().map_err(|e| e.to_string())? {
        let path = home.join("servers.dat");
        // Only prepend to a valid existing list; don't erase a damaged list.
        if let Ok(bytes) = std::fs::read(&path) {
            horizon_world::nbt::read_named(&bytes, horizon_world::nbt::Limits::default())
                .map_err(|_| "Existing server list is damaged; it was kept".to_string())?;
        }
        if !servers::load(&path)
            .iter()
            .any(|s| s.ip.eq_ignore_ascii_case(ADDRESS))
        {
            servers::prepend(&path, &ServerEntry::new("Rahul's server", ADDRESS))
                .map_err(|e| e.to_string())?;
        }
        std::fs::write(marker, b"1").map_err(|e| e.to_string())?;
    }
    seed2(home)
}
/// One-time offer of Rahul's Aternos server (update-safe: own marker, prepend only).
fn seed2(home: &Path) -> Result<(), String> {
    std::fs::create_dir_all(home).map_err(|e| e.to_string())?;
    let marker = home.join("risiya-server-offered-v2");
    if !marker.try_exists().map_err(|e| e.to_string())? {
        let path = home.join("servers.dat");
        // Only prepend to a valid existing list; don't erase a damaged list.
        if let Ok(bytes) = std::fs::read(&path) {
            horizon_world::nbt::read_named(&bytes, horizon_world::nbt::Limits::default())
                .map_err(|_| "Existing server list is damaged; it was kept".to_string())?;
        }
        if !servers::load(&path)
            .iter()
            .any(|s| s.ip.eq_ignore_ascii_case(ADDRESS2))
        {
            servers::prepend(&path, &ServerEntry::new("Rahul's Aternos", ADDRESS2))
                .map_err(|e| e.to_string())?;
        }
        std::fs::write(marker, b"1").map_err(|e| e.to_string())?;
    }
    Ok(())
}
/// Withdraw only the bundled Light pack, including saved ON selections in existing profiles.
/// Moving its folder outside shaderpacks preserves edits, without leaving it selectable.
pub fn remove_light(home: &Path) -> Result<(), String> {
    let marker = home.join("risiya-light-removed-v2");
    if marker.try_exists().map_err(|e| e.to_string())? {
        return Ok(());
    }
    let mut homes = vec![home.to_path_buf()];
    let profiles = home.join("profiles");
    if let Ok(entries) = std::fs::read_dir(profiles) {
        for entry in entries {
            let e = entry.map_err(|e| e.to_string())?;
            let name = e.file_name();
            let name = name.to_string_lossy();
            if name
                .strip_prefix("profile-")
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
                && e.file_type().map_err(|e| e.to_string())?.is_dir()
            {
                homes.push(e.path());
            }
        }
    }
    for game in homes {
        let graphics = game.join("horizon-graphics.txt");
        match std::fs::symlink_metadata(&graphics) {
            Ok(meta) if !meta.file_type().is_file() => {
                return Err("Graphics config is not a regular file; it was kept".into());
            }
            Ok(_) => {
                let original = std::fs::read_to_string(&graphics).map_err(|e| e.to_string())?;
                let mut updated = String::new();
                for line in original.split_inclusive('\n') {
                    let text = line.trim_end_matches(['\r', '\n']);
                    if text == "pickedShaderPack:risiya_light" {
                        updated.push_str("pickedShaderPack:");
                        updated.push_str(&line[text.len()..]);
                    } else {
                        updated.push_str(line);
                    }
                }
                if updated != original {
                    let stage = graphics.with_extension("risiya-removal.partial");
                    std::fs::write(&stage, updated).map_err(|e| e.to_string())?;
                    std::fs::rename(stage, graphics).map_err(|e| e.to_string())?;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        let pack = game.join("shaderpacks/risiya_light");
        match std::fs::symlink_metadata(&pack) {
            Ok(_) => {
                let disabled = game.join("disabled-shaders");
                std::fs::create_dir_all(&disabled).map_err(|e| e.to_string())?;
                let dest = disabled.join("risiya_light-removed-v2");
                if dest.try_exists().map_err(|e| e.to_string())? {
                    return Err(
                        "Withdrawn shader backup already exists; no files overwritten".into(),
                    );
                }
                std::fs::rename(pack, dest).map_err(|e| e.to_string())?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    std::fs::create_dir_all(home).map_err(|e| e.to_string())?;
    std::fs::write(marker, b"1").map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn dir(t: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("risiya-removal-{t}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
    #[test]
    fn new_install_has_server_no_shader() {
        let p = dir("new");
        seed(&p).unwrap();
        assert!(
            servers::load(&p.join("servers.dat"))
                .iter()
                .any(|s| s.ip == ADDRESS)
        );
        assert!(
            servers::load(&p.join("servers.dat"))
                .iter()
                .any(|s| s.ip == ADDRESS2)
        );
        assert!(!p.join("shaderpacks/risiya_light").exists());
        servers::save(&p.join("servers.dat"), &[]).unwrap();
        seed(&p).unwrap();
        assert!(servers::load(&p.join("servers.dat")).is_empty());
        std::fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn existing_all_profiles_clear_only_light_preserve_everything_else() {
        let p = dir("old");
        for sub in ["", "profiles/profile-2"] {
            let h = p.join(sub);
            std::fs::create_dir_all(h.join("shaderpacks/risiya_light")).unwrap();
            std::fs::write(h.join("shaderpacks/risiya_light/user-edit"), b"kept").unwrap();
            std::fs::write(
                h.join("horizon-graphics.txt"),
                b"unknown:keep\r\npickedShaderPack:risiya_light\r\nfoo:bar",
            )
            .unwrap();
        }
        std::fs::create_dir_all(p.join("profiles/profile-3")).unwrap();
        std::fs::write(
            p.join("profiles/profile-3/horizon-graphics.txt"),
            b"pickedShaderPack:another",
        )
        .unwrap();
        remove_light(&p).unwrap();
        for sub in ["", "profiles/profile-2"] {
            let h = p.join(sub);
            assert_eq!(
                std::fs::read(h.join("horizon-graphics.txt")).unwrap(),
                b"unknown:keep\r\npickedShaderPack:\r\nfoo:bar"
            );
            assert!(!h.join("shaderpacks/risiya_light").exists());
            assert_eq!(
                std::fs::read(h.join("disabled-shaders/risiya_light-removed-v2/user-edit"))
                    .unwrap(),
                b"kept"
            );
        }
        assert_eq!(
            std::fs::read(p.join("profiles/profile-3/horizon-graphics.txt")).unwrap(),
            b"pickedShaderPack:another"
        );
        remove_light(&p).unwrap();
        std::fs::remove_dir_all(p).unwrap();
    }
}
