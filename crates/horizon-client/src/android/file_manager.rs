//! In-launcher file manager over the allowlisted game folders.
//!
//! Browsing and mutations go through document_store, the same symlink-safe
//! allowlist the system Files provider uses, so the manager cannot reach
//! outside the game's folders. Long work runs on a worker thread: blocking
//! the launcher thread is what tripped Android's input watchdog before.
use super::{
    chooser::Chooser,
    pages::{Action, button},
};
use crate::document_store::{self, Entry};
use horizon_client::{
    osk::Output,
    ui_theme::{Face, Ui, color},
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
};

pub(super) enum TextTarget {
    NewFolder,
    Rename(String),
}

pub(super) struct Files {
    pub cwd: String,
    entries: Vec<Entry>,
    writable: bool,
    picked: Option<usize>,
    clipboard: Option<(String, bool)>,
    text: String,
    text_target: Option<TextTarget>,
    worker: Option<Receiver<Result<String, String>>>,
    cancel: Arc<AtomicBool>,
    progress: Arc<Mutex<String>>,
}

fn size_label(bytes: u64) -> String {
    if bytes >= 1 << 30 {
        format!("{:.1} GiB", bytes as f64 / f64::from(1 << 30))
    } else if bytes >= 1 << 20 {
        format!("{:.1} MiB", bytes as f64 / f64::from(1 << 20))
    } else if bytes >= 1024 {
        format!("{} KiB", bytes / 1024)
    } else {
        format!("{bytes} B")
    }
}

/// Civil date from Unix milliseconds (Howard Hinnant's algorithm).
fn date_label(ms: i64) -> String {
    if ms <= 0 {
        return String::new();
    }
    let days = ms.div_euclid(86_400_000);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    format!("{year:04}-{month:02}-{day:02}")
}

fn pretty_path(id: &str) -> String {
    if id == document_store::ROOT {
        return "Horizon Android".into();
    }
    let mut parts = id.split('/');
    let first = parts.next().unwrap_or_default();
    let mut out: Vec<&str> = Vec::new();
    if first == "reports" {
        out.push("Reports");
    } else if let Some(folder) = parts.next() {
        out.push(match folder {
            "mods" => "Mods",
            "packs" => "Resource Packs",
            "shaders" => "Shader Packs",
            "worlds" => "Worlds",
            "configuration" => "Configuration",
            other => other,
        });
    }
    out.extend(parts);
    out.join(" / ")
}

impl Files {
    pub fn new() -> Self {
        Self {
            cwd: document_store::ROOT.into(),
            entries: Vec::new(),
            writable: false,
            picked: None,
            clipboard: None,
            text: String::new(),
            text_target: None,
            worker: None,
            cancel: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(Mutex::new(String::new())),
        }
    }

    pub fn open(&mut self, home: &Path) -> Option<String> {
        self.picked = None;
        self.cancel_edit();
        self.cwd = document_store::ROOT.into();
        self.refresh(home).err()
    }

    pub fn refresh(&mut self, home: &Path) -> Result<(), String> {
        match document_store::list(home, &self.cwd) {
            Ok(entries) => {
                self.writable = document_store::writable_dir(home, &self.cwd).is_ok();
                self.entries = entries;
                Ok(())
            }
            Err(error) => {
                if self.cwd != document_store::ROOT {
                    self.cwd = document_store::ROOT.into();
                    self.entries = document_store::list(home, &self.cwd).unwrap_or_default();
                }
                self.writable = false;
                Err(format!("Could not open that folder: {error}"))
            }
        }
    }

    pub fn busy(&self) -> bool {
        self.worker.is_some()
    }

    pub fn at_root(&self) -> bool {
        self.cwd == document_store::ROOT
    }

    pub fn clipboard(&self) -> bool {
        self.clipboard.is_some()
    }

    /// Finished worker result, if any. Refreshes the listing once.
    pub fn poll(&mut self, home: &Path) -> Option<String> {
        let result = self.worker.as_ref()?.try_recv().ok()?;
        self.worker = None;
        self.cancel.store(false, Ordering::Relaxed);
        *self.progress.lock().unwrap_or_else(|e| e.into_inner()) = String::new();
        let _ = self.refresh(home);
        Some(result.unwrap_or_else(|error| error))
    }

    fn spawn(
        &mut self,
        job: impl FnOnce(&AtomicBool, &Arc<Mutex<String>>) -> Result<String, String> + Send + 'static,
    ) {
        if self.worker.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        self.worker = Some(receiver);
        self.cancel.store(false, Ordering::Relaxed);
        let cancel = self.cancel.clone();
        let progress = self.progress.clone();
        let _ = std::thread::Builder::new()
            .name("File manager".into())
            .spawn(move || {
                let _ = sender.send(job(&cancel, &progress));
            });
    }

    pub fn up(&mut self, home: &Path) -> Option<String> {
        let parent = document_store::parent_id(&self.cwd)?;
        self.cwd = parent;
        self.picked = None;
        self.refresh(home).err()
    }

    pub fn open_entry(&mut self, home: &Path, index: usize) -> Option<String> {
        let entry = self.entries.get(index)?;
        if !entry.dir {
            return None;
        }
        self.cwd = entry.id.clone();
        self.picked = None;
        self.refresh(home).err()
    }

    /// The ops dialog for one row. Tapping a file row picks it directly.
    pub fn ops(&mut self, index: usize) -> Option<Chooser> {
        let entry = self.entries.get(index)?;
        self.picked = Some(index);
        let mut choices = Vec::new();
        if entry.dir {
            choices.push((
                "Open".into(),
                String::new(),
                Action::FilesOpen(index),
                false,
            ));
        }
        if !entry.fixed {
            choices.push(("Rename".into(), String::new(), Action::FilesRename, false));
        }
        choices.push((
            "Copy".into(),
            "Then open a folder and tap Paste".into(),
            Action::FilesCopy,
            false,
        ));
        if !entry.fixed {
            choices.push((
                "Move".into(),
                "Then open a folder and tap Paste".into(),
                Action::FilesCut,
                false,
            ));
        }
        if !entry.dir && entry.name.to_ascii_lowercase().ends_with(".zip") {
            choices.push((
                "Extract here".into(),
                "Into a new folder next to the archive".into(),
                Action::FilesExtract,
                false,
            ));
        }
        if !entry.fixed {
            choices.push((
                "Delete".into(),
                "Asks once more before removing".into(),
                Action::FilesDeleteAsk,
                false,
            ));
        }
        choices.push(("Cancel".into(), String::new(), Action::Dismiss, true));
        let detail = if entry.dir {
            "Folder".to_owned()
        } else {
            format!(
                "{}  /  {}",
                size_label(entry.size),
                date_label(entry.modified_ms)
            )
        };
        Some(Chooser::custom(
            "File",
            "Pick an action. Changes stay inside Horizon's folders.",
            Some(format!("{}  /  {detail}", entry.name)),
            choices,
        ))
    }

    fn picked_entry(&self) -> Option<&Entry> {
        self.picked.and_then(|i| self.entries.get(i))
    }

    pub fn picked_name(&self) -> Option<String> {
        self.picked_entry().map(|e| e.name.clone())
    }

    pub fn begin_rename(&mut self) -> Option<String> {
        let entry = self.picked_entry()?;
        let id = entry.id.clone();
        let name = entry.name.clone();
        self.text = name.clone();
        self.text_target = Some(TextTarget::Rename(id));
        Some(name)
    }

    pub fn begin_new_folder(&mut self) {
        self.text.clear();
        self.text_target = Some(TextTarget::NewFolder);
    }

    pub fn editing(&self) -> bool {
        self.text_target.is_some()
    }

    pub fn cancel_edit(&mut self) {
        self.text_target = None;
        self.text.clear();
    }

    /// Returns Some(status) when the edit finished (Enter or Close).
    pub fn edit(&mut self, out: &Output, home: &Path) -> Option<String> {
        match out {
            Output::Text(text) => {
                self.text.push_str(text);
                None
            }
            Output::Backspace => {
                self.text.pop();
                None
            }
            Output::Close => {
                self.cancel_edit();
                Some("Name edit closed".into())
            }
            Output::Enter => {
                let name = self.text.trim().to_owned();
                let target = self.text_target.take();
                self.text.clear();
                let Some(target) = target else {
                    return Some("Nothing to name".into());
                };
                if name.is_empty() {
                    return Some("Type a name first".into());
                }
                let result = match target {
                    TextTarget::NewFolder => document_store::make_dir(home, &self.cwd, &name)
                        .map(|_| format!("Folder created: {name}")),
                    TextTarget::Rename(id) => document_store::rename_entry(home, &id, &name)
                        .map(|_| format!("Renamed to {name}")),
                };
                let status = match result {
                    Ok(status) => {
                        crate::document_provider::mark_changed();
                        let _ = self.refresh(home);
                        status
                    }
                    Err(error) => format!("Could not apply the name: {error}"),
                };
                Some(status)
            }
        }
    }

    pub fn remember(&mut self, cut: bool) -> Option<String> {
        let entry = self.picked_entry()?;
        self.clipboard = Some((entry.id.clone(), cut));
        Some(format!(
            "{} selected. Open the destination folder and tap Paste.",
            if cut { "Move" } else { "Copy" }
        ))
    }

    pub fn paste(&mut self, home: PathBuf) -> Option<String> {
        let (source, cut) = self.clipboard.clone()?;
        let destination = self.cwd.clone();
        if cut {
            self.clipboard = None;
        }
        self.spawn(move |cancel, progress| {
            let name = if cut {
                document_store::move_into(&home, &source, &destination)
                    .map_err(|e| e.to_string())?
            } else {
                document_store::copy_into(&home, &source, &destination, &mut |bytes| {
                    *progress.lock().unwrap_or_else(|e| e.into_inner()) =
                        format!("Copying: {} copied", size_label(bytes));
                })
                .map_err(|e| e.to_string())?
            };
            let _ = cancel;
            crate::document_provider::mark_changed();
            Ok(format!(
                "{}d {name} here.",
                if cut { "Move" } else { "Copie" }
            ))
        });
        Some(if cut {
            "Moving...".into()
        } else {
            "Copying...".into()
        })
    }

    pub fn extract(&mut self, home: PathBuf) -> Option<String> {
        let entry = self.picked_entry()?;
        let archive = entry.id.clone();
        let destination = self.cwd.clone();
        self.spawn(move |cancel, progress| {
            let folder = document_store::extract_zip(
                &home,
                &archive,
                &destination,
                &|| cancel.load(Ordering::Relaxed),
                &mut |bytes| {
                    *progress.lock().unwrap_or_else(|e| e.into_inner()) =
                        format!("Extracting: {} written", size_label(bytes));
                },
            )
            .map_err(|e| e.to_string())?;
            crate::document_provider::mark_changed();
            Ok(format!("Extracted into {folder}."))
        });
        Some("Extracting...".into())
    }

    pub fn delete(&mut self, home: PathBuf) -> Option<String> {
        let entry = self.picked_entry()?;
        if entry.fixed {
            return Some("That item is managed by Horizon and cannot be deleted.".into());
        }
        let id = entry.id.clone();
        let name = entry.name.clone();
        self.picked = None;
        self.spawn(move |_, _| {
            document_store::delete_tree(&home, &id).map_err(|e| e.to_string())?;
            crate::document_provider::mark_changed();
            Ok(format!("Deleted {name}."))
        });
        Some("Deleting...".into())
    }

    pub fn draw(
        &mut self,
        ui: &mut Ui,
        targets: &mut Vec<([f32; 4], Action)>,
        list_targets: &mut Vec<([f32; 4], Action)>,
        scroll: f32,
    ) -> [f32; 3] {
        let (w, h) = (ui.w, ui.h);
        ui.text(Face::Medium, 18.0, 32.0, 140.0, color::TEXT, "File Manager");
        let crumb = ui.fit_text(Face::Regular, 13.0, &pretty_path(&self.cwd), w - 64.0);
        ui.text(Face::Regular, 13.0, 32.0, 164.0, color::MUTED, &crumb);
        // Toolbar.
        let mut buttons: Vec<(&str, Action)> = Vec::new();
        if !self.at_root() {
            buttons.push(("Up", Action::FilesUp));
        }
        if self.writable {
            buttons.push(("New folder", Action::FilesNewFolder));
            buttons.push(("Import file", Action::FilesImport));
            if self.clipboard() {
                buttons.push(("Paste", Action::FilesPaste));
            }
        }
        buttons.push(("In system Files", Action::FilesReveal));
        buttons.push(("Refresh", Action::FilesRefresh));
        let count = buttons.len();
        let columns = count.min(((w - 64.0) / 110.0).floor().max(2.0) as usize);
        let gap = 8.0;
        let width = (w - 64.0 - gap * (columns as f32 - 1.0)) / columns as f32;
        for (i, (label, action)) in buttons.into_iter().enumerate() {
            button(
                ui,
                targets,
                [
                    32.0 + (i % columns) as f32 * (width + gap),
                    186.0 + (i / columns) as f32 * 42.0,
                    width,
                    34.0,
                ],
                label,
                action,
                false,
            );
        }
        let top = 234.0 + ((count.saturating_sub(1)) / columns) as f32 * 42.0;
        let bottom = h - 96.0;
        let row_height = 44.0;
        let maximum = (self.entries.len() as f32 * row_height - (bottom - top)).max(0.0);
        if self.entries.is_empty() && !self.busy() {
            ui.text(
                Face::Regular,
                15.0,
                32.0,
                top + 12.0,
                color::MUTED,
                "Empty folder. Import a file or create a folder to begin.",
            );
        }
        if self.busy() {
            let progress = self
                .progress
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            ui.text(
                Face::Medium,
                16.0,
                32.0,
                top + 12.0,
                color::TEXT,
                if progress.is_empty() {
                    "Working..."
                } else {
                    progress.as_str()
                },
            );
        }
        for (index, entry) in self.entries.iter().enumerate() {
            let y = top + index as f32 * row_height - scroll;
            list_targets.push(([32.0, y, w - 140.0, row_height - 4.0], {
                if entry.dir {
                    Action::FilesOpen(index)
                } else {
                    Action::FilesOps(index)
                }
            }));
            list_targets.push((
                [w - 100.0, y, 68.0, row_height - 4.0],
                Action::FilesOps(index),
            ));
            if y < top || y + row_height - 4.0 > bottom {
                continue;
            }
            targets.push((
                [32.0, y, w - 140.0, row_height - 4.0],
                if entry.dir {
                    Action::FilesOpen(index)
                } else {
                    Action::FilesOps(index)
                },
            ));
            let name = if entry.dir {
                format!("{}/", entry.name)
            } else {
                entry.name.clone()
            };
            let name = ui.fit_text(Face::Medium, 17.0, &name, w - 320.0);
            ui.text(Face::Medium, 17.0, 32.0, y + 8.0, color::TEXT, &name);
            let detail = if entry.dir {
                "Folder".to_owned()
            } else {
                format!(
                    "{}  {}",
                    size_label(entry.size),
                    date_label(entry.modified_ms)
                )
            };
            ui.text(
                Face::Regular,
                13.0,
                w - 260.0,
                y + 11.0,
                color::MUTED,
                &ui.fit_text(Face::Regular, 13.0, &detail, 150.0),
            );
            button(
                ui,
                targets,
                [w - 100.0, y, 68.0, row_height - 10.0],
                "Ops",
                Action::FilesOps(index),
                false,
            );
        }
        [top, bottom, maximum]
    }
}
