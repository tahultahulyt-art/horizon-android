//! Kusobo's small management layer over the unmodified Horizon .hmod sandbox.
use std::{io::Write, path::Path};
const SAMPLE: &[u8] =
    include_bytes!("../../../../packaging/android/kusobo/kusobo_coordinates.hmod");
const TEMPLATE: &[u8] = include_bytes!("../../../../packaging/android/kusobo/kusobo-template.zip");

/// Called only by the launcher's worker. Never replaces user files or enables code.
pub fn install(game: &Path, template: bool) -> Result<String, String> {
    let (folder, name, bytes) = if template {
        ("config", "kusobo-template.zip", TEMPLATE)
    } else {
        ("horizon-mods", "kusobo_coordinates.hmod", SAMPLE)
    };
    std::fs::create_dir_all(game).map_err(|e| e.to_string())?;
    let dir = game.join(folder);
    if std::fs::symlink_metadata(&dir).is_ok_and(|m| m.file_type().is_symlink() || !m.is_dir()) {
        return Err("Refusing a redirected Kusobo destination".into());
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(name);
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(format!(
                "{name} already exists. Kept the existing file unchanged."
            ));
        }
        Err(e) => return Err(e.to_string()),
    };
    if let Err(e) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(&path);
        return Err(e.to_string());
    }
    Ok(if template {
        "Template exported to Config/kusobo-template.zip. Copy it with Files; a Rust development computer is needed to build it."
    } else {
        "Kusobo Coordinates installed OFF. Enable in Installed, then Play. /coords toggles HUD, /where prints position."
    }.into())
}
