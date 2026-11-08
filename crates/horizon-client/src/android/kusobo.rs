//! Kusobo's small management layer over the unmodified Horizon .hmod sandbox.
use std::{io::Write, path::Path};
const SAMPLE: &[u8] =
    include_bytes!("../../../../packaging/android/kusobo/kusobo_coordinates.hmod");
const TEMPLATE: &[u8] = include_bytes!("../../../../packaging/android/kusobo/kusobo-template.zip");
const SCRIPT_SAMPLE: &[u8] =
    include_bytes!("../../../../packaging/android/kusobo/script/kusobo_cps.ksmod");
const SCRIPT_TEMPLATE: &[u8] =
    include_bytes!("../../../../packaging/android/kusobo/script/kusobo-script-template.zip");

/// A resource the Kusobo Loader page installs or exports.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    /// The WebAssembly coordinates sample mod (installed OFF).
    Sample,
    /// The Rust/WebAssembly source template (needs a computer).
    Template,
    /// The Lua CPS counter sample mod (installed OFF).
    ScriptSample,
    /// The Lua script template + phone guide (edited on the phone).
    ScriptTemplate,
}

/// Called only by the launcher's worker. Never replaces user files or enables code.
pub fn install(game: &Path, resource: Resource) -> Result<String, String> {
    let (folder, name, bytes) = match resource {
        Resource::Sample => ("horizon-mods", "kusobo_coordinates.hmod", SAMPLE),
        Resource::Template => ("", "kusobo-template.zip", TEMPLATE),
        Resource::ScriptSample => ("horizon-mods", "kusobo_cps.ksmod", SCRIPT_SAMPLE),
        Resource::ScriptTemplate => ("", "kusobo-script-template.zip", SCRIPT_TEMPLATE),
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
    Ok(match resource {
        Resource::Sample => "Kusobo Coordinates installed OFF. Enable in Installed, then Play. /coords toggles HUD, /where prints position.",
        Resource::Template => "Template exported to Configuration/kusobo-template.zip. Copy it with Files; a Rust development computer is needed to build it.",
        Resource::ScriptSample => "Kusobo CPS Counter installed OFF. Enable in Installed, then Play. /cps moves the counter.",
        Resource::ScriptTemplate => "Script template exported to Configuration/kusobo-script-template.zip. Open GUIDE.txt inside it; edit on your phone, no computer needed.",
    }.into())
}
