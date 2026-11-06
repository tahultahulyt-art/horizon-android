//! Allowlisted document tree, independent of Android and the running game.
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

pub const ROOT: &str = "horizon";
const DIRECTORY: &str = "vnd.android.document/directory";
const MAX_CHILDREN: usize = 32768;
const CONFIG_FILES: &[&str] = &[
    "options.txt",
    "horizon-graphics.txt",
    "horizon-touch.json",
    "horizon-keypad.json",
];
static GAME_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn set_game_active(active: bool) {
    GAME_ACTIVE.store(active, std::sync::atomic::Ordering::Release);
}
fn world_access(id: &str) -> io::Result<()> {
    if matches!(id.split('/').nth(1), Some("worlds" | "configuration"))
        && GAME_ACTIVE.load(std::sync::atomic::Ordering::Acquire)
    {
        return Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "Return to the launcher before accessing worlds or configuration",
        ));
    }
    Ok(())
}

fn invalid() -> io::Error {
    io::Error::new(ErrorKind::InvalidInput, "Invalid Horizon document")
}
fn component(name: &str) -> io::Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.starts_with('.')
        || name.len() > 240
        || name.chars().any(|c| c.is_control() || "/\\:".contains(c))
    {
        return Err(invalid());
    }
    Ok(())
}

pub fn folder_id(profile: u32, folder: &str) -> String {
    if folder == "reports" {
        "reports".into()
    } else {
        format!("p{profile}/{folder}")
    }
}
pub fn profile(home: &Path) -> u32 {
    fs::read(home.join("android-launcher.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .and_then(|v| v["profile"].as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0)
}
struct Document {
    path: PathBuf,
    label: Option<&'static str>,
    reports: bool,
    configuration: bool,
}
fn shareable_report(name: &str) -> bool {
    (name.starts_with("horizon-diagnostics-") && name.ends_with(".zip")) || name.ends_with(".jsonl")
}
fn resolve(home: &Path, id: &str) -> io::Result<Document> {
    let mut parts = id.split('/');
    let first = parts.next().ok_or_else(invalid)?;
    let (base, label, reports) = if first == "reports" {
        (home.join("performance"), "Reports", true)
    } else {
        let number = first.strip_prefix('p').ok_or_else(invalid)?;
        let profile: u32 = number.parse().map_err(|_| invalid())?;
        // IDs are canonical and remain tied to their profile after switching.
        if format!("p{profile}") != first {
            return Err(invalid());
        }
        let root = if profile == 0 {
            home.to_owned()
        } else {
            home.join("profiles").join(format!("profile-{profile}"))
        };
        let (folder, label) = match parts.next() {
            Some("mods") => ("horizon-mods", "Mods"),
            Some("packs") => ("resourcepacks", "Resource Packs"),
            Some("shaders") => ("shaderpacks", "Shader Packs"),
            Some("worlds") => ("saves", "Worlds"),
            Some("configuration") => ("", "Configuration"),
            _ => return Err(invalid()),
        };
        (root.join(folder), label, false)
    };
    let tail: Vec<_> = parts.collect();
    let configuration = label == "Configuration";
    if configuration && !tail.is_empty() && (tail.len() != 1 || !CONFIG_FILES.contains(&tail[0])) {
        return Err(invalid());
    }
    for name in &tail {
        component(name)?;
    }
    // Internal session/error logs and raw OS traces are private. Sharing goes
    // through the bounded, redacted ZIP exporter; retain standalone performance
    // reports for existing workflows, including previously granted document IDs.
    if reports && !tail.is_empty() && (tail.len() != 1 || !shareable_report(tail[0])) {
        return Err(invalid());
    }
    let mut path = base.clone();
    for name in &tail {
        path.push(name);
    }
    // Reject every symlink, including a replaced profile or allowlisted root.
    let relative = path.strip_prefix(home).map_err(|_| invalid())?;
    let mut checked = home.to_owned();
    for part in relative.components() {
        checked.push(part);
        match fs::symlink_metadata(&checked) {
            Ok(m) if m.file_type().is_symlink() => return Err(invalid()),
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    if tail.is_empty() {
        fs::create_dir_all(&base)?;
    }
    Ok(Document {
        path,
        label: tail.is_empty().then_some(label),
        reports,
        configuration,
    })
}
fn row(home: &Path, id: &str) -> io::Result<Value> {
    if id == ROOT {
        return Ok(
            json!({"document_id": ROOT, "_display_name": "Horizon Android",
            "mime_type": DIRECTORY, "flags": 0}),
        );
    }
    let doc = resolve(home, id)?;
    let meta = fs::symlink_metadata(&doc.path)?;
    if !meta.is_dir() && !meta.is_file() {
        return Err(invalid());
    }
    let name = doc.label.map(str::to_owned).unwrap_or_else(|| {
        doc.path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    });
    let mime = if meta.is_dir() {
        DIRECTORY
    } else {
        match doc.path.extension().and_then(|s| s.to_str()) {
            Some("jsonl") => "application/x-ndjson",
            Some("zip" | "hmod" | "hshader") => "application/zip",
            Some("png") => "image/png",
            Some("json") => "application/json",
            Some("txt" | "wgsl" | "glsl" | "fsh" | "vsh" | "toml") => "text/plain",
            _ => "application/octet-stream",
        }
    };
    // Android DocumentsContract.Document flags. Fixed roots cannot be removed.
    let flags = if doc.configuration {
        if meta.is_file() { 2 } else { 0 }
    } else if doc.reports {
        0
    } else if meta.is_dir() {
        8
    } else {
        2
    };
    let flags = flags
        | if doc.label.is_none() && !doc.reports && !doc.configuration {
            4 | if meta.is_file() { 64 } else { 0 }
        } else {
            0
        };
    Ok(
        json!({"document_id": id, "_display_name": name, "mime_type": mime,
        "flags": flags, "_size": meta.len(), "last_modified": meta.modified().ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|t| t.as_millis().min(i64::MAX as u128) as i64).unwrap_or(0)}),
    )
}
pub fn children(home: &Path, id: &str) -> io::Result<Vec<Value>> {
    if id == ROOT {
        return [
            "mods",
            "packs",
            "shaders",
            "worlds",
            "configuration",
            "reports",
        ]
        .iter()
        .map(|folder| row(home, &folder_id(profile(home), folder)))
        .collect();
    }
    world_access(id)?;
    let doc = resolve(home, id)?;
    let mut result = Vec::new();
    for (index, entry) in fs::read_dir(doc.path)?.enumerate() {
        if index == MAX_CHILDREN {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Too many files in one folder to list safely",
            ));
        }
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if doc.configuration && !CONFIG_FILES.contains(&name) {
            continue;
        }
        if component(name).is_err() || entry.file_type()?.is_symlink() {
            continue;
        }
        let child = format!("{id}/{name}");
        if let Ok(value) = row(home, &child) {
            result.push(value);
        }
    }
    result.sort_by(|a, b| {
        a["_display_name"]
            .as_str()
            .cmp(&b["_display_name"].as_str())
    });
    Ok(result)
}
pub fn is_child(home: &Path, parent: &str, child: &str) -> bool {
    child != ROOT
        && child != parent
        && resolve(home, child).is_ok()
        && (parent == ROOT
            || (resolve(home, parent).is_ok()
                && child
                    .strip_prefix(parent)
                    .is_some_and(|rest| rest.starts_with('/'))))
}
pub fn open(home: &Path, id: &str, mode: &str) -> io::Result<File> {
    world_access(id)?;
    let doc = resolve(home, id)?;
    if !fs::metadata(&doc.path)?.is_file() || (doc.reports && mode != "r") {
        return Err(invalid());
    }
    let mut options = OpenOptions::new();
    match mode {
        "r" => {
            options.read(true);
        }
        "w" | "wt" => {
            options.write(true).truncate(true);
        }
        "wa" => {
            options.append(true);
        }
        "rw" => {
            options.read(true).write(true);
        }
        "rwt" => {
            options.read(true).write(true).truncate(true);
        }
        _ => return Err(invalid()),
    }
    options.open(doc.path)
}
pub fn call(home: &Path, op: &str, id: &str, arg: &str) -> io::Result<Value> {
    if !matches!(op, "document" | "path" | "child") {
        world_access(id)?;
    }
    match op {
        "roots" => Ok(json!([{"root_id": ROOT, "document_id": ROOT,
            "title": "Horizon Android", "flags": 18, "mime_types": "*/*"}])),
        "document" => Ok(json!([row(home, id)?])),
        "children" => Ok(json!(children(home, id)?)),
        "child" => Ok(json!(is_child(home, id, arg))),
        "path" => {
            row(home, id)?;
            let mut path = vec![ROOT.to_owned()];
            if id != ROOT {
                let mut parts = id.split('/');
                let first = parts.next().ok_or_else(invalid)?;
                let mut current = if first == "reports" {
                    first.into()
                } else {
                    format!("{first}/{}", parts.next().ok_or_else(invalid)?)
                };
                path.push(current.clone());
                for name in parts {
                    current.push('/');
                    current.push_str(name);
                    path.push(current.clone());
                }
            }
            if !arg.is_empty() {
                let offset = path.iter().position(|p| p == arg).ok_or_else(invalid)?;
                path.drain(..offset);
            }
            Ok(json!(path))
        }
        "create" | "mkdir" => {
            component(arg)?;
            let parent = resolve(home, id)?;
            if parent.reports || !parent.path.is_dir() || (parent.configuration && op == "mkdir") {
                return Err(invalid());
            }
            // create_new never overwrites a name. Let DocumentsUI request another name.
            let child = format!("{id}/{arg}");
            let target = resolve(home, &child)?;
            if op == "mkdir" {
                fs::create_dir(target.path)?;
            } else {
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(target.path)?;
            }
            Ok(json!(child))
        }
        "rename" | "delete" => {
            let doc = resolve(home, id)?;
            if doc.label.is_some() || doc.reports || doc.configuration {
                return Err(invalid());
            }
            if op == "delete" {
                // Nonempty directories must be emptied by the client explicitly.
                if doc.path.is_dir() {
                    fs::remove_dir(doc.path)?;
                } else {
                    fs::remove_file(doc.path)?;
                }
                Ok(Value::Null)
            } else {
                if !doc.path.is_file() {
                    return Err(invalid());
                }
                component(arg)?;
                let (parent, _) = id.rsplit_once('/').ok_or_else(invalid)?;
                let child = format!("{parent}/{arg}");
                let target = resolve(home, &child)?;
                if target.path.try_exists()? {
                    return Err(io::Error::from(ErrorKind::AlreadyExists));
                }
                rename_new(&doc.path, &target.path)?;
                Ok(json!(child))
            }
        }
        _ => Err(invalid()),
    }
}

fn rename_new(source: &Path, target: &Path) -> io::Result<()> {
    #[cfg(target_os = "android")]
    {
        // Android SELinux disallows hard links. renameat2 preserves no-overwrite
        // semantics even if the installer creates the target concurrently.
        rustix::fs::renameat_with(
            rustix::fs::CWD,
            source,
            rustix::fs::CWD,
            target,
            rustix::fs::RenameFlags::NOREPLACE,
        )?;
    }
    #[cfg(not(target_os = "android"))]
    {
        fs::hard_link(source, target)?;
        if let Err(error) = fs::remove_file(source) {
            let _ = fs::remove_file(target);
            return Err(error);
        }
    }
    Ok(())
}


// ---- In-launcher file manager ---------------------------------------------
// The manager browses and mutates the same allowlisted tree through these
// typed wrappers. The SAF surface above keeps DocumentsContract semantics.

/// One row of the in-launcher file manager.
pub struct Entry {
    pub id: String,
    pub name: String,
    pub dir: bool,
    pub size: u64,
    pub modified_ms: i64,
    /// Fixed rows (roots, reports, configuration files) cannot be renamed,
    /// moved or deleted, but folders among them can still be browsed.
    pub fixed: bool,
}

fn entry(home: &Path, id: &str) -> io::Result<Entry> {
    let doc = resolve(home, id)?;
    let meta = fs::symlink_metadata(&doc.path)?;
    if !meta.is_dir() && !meta.is_file() {
        return Err(invalid());
    }
    let name = doc.label.map(str::to_owned).unwrap_or_else(|| {
        doc.path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    });
    Ok(Entry {
        id: id.to_owned(),
        name,
        dir: meta.is_dir(),
        size: meta.len(),
        modified_ms: meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|t| t.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or(0),
        fixed: doc.label.is_some() || doc.reports || doc.configuration,
    })
}

/// Typed listing shared with the SAF `children` shape: same filters and order.
pub fn list(home: &Path, id: &str) -> io::Result<Vec<Entry>> {
    if id == ROOT {
        return [
            "mods",
            "packs",
            "shaders",
            "worlds",
            "configuration",
            "reports",
        ]
        .iter()
        .map(|folder| entry(home, &folder_id(profile(home), folder)))
        .collect();
    }
    world_access(id)?;
    let doc = resolve(home, id)?;
    if !fs::symlink_metadata(&doc.path)?.is_dir() {
        return Err(invalid());
    }
    let mut result = Vec::new();
    for (index, item) in fs::read_dir(&doc.path)?.enumerate() {
        if index == MAX_CHILDREN {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Too many files in one folder to list safely",
            ));
        }
        let item = item?;
        let name = item.file_name();
        let Some(name) = name.to_str() else { continue };
        if doc.configuration && !CONFIG_FILES.contains(&name) {
            continue;
        }
        if component(name).is_err() || item.file_type()?.is_symlink() {
            continue;
        }
        result.push(entry(home, &format!("{id}/{name}"))?);
    }
    result.sort_by(|a, b| b.dir.cmp(&a.dir).then_with(|| a.name.cmp(&b.name)));
    Ok(result)
}

/// Folders the manager may create, paste or extract into.
pub fn writable_dir(home: &Path, id: &str) -> io::Result<()> {
    world_access(id)?;
    let doc = resolve(home, id)?;
    if doc.reports || doc.configuration || !doc.path.is_dir() {
        return Err(invalid());
    }
    Ok(())
}

/// Parent document id for the manager's Up button.
pub fn parent_id(id: &str) -> Option<String> {
    if id == ROOT {
        return None;
    }
    // "reports" and p<profile>/<folder> sit directly under the root.
    match id.rsplit_once('/') {
        None => Some(ROOT.into()),
        Some((parent, _)) if parent == "reports" || !parent.contains('/') => Some(ROOT.into()),
        Some((parent, _)) => Some(parent.to_owned()),
    }
}

/// A mutable (non-fixed) entry.
fn mutable(home: &Path, id: &str) -> io::Result<Document> {
    world_access(id)?;
    let doc = resolve(home, id)?;
    if doc.label.is_some() || doc.reports || doc.configuration {
        return Err(invalid());
    }
    Ok(doc)
}

/// Resolve the path of a not-yet-created child for staged imports.
pub fn child_path(home: &Path, parent: &str, name: &str) -> io::Result<PathBuf> {
    writable_dir(home, parent)?;
    component(name)?;
    let child = format!("{parent}/{name}");
    Ok(resolve(home, &child)?.path)
}

/// First `name (n)` variant that does not exist yet under the parent.
pub fn unique_child(home: &Path, parent: &str, name: &str) -> io::Result<String> {
    component(name)?;
    if !child_path(home, parent, name)?.exists() {
        return Ok(name.to_owned());
    }
    let (stem, ext) = match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem, format!(".{ext}")),
        _ => (name, String::new()),
    };
    for i in 1..=10000u32 {
        let suffix = format!(" ({i}){ext}");
        let keep = 240usize.saturating_sub(suffix.len());
        let mut short: String = stem.chars().take(keep).collect();
        while short.len() > keep {
            short.pop();
        }
        let candidate = format!("{short}{suffix}");
        if component(&candidate).is_ok() && !child_path(home, parent, &candidate)?.exists() {
            return Ok(candidate);
        }
    }
    Err(io::Error::new(
        ErrorKind::AlreadyExists,
        "Too many filename conflicts",
    ))
}

pub fn make_dir(home: &Path, parent: &str, name: &str) -> io::Result<()> {
    let target = child_path(home, parent, name)?;
    fs::create_dir(&target)?;
    Ok(())
}

/// Rename a file or folder; never overwrites an existing name.
pub fn rename_entry(home: &Path, id: &str, name: &str) -> io::Result<String> {
    let doc = mutable(home, id)?;
    component(name)?;
    let (parent, _) = id.rsplit_once('/').ok_or_else(invalid)?;
    let child = format!("{parent}/{name}");
    let target = resolve(home, &child)?;
    if target.path.try_exists()? {
        return Err(io::Error::from(ErrorKind::AlreadyExists));
    }
    if doc.path.is_file() {
        rename_new(&doc.path, &target.path)?;
    } else {
        fs::rename(&doc.path, &target.path)?;
    }
    Ok(child)
}

/// Recursive delete that never follows symbolic links.
pub fn delete_tree(home: &Path, id: &str) -> io::Result<()> {
    let doc = mutable(home, id)?;
    delete_path(&doc.path)?;
    Ok(())
}

fn delete_path(path: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if meta.is_dir() && !meta.file_type().is_symlink() {
        for item in fs::read_dir(path)? {
            delete_path(&item?.path())?;
        }
        fs::remove_dir(path)
    } else {
        fs::remove_file(path)
    }
}

const MAX_EXTRACT_BYTES: u64 = 4 << 30;

fn copy_stream(
    mut read: impl io::Read,
    mut write: impl io::Write,
    limit: u64,
    progress: &mut dyn FnMut(u64),
) -> io::Result<u64> {
    let mut total = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        let n = io::Read::read(&mut read, &mut buffer)?;
        if n == 0 {
            return Ok(total);
        }
        total += n as u64;
        if total > limit {
            return Err(io::Error::new(ErrorKind::InvalidData, "File is too large"));
        }
        io::Write::write_all(&mut write, &buffer[..n])?;
        progress(total);
    }
}

/// Copy a file or folder into a writable folder. The source may be read-only
/// (reports, configuration); the destination may not sit inside the source.
pub fn copy_into(
    home: &Path,
    source: &str,
    destination: &str,
    progress: &mut dyn FnMut(u64),
) -> io::Result<String> {
    world_access(source)?;
    let src = resolve(home, source)?;
    if src.label.is_some() {
        return Err(invalid());
    }
    writable_dir(home, destination)?;
    let dst = resolve(home, destination)?;
    if dst.path.starts_with(&src.path) {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "Cannot copy a folder into itself",
        ));
    }
    let name = unique_child(home, destination, &entry(home, source)?.name)?;
    let target = child_path(home, destination, &name)?;
    copy_path(&src.path, &target, progress)?;
    Ok(name)
}

fn copy_path(source: &Path, target: &Path, progress: &mut dyn FnMut(u64)) -> io::Result<()> {
    let meta = fs::symlink_metadata(source)?;
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    if meta.is_dir() {
        fs::create_dir(target)?;
        for item in fs::read_dir(source)? {
            let item = item?;
            copy_path(&item.path(), &target.join(item.file_name()), progress)?;
        }
        Ok(())
    } else {
        let input = File::open(source)?;
        let output = OpenOptions::new().write(true).create_new(true).open(target)?;
        copy_stream(input, output, MAX_EXTRACT_BYTES, progress)?;
        Ok(())
    }
}

/// Move a file or folder into another writable folder on the same volume.
pub fn move_into(home: &Path, source: &str, destination: &str) -> io::Result<String> {
    let src = mutable(home, source)?;
    writable_dir(home, destination)?;
    let dst = resolve(home, destination)?;
    if dst.path.starts_with(&src.path) {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "Cannot move a folder into itself",
        ));
    }
    let name = unique_child(home, destination, &entry(home, source)?.name)?;
    let target = child_path(home, destination, &name)?;
    fs::rename(&src.path, &target)?;
    Ok(name)
}

/// Extract a ZIP into a new folder named after the archive. Returns the
/// created folder name. Path safety and size caps live in content_files.
pub fn extract_zip(
    home: &Path,
    archive: &str,
    destination: &str,
    cancelled: &dyn Fn() -> bool,
    progress: &mut dyn FnMut(u64),
) -> io::Result<String> {
    world_access(archive)?;
    let src = resolve(home, archive)?;
    if !src.path.is_file() {
        return Err(invalid());
    }
    writable_dir(home, destination)?;
    let stem = src
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| component(s).is_ok())
        .unwrap_or("extracted")
        .to_owned();
    let folder = unique_child(home, destination, &stem)?;
    let base = child_path(home, destination, &folder)?;
    fs::create_dir(&base)?;
    let result = crate::content_files::extract(&src.path, &base, cancelled, progress);
    match result {
        Ok(()) => {
            Ok(folder)
        }
        Err(error) => {
            let _ = delete_path(&base);
            Err(io::Error::new(ErrorKind::InvalidData, error.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundaries_profiles_and_non_overwriting_operations() {
        let home = std::env::temp_dir().join(format!(
            "horizon-documents-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&home).unwrap();
        let roots = children(&home, ROOT).unwrap();
        assert_eq!(roots.len(), 6);
        assert_eq!(roots[0]["document_id"], "p0/mods");
        for id in [
            "../tokens",
            "p0/mods/../account.json",
            "p0/mods/a\\b",
            "p0/accounts",
            "p00/mods",
            "p0/mods/.secret",
        ] {
            assert!(resolve(&home, id).is_err(), "{id}");
        }
        call(&home, "create", "p0/mods", "test.hmod").unwrap();
        assert!(call(&home, "create", "p0/mods", "test.hmod").is_err());
        fs::write(home.join("horizon-mods/test.hmod"), b"original").unwrap();
        call(&home, "create", "p0/mods", "other.hmod").unwrap();
        assert!(call(&home, "rename", "p0/mods/other.hmod", "test.hmod").is_err());
        assert_eq!(
            fs::read(home.join("horizon-mods/test.hmod")).unwrap(),
            b"original"
        );
        assert!(call(&home, "delete", "p0/mods", "").is_err());
        assert!(call(&home, "create", "reports", "bad.txt").is_err());
        fs::write(home.join("performance/report.jsonl"), b"{}").unwrap();
        fs::write(home.join("performance/current-session.json"), b"{}").unwrap();
        fs::write(home.join("performance/exit-1.trace"), b"private trace").unwrap();
        assert!(open(&home, "reports/exit-1.trace", "r").is_err());
        assert!(open(&home, "reports/current-session.json", "r").is_err());
        assert_eq!(children(&home, "reports").unwrap().len(), 1);
        assert!(open(&home, "reports/report.jsonl", "r").is_ok());
        assert!(open(&home, "reports/report.jsonl", "w").is_err());
        assert!(!is_child(&home, "p0/mods", "p0/packs/pack.zip"));
        assert!(is_child(&home, ROOT, "p0/mods/test.hmod"));
        assert_eq!(
            call(&home, "path", "p0/mods/test.hmod", "").unwrap(),
            json!(["horizon", "p0/mods", "p0/mods/test.hmod"])
        );
        assert!(call(&home, "path", "p0/mods/test.hmod", "p0/packs").is_err());
        fs::write(home.join("android-launcher.json"), br#"{"profile":7}"#).unwrap();
        assert_eq!(children(&home, ROOT).unwrap()[0]["document_id"], "p7/mods");
        assert!(open(&home, "p0/mods/test.hmod", "r").is_ok());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(home.join("performance"), home.join("horizon-mods/link"))
                .unwrap();
            assert!(resolve(&home, "p0/mods/link/report.jsonl").is_err());
            assert!(!is_child(&home, ROOT, "p0/mods/link/report.jsonl"));
        }
        fs::remove_dir_all(home).unwrap();
    }
}

#[cfg(test)]
mod world_tests {
    use super::*;
    #[test]
    fn worlds_roundtrip_and_busy_guard() {
        let home =
            std::env::temp_dir().join(format!("horizon-world-documents-{}", std::process::id()));
        fs::create_dir_all(&home).unwrap();
        let id = call(&home, "mkdir", "p0/worlds", "Example").unwrap();
        assert_eq!(id, "p0/worlds/Example");
        call(&home, "create", id.as_str().unwrap(), "level.dat").unwrap();
        use std::io::{Read, Write};
        open(&home, "p0/worlds/Example/level.dat", "wt")
            .unwrap()
            .write_all(b"fixture")
            .unwrap();
        let mut data = Vec::new();
        open(&home, "p0/worlds/Example/level.dat", "r")
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        assert_eq!(data, b"fixture");
        fs::write(home.join("options.txt"), b"renderDistance:16\n").unwrap();
        fs::write(home.join("account.json"), b"private").unwrap();
        fs::write(home.join("android-launcher.json"), br#"{"profile":0}"#).unwrap();
        let config = children(&home, "p0/configuration").unwrap();
        assert_eq!(config.len(), 1);
        assert_eq!(config[0]["_display_name"], "options.txt");
        assert_eq!(config[0]["flags"], 2);
        assert!(open(&home, "p0/configuration/account.json", "r").is_err());
        assert!(open(&home, "p0/configuration/android-launcher.json", "r").is_err());
        assert!(open(&home, "p0/configuration/../account.json", "r").is_err());
        assert!(call(&home, "mkdir", "p0/configuration", "options.txt").is_err());
        assert!(call(&home, "delete", "p0/configuration/options.txt", "").is_err());
        open(&home, "p0/configuration/options.txt", "wt")
            .unwrap()
            .write_all(b"renderDistance:12\n")
            .unwrap();
        assert_eq!(
            fs::read(home.join("options.txt")).unwrap(),
            b"renderDistance:12\n"
        );
        fs::create_dir_all(home.join("profiles/profile-7")).unwrap();
        fs::write(
            home.join("profiles/profile-7/options.txt"),
            b"renderDistance:8\n",
        )
        .unwrap();
        assert_eq!(children(&home, "p7/configuration").unwrap().len(), 1);
        set_game_active(true);
        let blocked = open(&home, "p0/worlds/Example/level.dat", "r").is_err()
            && call(&home, "delete", "p0/worlds/Example/level.dat", "").is_err()
            && children(&home, "p0/worlds").is_err();
        let config_blocked = open(&home, "p0/configuration/options.txt", "r").is_err()
            && children(&home, "p0/configuration").is_err();
        set_game_active(false);
        assert!(blocked);
        assert!(config_blocked);
        assert!(open(&home, "p0/worlds/Example/level.dat", "r").is_ok());
        fs::remove_dir_all(home).unwrap();
    }
}
