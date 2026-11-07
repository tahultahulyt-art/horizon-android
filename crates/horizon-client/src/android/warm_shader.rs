//! Own Horizon variant only. Optional original shader, never selects it automatically.
use std::path::Path;
pub fn seed(home: &Path) -> Result<(), String> {
    let marker = home.join("warm-lite-offered-v1");
    if marker.try_exists().map_err(|e| e.to_string())? {
        return Ok(());
    }
    let pack = home.join("shaderpacks/horizon_warm_lite");
    if !pack.try_exists().map_err(|e| e.to_string())? {
        std::fs::create_dir_all(&pack).map_err(|e| e.to_string())?;
        for (name, bytes) in [
            (
                "horizon-shaders.toml",
                include_bytes!(
                    "../../../horizon-shader-api/packs/horizon_warm_lite/horizon-shaders.toml"
                )
                .as_slice(),
            ),
            (
                "sky.wgsl",
                include_bytes!("../../../horizon-shader-api/packs/horizon_warm_lite/sky.wgsl")
                    .as_slice(),
            ),
            (
                "grade.wgsl",
                include_bytes!("../../../horizon-shader-api/packs/horizon_warm_lite/grade.wgsl")
                    .as_slice(),
            ),
            (
                "LICENSE.txt",
                include_bytes!("../../../horizon-shader-api/packs/horizon_warm_lite/LICENSE.txt")
                    .as_slice(),
            ),
            (
                "README.md",
                include_bytes!("../../../horizon-shader-api/packs/horizon_warm_lite/README.md")
                    .as_slice(),
            ),
        ] {
            std::fs::write(pack.join(name), bytes).map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(marker, b"1").map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installs_once_never_enables_or_replaces_user_edit() {
        let p = std::env::temp_dir().join(format!("warm-lite-install-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        seed(&p).unwrap();
        assert!(p.join("shaderpacks/horizon_warm_lite/grade.wgsl").is_file());
        assert!(!p.join("horizon-graphics.txt").exists());
        std::fs::write(p.join("shaderpacks/horizon_warm_lite/grade.wgsl"), b"edit").unwrap();
        seed(&p).unwrap();
        assert_eq!(
            std::fs::read(p.join("shaderpacks/horizon_warm_lite/grade.wgsl")).unwrap(),
            b"edit"
        );
        std::fs::remove_dir_all(p).unwrap();
    }
}
