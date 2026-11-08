//! Screenshot-requested in-game defaults. Startup calls this for new game
//! homes (seed); the standard build also merges them once into existing
//! installs (apply_to_existing).
use std::path::Path;
pub const OPTIONS: &str = "version:5023\npreferredGraphicsBackend:\"opengl\"\ngraphicsPreset:\"custom\"\nfov:95\ngamma:1.0\nrenderDistance:12\nsimulationDistance:12\nbiomeBlendRadius:2\nao:true\nrenderClouds:\"fast\"\nparticles:0\nmipmapLevels:4\nentityShadows:true\nentityDistanceScaling:1.0\nmenuBackgroundBlurriness:5\ncloudRange:128\ncutoutLeaves:true\nimprovedTransparency:false\ntextureFiltering:0\nmaxAnisotropyBit:2\nweatherRadius:10\nshowAutosaveIndicator:true\nvignette:true\nattackIndicator:1\nchunkSectionFadeInTime:0.75\nmaxFps:260\nenableVsync:false\ninactivityFpsLimit:\"afk\"\nguiScale:0\nfullscreen:false\nexclusiveFullscreen:false\nsoundCategory_master:1.0\nsoundCategory_music:0.0\nsoundCategory_record:1.0\nsoundCategory_weather:1.0\nsoundCategory_block:1.0\nsoundCategory_hostile:1.0\nsoundCategory_neutral:1.0\nsoundCategory_player:1.0\nsoundCategory_ambient:1.0\nsoundCategory_voice:1.0\nsoundCategory_ui:1.0\nshowSubtitles:false\ndirectionalAudio:false\n";
pub const GRAPHICS: &str = "preset:custom\ngraphicsApi:opengl\nmotionBlur:true\nentityCulling:true\nditherCutoutMips:true\nupscaling:quality\nframePacing:precise\nbatteryMode:true\noverlay:fps\npickedShaderPack:\n";
pub const EXTRAS: &str = "fullbright:true\ndynamicLights:true\nownNameTag:true\nmouseTweaks:true\n";
/// One-time merge of these defaults into an existing game home: every key
/// the defaults set is written, keys they do not mention keep the player's
/// values. The marker makes it idempotent.
const MIGRATION_MARKER: &str = "risiya-style-defaults-v1";

pub fn apply_to_existing(home: &Path) -> Result<bool, String> {
    let marker = home.join(MIGRATION_MARKER);
    if marker.try_exists().map_err(|e| e.to_string())? {
        return Ok(false);
    }
    std::fs::create_dir_all(home).map_err(|e| e.to_string())?;
    for (name, defaults) in [
        ("options.txt", OPTIONS),
        ("horizon-graphics.txt", GRAPHICS),
        ("horizon-extras.txt", EXTRAS),
    ] {
        merge_keys(&home.join(name), defaults)?;
    }
    std::fs::write(marker, b"1").map_err(|e| e.to_string())?;
    Ok(true)
}

fn merge_keys(path: &Path, defaults: &str) -> Result<(), String> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = existing.lines().map(str::to_string).collect();
    for line in defaults.lines() {
        let Some((key, _)) = line.split_once(':') else {
            continue;
        };
        match lines
            .iter()
            .position(|l| l.split_once(':').is_some_and(|(k, _)| k == key))
        {
            Some(i) => lines[i] = line.to_string(),
            None => lines.push(line.to_string()),
        }
    }
    let mut out = lines.join("\n");
    out.push('\n');
    std::fs::write(path, out).map_err(|e| e.to_string())
}

pub fn seed(home: &Path) -> Result<bool, String> {
    // Any of these files means the game home was already used. Malformed files count as existing.
    for name in ["options.txt", "horizon-graphics.txt", "horizon-extras.txt"] {
        match std::fs::symlink_metadata(home.join(name)) {
            Ok(_) => return Ok(false),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string())?,
        }
    }
    std::fs::create_dir_all(home).map_err(|e| e.to_string())?;
    use std::io::Write;
    // If interrupted, never overwrite an existing file on the next run.
    for (name, text) in [
        ("options.txt", OPTIONS),
        ("horizon-graphics.txt", GRAPHICS),
        ("horizon-extras.txt", EXTRAS),
    ] {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(home.join(name))
            .map_err(|e| e.to_string())?;
        f.write_all(text.as_bytes())
            .and_then(|_| f.sync_all())
            .map_err(|e| e.to_string())?;
    }
    Ok(true)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn p(t: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("risiya-game-{t}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }
    #[test]
    fn new_defaults_once_no_shader() {
        let p = p("new");
        assert!(seed(&p).unwrap());
        assert_eq!(
            std::fs::read_to_string(p.join("options.txt")).unwrap(),
            OPTIONS
        );
        assert_eq!(
            std::fs::read_to_string(p.join("horizon-graphics.txt")).unwrap(),
            GRAPHICS
        );
        assert_eq!(
            std::fs::read_to_string(p.join("horizon-extras.txt")).unwrap(),
            EXTRAS
        );
        assert!(GRAPHICS.lines().any(|s| s == "motionBlur:true"));
        assert!(GRAPHICS.lines().any(|s| s == "batteryMode:true"));
        assert!(OPTIONS.lines().any(|s| s == "fov:95"));
        assert!(EXTRAS.lines().any(|s| s == "fullbright:true"));
        std::fs::write(p.join("options.txt"), b"user edit").unwrap();
        assert!(!seed(&p).unwrap());
        assert_eq!(std::fs::read(p.join("options.txt")).unwrap(), b"user edit");
        std::fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn existing_install_merges_once() {
        let p = p("merge");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(
            p.join("options.txt"),
            b"version:5023\nfov:70\nsoundCategory_music:0.5\n",
        )
        .unwrap();
        assert!(apply_to_existing(&p).unwrap());
        let merged = std::fs::read_to_string(p.join("options.txt")).unwrap();
        // Default keys are applied; a player key outside the defaults is kept.
        assert!(merged.lines().any(|s| s == "fov:95"));
        assert!(merged.lines().any(|s| s == "renderDistance:12"));
        assert!(merged.lines().any(|s| s == "soundCategory_music:0.0"));
        assert!(merged.lines().any(|s| s == "version:5023"));
        // Missing files are created; the graphics defaults land too.
        let graphics = std::fs::read_to_string(p.join("horizon-graphics.txt")).unwrap();
        assert!(graphics.lines().any(|s| s == "motionBlur:true"));
        // The marker makes a second run a no-op, so later edits survive.
        std::fs::write(p.join("options.txt"), b"fov:80\n").unwrap();
        assert!(!apply_to_existing(&p).unwrap());
        assert_eq!(
            std::fs::read_to_string(p.join("options.txt")).unwrap(),
            "fov:80\n"
        );
        std::fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn partial_or_malformed_existing_files_never_replaced() {
        for name in ["options.txt", "horizon-graphics.txt", "horizon-extras.txt"] {
            let p = p(name);
            std::fs::create_dir_all(&p).unwrap();
            std::fs::write(p.join(name), b"{").unwrap();
            assert!(!seed(&p).unwrap());
            assert_eq!(std::fs::read(p.join(name)).unwrap(), b"{");
            std::fs::remove_dir_all(p).unwrap();
        }
    }
}
