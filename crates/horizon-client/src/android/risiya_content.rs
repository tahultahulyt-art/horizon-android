//! One-time local server list and optional native shader, only for Risiya builds.
use horizon_protocol::servers::{self, ServerEntry};
use std::path::Path;
const ADDRESS: &str = "delhi-791455.indernos.in:25565";
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
    let marker = home.join("risiya-light-offered-v1");
    if !marker.try_exists().map_err(|e| e.to_string())? {
        let pack = home.join("shaderpacks/risiya_light");
        if !pack.try_exists().map_err(|e| e.to_string())? {
            std::fs::create_dir_all(&pack).map_err(|e| e.to_string())?;
            for (name, bytes) in [
                (
                    "horizon-shaders.toml",
                    include_bytes!("assets/risiya/light/horizon-shaders.toml").as_slice(),
                ),
                (
                    "post.wgsl",
                    include_bytes!("assets/risiya/light/post.wgsl").as_slice(),
                ),
                (
                    "LICENSE.txt",
                    include_bytes!("assets/risiya/light/LICENSE.txt").as_slice(),
                ),
            ] {
                std::fs::write(pack.join(name), bytes).map_err(|e| e.to_string())?;
            }
        }
        // No pickedShaderPack option is written: installed, not forced ON.
        std::fs::write(marker, b"1").map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn server_once_and_shader_optional_without_replacing_user_files() {
        let p = std::env::temp_dir().join(format!("risiya-content-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        seed(&p).unwrap();
        let list = servers::load(&p.join("servers.dat"));
        assert!(list.iter().any(|s| s.ip == ADDRESS));
        assert!(
            p.join("shaderpacks/risiya_light/horizon-shaders.toml")
                .is_file()
        );
        assert!(!p.join("options.txt").exists());
        servers::save(&p.join("servers.dat"), &[]).unwrap();
        std::fs::write(p.join("shaderpacks/risiya_light/post.wgsl"), b"user edit").unwrap();
        seed(&p).unwrap();
        assert!(servers::load(&p.join("servers.dat")).is_empty());
        assert_eq!(
            std::fs::read(p.join("shaderpacks/risiya_light/post.wgsl")).unwrap(),
            b"user edit"
        );
        std::fs::remove_dir_all(p).unwrap();
    }
}
