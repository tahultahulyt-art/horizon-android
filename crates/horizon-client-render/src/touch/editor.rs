use super::*;

pub struct Editor {
    pub selected: Option<Control>,
    option: Option<usize>,
    apply_all: bool,
    original: Settings,
    drag: Option<(u64, Drag)>,
    error: Option<String>,
    palette: bool,
    selected_custom: Option<usize>,
    preset_menu: bool,
    key_picker: bool,
    key_page: usize,
    raw_keys: bool,
    search: String,
    search_editing: bool,
    chord_mode: bool,
    chord_keys: Vec<String>,
}

enum Drag {
    Control([f32; 2]),
    Slider(usize),
}

impl TouchControls {
    pub fn open_editor(&mut self) {
        self.clear();
        self.editor = Some(Editor {
            selected: None,
            option: None,
            apply_all: false,
            original: self.settings.clone(),
            drag: None,
            error: None,
            palette: false,
            selected_custom: None,
            preset_menu: false,
            key_picker: false,
            key_page: 0,
            raw_keys: true,
            search: String::new(),
            search_editing: false,
            chord_mode: false,
            chord_keys: Vec::new(),
        });
    }
    pub fn cancel_editor(&mut self) {
        if let Some(e) = self.editor.take() {
            self.settings = e.original;
        }
        self.clear();
    }
    /// The editor owns every finger: editing cannot move, mine or open menus.
    /// Returns true when Done/Cancel closed the editor.
    pub fn edit_touch(&mut self, id: u64, p: [f32; 2], started: bool, ended: bool) -> bool {
        let Some(mut e) = self.editor.take() else {
            return false;
        };
        self.clicked |= started;
        if self.edit_custom(&mut e, id, p, started, ended) {
            self.editor = Some(e);
            return false;
        }
        if started {
            if layout::contains([0.02, 0.04, 0.105, 0.075], p) {
                match self.settings.save() {
                    Ok(()) => {
                        self.clear();
                        return true;
                    }
                    Err(error) => e.error = Some(error.to_string()),
                }
            } else if layout::contains([0.135, 0.04, 0.105, 0.075], p) {
                self.settings = e.original;
                self.clear();
                return true;
            } else if layout::contains([0.25, 0.04, 0.105, 0.075], p) {
                if self.reset_active_preset() {
                    self.editor = Some(e);
                    return false;
                }
                self.settings.layouts[self.settings.mode.index()] =
                    Settings::default().layouts[self.settings.mode.index()];
                self.settings.button_visibility = [0; 14];
                for (i, button) in self.settings.custom_buttons.iter_mut().enumerate() {
                    button.layouts[self.settings.mode.index()] =
                        CustomButton::new(button.key.clone(), i).layouts
                            [self.settings.mode.index()];
                }
            } else if layout::contains([0.02, 0.13, 0.335, 0.075], p) {
                e.palette = !e.palette;
                e.selected = None;
                e.selected_custom = None;
                e.key_picker = false;
                e.drag = None;
            } else if e.palette {
                for (index, control) in Control::ALL.into_iter().enumerate() {
                    if layout::contains(palette_rect(index), p) {
                        let show = !self.visible(control);
                        self.settings.button_visibility[index] = if show { 1 } else { 2 };
                    }
                }
            } else if layout::contains([0.32, 0.88, 0.36, 0.07], p) && e.selected.is_some() {
                self.settings.button_visibility[e.selected.unwrap() as usize] = 2;
                e.selected = None;
                e.drag = None;
            } else if layout::contains([0.32, 0.61, 0.36, 0.06], p)
                && e.selected.is_some()
                && e.option.is_some()
            {
                e.apply_all = !e.apply_all;
            } else if layout::contains([0.32, 0.68, 0.36, 0.07], p)
                && e.selected.is_some()
                && e.option.is_some()
            {
                e.drag = Some((id, Drag::Slider(e.option.unwrap())));
            } else if layout::contains([0.32, 0.79, 0.17, 0.07], p) && e.selected.is_some() {
                e.option = Some(1);
            } else if layout::contains([0.51, 0.79, 0.17, 0.07], p) && e.selected.is_some() {
                e.option = Some(0);
            } else if let Some(c) = Control::ALL.into_iter().rev().find(|c| {
                self.visible(*c) && layout::contains(self.placement(*c).rect(self.aspect()), p)
            }) {
                e.selected = Some(c);
                e.option = None;
                let center = self.placement(c).center;
                e.drag = Some((id, Drag::Control([p[0] - center[0], p[1] - center[1]])));
            } else {
                e.selected = None;
                e.option = None;
            }
        }
        if let Some((owner, drag)) = &e.drag
            && *owner == id
            && let Some(selected) = e.selected
        {
            let aspect = self.aspect();
            let slot = self.settings.slot(selected);
            let place = &mut self.settings.layouts[self.settings.mode.index()][slot];
            match drag {
                Drag::Control(offset) => {
                    let mut center = [p[0] - offset[0], p[1] - offset[1]];
                    if self.settings.left_handed {
                        center[0] = 1.0 - center[0];
                    }
                    // Match the displayed size on narrow portrait surfaces.
                    // A stored button may be wider than the screen; its raw
                    // size would otherwise produce inverted clamp bounds.
                    let size = (place.size * self.settings.button_scale)
                        .min(aspect)
                        .min(1.0);
                    place.center = [
                        center[0].clamp(size / aspect / 2.0, 1.0 - size / aspect / 2.0),
                        center[1].clamp(size / 2.0, 1.0 - size / 2.0),
                    ];
                }
                Drag::Slider(index) => {
                    let value = ((p[0] - 0.335) / 0.33).clamp(0.0, 1.0);
                    for (i, place) in self.settings.layouts[self.settings.mode.index()]
                        .iter_mut()
                        .enumerate()
                    {
                        if i == slot || e.apply_all {
                            if *index == 0 {
                                place.size = 0.045 + value * 0.405;
                            } else {
                                place.opacity = value;
                            }
                        }
                    }
                }
            }
            if ended {
                e.drag = None;
            }
        }

        self.editor = Some(e);
        false
    }
    pub(super) fn draw_editor_toolbar(&self, b: &mut HudBatch, a: &HudAtlas) {
        for (rect, label) in [
            ([0.02, 0.04, 0.105, 0.075], "Done"),
            ([0.135, 0.04, 0.105, 0.075], "Cancel"),
            ([0.25, 0.04, 0.105, 0.075], "Reset"),
            ([0.37, 0.04, 0.16, 0.075], "Layouts..."),
            ([0.02, 0.13, 0.335, 0.075], "Add buttons..."),
        ] {
            plate(b, a, rect, label, false);
        }
    }

    pub(super) fn draw_editor(&self, b: &mut HudBatch, a: &HudAtlas) {
        let e = self.editor.as_ref().unwrap();
        let (w, h) = (b.width, b.height);
        let text = rgba(245, 245, 245, 255);
        if let Some(selected) = e.selected {
            plate(b, a, [0.32, 0.88, 0.36, 0.07], "Remove button", false);
            let p = self.placement(selected);
            plate(
                b,
                a,
                [0.32, 0.79, 0.17, 0.07],
                "Opacity",
                e.option == Some(1),
            );
            plate(b, a, [0.51, 0.79, 0.17, 0.07], "Size", e.option == Some(0));
            if let Some(option) = e.option {
                b.fill(
                    a,
                    w * 0.31,
                    h * 0.59,
                    w * 0.38,
                    h * 0.18,
                    rgba(15, 15, 15, 235),
                );
                plate(
                    b,
                    a,
                    [0.32, 0.61, 0.36, 0.06],
                    if e.apply_all {
                        "Apply to all: On"
                    } else {
                        "Apply to all: Off"
                    },
                    e.apply_all,
                );
                let value = if option == 0 {
                    (p.size - 0.045) / 0.405
                } else {
                    p.opacity
                };
                let percent = if option == 0 {
                    p.size * 100.0
                } else {
                    p.opacity * 100.0
                };
                b.text(
                    a,
                    &format!("{} {:.0}%", selected.name(), percent),
                    w * 0.325,
                    h * 0.68,
                    text,
                    true,
                );
                nine(
                    b,
                    a,
                    "ui/slider_background",
                    1.0,
                    [w * 0.335, h * 0.72, w * 0.33, h * 0.025],
                    rgba(255, 255, 255, 255),
                );
                nine(
                    b,
                    a,
                    "ui/slider_button_default",
                    2.0,
                    [w * (0.335 + value * 0.33) - 3.0, h * 0.707, 6.0, h * 0.05],
                    rgba(255, 255, 255, 255),
                );
            }
        }
        if e.palette {
            b.fill(
                a,
                w * 0.2,
                h * 0.14,
                w * 0.6,
                h * 0.72,
                rgba(15, 15, 15, 245),
            );
            for (index, control) in Control::ALL.into_iter().enumerate() {
                let shown = self.visible(control);
                plate(
                    b,
                    a,
                    palette_rect(index),
                    &format!("{} {}", if shown { "[-]" } else { "[+]" }, control.name()),
                    shown,
                );
            }
        }
        self.draw_custom_editor(b, a, e);
        if e.error.is_some() {
            b.text(
                a,
                "Save failed. Retry Done.",
                w * 0.33,
                h * 0.88,
                rgba(255, 120, 120, 255),
                true,
            );
        }
    }
}
fn palette_rect(index: usize) -> [f32; 4] {
    [
        0.22 + (index % 2) as f32 * 0.29,
        0.17 + (index / 2) as f32 * 0.11,
        0.27,
        0.09,
    ]
}
fn plate(b: &mut HudBatch, a: &HudAtlas, [x, y, w, h]: [f32; 4], label: &str, on: bool) {
    let (bw, bh) = (b.width, b.height);
    nine(
        b,
        a,
        "ui/button_borderless_light",
        1.0,
        [x * bw, y * bh, w * bw, h * bh],
        if on {
            rgba(100, 210, 90, 255)
        } else {
            rgba(255, 255, 255, 255)
        },
    );
    if let Some((first, second)) = label.split_once(" & ") {
        b.text(
            a,
            &format!("{first} &"),
            x * bw + 3.0,
            y * bh + (h * bh - 20.0) / 2.0,
            rgba(35, 35, 35, 255),
            false,
        );
        b.text(
            a,
            second,
            x * bw + 3.0,
            y * bh + (h * bh - 20.0) / 2.0 + 10.0,
            rgba(35, 35, 35, 255),
            false,
        );
    } else {
        let mut display = label.to_owned();
        while HudBatch::text_width(a, &display) > w * bw - 6.0 && !display.is_empty() {
            display.pop();
        }
        b.text(
            a,
            &display,
            x * bw + 3.0,
            y * bh + (h * bh - 8.0) / 2.0,
            rgba(35, 35, 35, 255),
            false,
        );
    }
}

impl TouchControls {
    fn edit_custom(
        &mut self,
        e: &mut Editor,
        id: u64,
        p: [f32; 2],
        started: bool,
        ended: bool,
    ) -> bool {
        if started && layout::contains([0.37, 0.04, 0.16, 0.075], p) {
            e.preset_menu = !e.preset_menu;
            e.key_picker = false;
            e.palette = false;
            return true;
        }
        if e.preset_menu {
            if started {
                for (i, name) in super::presets::NAMES.into_iter().enumerate() {
                    if layout::contains([0.22, 0.26 + i as f32 * 0.13, 0.56, 0.10], p) {
                        self.choose_preset(name);
                        e.preset_menu = false;
                        e.selected = None;
                        e.selected_custom = None;
                        e.option = None;
                        e.drag = None;
                        break;
                    }
                }
            }
            return true;
        }
        if p[1] < 0.13 && started {
            return false;
        }
        if e.key_picker {
            if started {
                if e.search_editing {
                    if layout::contains([0.08, 0.84, 0.24, 0.075], p) {
                        e.search.pop();
                    } else if layout::contains([0.38, 0.84, 0.24, 0.075], p) {
                        e.search.clear();
                    } else if layout::contains([0.68, 0.84, 0.24, 0.075], p) {
                        e.search_editing = false;
                    } else {
                        for (i, c) in SEARCH_CHARS.chars().enumerate() {
                            if layout::contains(search_key_rect(i), p) && e.search.len() < 32 {
                                e.search.push(c);
                                break;
                            }
                        }
                    }
                    e.key_page = 0;
                } else if layout::contains([0.08, 0.16, 0.26, 0.06], p) {
                    e.raw_keys = false;
                    e.key_page = 0;
                } else if layout::contains([0.37, 0.16, 0.26, 0.06], p) {
                    e.raw_keys = true;
                    e.key_page = 0;
                } else if layout::contains([0.66, 0.16, 0.26, 0.06], p) {
                    e.chord_mode = !e.chord_mode;
                    e.chord_keys.clear();
                } else if layout::contains([0.08, 0.24, 0.84, 0.065], p) {
                    e.search_editing = true;
                } else if layout::contains([0.08, 0.85, 0.24, 0.075], p) {
                    e.key_page = e.key_page.saturating_sub(1);
                } else if layout::contains([0.38, 0.85, 0.24, 0.075], p) {
                    if e.chord_mode {
                        if let Some(index) = e.selected_custom
                            && !e.chord_keys.is_empty()
                        {
                            self.settings.custom_buttons[index].key = if e.chord_keys.len() == 1 {
                                e.chord_keys[0].clone()
                            } else {
                                format!("chord:{}", e.chord_keys.join("+"))
                            };
                            self.settings.custom_buttons[index].caption.clear();
                            e.key_picker = false;
                        }
                    } else {
                        e.key_picker = false;
                    }
                } else if layout::contains([0.68, 0.85, 0.24, 0.075], p) {
                    e.key_page = (e.key_page + 1).min(
                        picker_choices(e.raw_keys, &e.search)
                            .len()
                            .saturating_sub(1)
                            / KEYS_PER_PAGE,
                    );
                } else {
                    for (i, (token, _)) in picker_choices(e.raw_keys, &e.search)
                        .into_iter()
                        .skip(e.key_page * KEYS_PER_PAGE)
                        .take(KEYS_PER_PAGE)
                        .enumerate()
                    {
                        if layout::contains(key_rect(i), p)
                            && let Some(index) = e.selected_custom
                        {
                            if e.chord_mode {
                                if let Some(at) = e.chord_keys.iter().position(|key| *key == token)
                                {
                                    e.chord_keys.remove(at);
                                } else if e.chord_keys.len() < 8 {
                                    e.chord_keys.push(token);
                                }
                            } else {
                                self.settings.custom_buttons[index].key = token;
                                self.settings.custom_buttons[index].caption.clear();
                                e.key_picker = false;
                            }
                            break;
                        }
                    }
                }
            }
            return true;
        }
        if e.palette && started && layout::contains([0.22, 0.84, 0.56, 0.07], p) {
            if self.settings.custom_buttons.len() < 64 {
                let index = self.settings.custom_buttons.len();
                self.settings
                    .custom_buttons
                    .push(CustomButton::new("key.keyboard.unknown".into(), index));
                e.selected = None;
                e.selected_custom = Some(index);
                e.key_picker = true;
                e.raw_keys = true;
                e.chord_mode = false;
                e.chord_keys.clear();
                e.search.clear();
                e.search_editing = false;
                e.key_page = 0;
                e.palette = false;
                e.option = None;
                e.drag = None;
            }
            return true;
        }
        if e.palette {
            return false;
        }
        if started && e.selected_custom.is_some() {
            if layout::contains([0.32, 0.54, 0.36, 0.07], p) {
                e.key_picker = true;
                e.chord_mode = false;
                e.chord_keys.clear();
                e.search.clear();
                e.search_editing = false;
                e.key_page = 0;
                e.drag = None;
                return true;
            }
            if layout::contains([0.32, 0.88, 0.36, 0.07], p) {
                self.settings
                    .custom_buttons
                    .remove(e.selected_custom.unwrap());
                e.selected_custom = None;
                e.drag = None;
                return true;
            }
            if layout::contains([0.32, 0.79, 0.17, 0.07], p) {
                e.option = Some(1);
                return true;
            }
            if layout::contains([0.51, 0.79, 0.17, 0.07], p) {
                e.option = Some(0);
                return true;
            }
            if layout::contains([0.32, 0.68, 0.36, 0.07], p)
                && let Some(option) = e.option
            {
                e.drag = Some((id, Drag::Slider(option)));
            } else {
                e.selected_custom = None;
                e.drag = None;
            }
        }
        if started
            && e.drag.is_none()
            && let Some(index) = self.custom_at(p)
        {
            let center = self.custom_placement(index).center;
            e.selected = None;
            e.selected_custom = Some(index);
            e.option = None;
            e.apply_all = false;
            e.drag = Some((id, Drag::Control([p[0] - center[0], p[1] - center[1]])));
        }
        let Some(index) = e.selected_custom else {
            return false;
        };
        if let Some((owner, drag)) = &e.drag
            && *owner == id
        {
            let aspect = self.aspect();
            let scale = self.settings.button_scale;
            let mirror = self.settings.left_handed;
            let shape = self.settings.custom_buttons[index].shape;
            let place =
                &mut self.settings.custom_buttons[index].layouts[self.settings.mode.index()];
            match drag {
                Drag::Control(offset) => {
                    let mut center = [p[0] - offset[0], p[1] - offset[1]];
                    if mirror {
                        center[0] = 1.0 - center[0];
                    }
                    let size = (place.size * scale).min(aspect).min(1.0);
                    let width = (size * shape / aspect).min(1.0);
                    place.center = [
                        center[0].clamp(width / 2.0, 1.0 - width / 2.0),
                        center[1].clamp(size / 2.0, 1.0 - size / 2.0),
                    ];
                }
                Drag::Slider(option) => {
                    let value = ((p[0] - 0.335) / 0.33).clamp(0.0, 1.0);
                    if *option == 0 {
                        place.size = 0.045 + value * 0.405;
                    } else {
                        place.opacity = value;
                    }
                }
            }
            if ended {
                e.drag = None;
            }
        }
        true
    }
    pub fn capture_custom_key(&mut self, name: &str) -> bool {
        if crate::keybinds::key_code(name).is_none() {
            return false;
        }
        if let Some(e) = &mut self.editor
            && e.key_picker
            && !e.search_editing
            && !e.chord_mode
            && let Some(i) = e.selected_custom
        {
            self.settings.custom_buttons[i].key = name.into();
            self.settings.custom_buttons[i].caption.clear();
            e.key_picker = false;
            true
        } else {
            false
        }
    }
    fn draw_custom_editor(&self, b: &mut HudBatch, a: &HudAtlas, e: &Editor) {
        let (w, h) = (b.width, b.height);
        if e.preset_menu {
            b.fill(
                a,
                b.width * 0.18,
                b.height * 0.16,
                b.width * 0.64,
                b.height * 0.71,
                rgba(15, 15, 15, 250),
            );
            b.text(
                a,
                "Choose layout - previous edits are kept",
                w * 0.22,
                h * 0.2,
                u32::MAX,
                false,
            );
            for (i, name) in super::presets::NAMES.into_iter().enumerate() {
                plate(
                    b,
                    a,
                    [0.22, 0.26 + i as f32 * 0.13, 0.56, 0.10],
                    name,
                    self.settings.active_layout == name,
                );
            }
            return;
        }
        if e.palette {
            plate(b, a, [0.22, 0.84, 0.56, 0.07], "Create button...", false);
        }
        if let Some(i) = e.selected_custom
            && !e.palette
            && !e.key_picker
        {
            let p = self.settings.custom_buttons[i].layouts[self.settings.mode.index()];
            plate(
                b,
                a,
                [0.32, 0.54, 0.36, 0.07],
                &format!("Action: {}", self.settings.custom_buttons[i].label()),
                false,
            );
            plate(
                b,
                a,
                [0.32, 0.79, 0.17, 0.07],
                "Opacity",
                e.option == Some(1),
            );
            plate(b, a, [0.51, 0.79, 0.17, 0.07], "Size", e.option == Some(0));
            plate(b, a, [0.32, 0.88, 0.36, 0.07], "Remove button", false);
            if let Some(option) = e.option {
                let value = if option == 0 {
                    (p.size - 0.045) / 0.405
                } else {
                    p.opacity
                };
                b.fill(
                    a,
                    w * 0.3,
                    h * 0.66,
                    w * 0.4,
                    h * 0.11,
                    rgba(15, 15, 15, 235),
                );
                let percent = if option == 0 {
                    p.size * 100.0
                } else {
                    p.opacity * 100.0
                };
                b.text(
                    a,
                    &format!(
                        "{} {:.0}%",
                        if option == 0 { "Size" } else { "Opacity" },
                        percent
                    ),
                    w * 0.325,
                    h * 0.68,
                    rgba(245, 245, 245, 255),
                    true,
                );
                nine(
                    b,
                    a,
                    "ui/slider_background",
                    1.0,
                    [w * 0.335, h * 0.72, w * 0.33, h * 0.025],
                    rgba(255, 255, 255, 255),
                );
                nine(
                    b,
                    a,
                    "ui/slider_button_default",
                    2.0,
                    [w * (0.335 + value * 0.33) - 3.0, h * 0.707, 6.0, h * 0.05],
                    rgba(255, 255, 255, 255),
                );
            }
        }
        if e.key_picker {
            b.fill(
                a,
                w * 0.05,
                h * 0.14,
                w * 0.9,
                h * 0.81,
                rgba(15, 15, 15, 250),
            );
            plate(b, a, [0.08, 0.16, 0.26, 0.06], "Game actions", !e.raw_keys);
            plate(b, a, [0.37, 0.16, 0.26, 0.06], "All keys", e.raw_keys);
            plate(b, a, [0.66, 0.16, 0.26, 0.06], "Multi-key", e.chord_mode);
            let search_label = if e.search.is_empty() {
                "Search keys... (tap)".to_owned()
            } else {
                format!("Search: {}", e.search)
            };
            plate(
                b,
                a,
                [0.08, 0.24, 0.84, 0.065],
                &search_label,
                e.search_editing,
            );
            let choices = picker_choices(e.raw_keys, &e.search);
            if e.search_editing {
                b.text(
                    a,
                    "Type a key name, then Show results",
                    w * 0.08,
                    h * 0.33,
                    u32::MAX,
                    false,
                );
                for (i, c) in SEARCH_CHARS.chars().enumerate() {
                    plate(
                        b,
                        a,
                        search_key_rect(i),
                        &c.to_ascii_uppercase().to_string(),
                        false,
                    );
                }
                plate(b, a, [0.08, 0.84, 0.24, 0.075], "Backspace", false);
                plate(b, a, [0.38, 0.84, 0.24, 0.075], "Clear", false);
                plate(b, a, [0.68, 0.84, 0.24, 0.075], "Show results", false);
            } else {
                let pages = choices.len().saturating_sub(1) / KEYS_PER_PAGE + 1;
                b.text(
                    a,
                    &format!("{} keys - page {}/{}", choices.len(), e.key_page + 1, pages),
                    w * 0.08,
                    h * 0.32,
                    u32::MAX,
                    false,
                );
                if choices.is_empty() {
                    b.text(
                        a,
                        "No matches. Tap Search to change it.",
                        w * 0.08,
                        h * 0.45,
                        u32::MAX,
                        false,
                    );
                }
                for (i, (token, label)) in choices
                    .iter()
                    .skip(e.key_page * KEYS_PER_PAGE)
                    .take(KEYS_PER_PAGE)
                    .enumerate()
                {
                    plate(b, a, key_rect(i), label, e.chord_keys.contains(token));
                }
                plate(b, a, [0.08, 0.85, 0.24, 0.075], "Previous", false);
                plate(
                    b,
                    a,
                    [0.38, 0.85, 0.24, 0.075],
                    if e.chord_mode { "Apply keys" } else { "Back" },
                    false,
                );
                if e.chord_mode {
                    let names = e
                        .chord_keys
                        .iter()
                        .map(|key| super::custom::CustomButton::new(key.clone(), 0).label())
                        .collect::<Vec<_>>()
                        .join(" + ");
                    b.text(
                        a,
                        &format!("Selected: {}", names),
                        w * 0.08,
                        h * 0.94,
                        u32::MAX,
                        false,
                    );
                }
                plate(b, a, [0.68, 0.85, 0.24, 0.075], "Next", false);
            }
        }
    }
}
const KEYS_PER_PAGE: usize = 16;
const SEARCH_CHARS: &str = "1234567890qwertyuiopasdfghjklzxcvbnm .-";
fn key_rect(i: usize) -> [f32; 4] {
    [
        0.08 + (i % 4) as f32 * 0.215,
        0.36 + (i / 4) as f32 * 0.116,
        0.195,
        0.095,
    ]
}
fn search_key_rect(i: usize) -> [f32; 4] {
    [
        0.08 + (i % 10) as f32 * 0.085,
        0.38 + (i / 10) as f32 * 0.11,
        0.075,
        0.09,
    ]
}
fn picker_choices(raw_keys: bool, search: &str) -> Vec<(String, String)> {
    let query = search.to_lowercase().replace(['.', '-', '_'], " ");
    super::custom::choices(raw_keys)
        .into_iter()
        .filter(|(token, label)| {
            let haystack = format!("{} {}", token, label)
                .to_lowercase()
                .replace(['.', '-', '_'], " ");
            query.split_whitespace().all(|word| haystack.contains(word))
        })
        .collect()
}

#[cfg(test)]
mod picker_tests {
    use super::*;
    #[test]
    fn all_keys_still_available_and_search_handles_tokens() {
        assert_eq!(
            picker_choices(true, "").len(),
            crate::keybinds::available_keys().len() + 5
        );
        assert!(
            picker_choices(true, "left shift")
                .iter()
                .any(|(t, _)| t == "key.keyboard.left.shift")
        );
        assert!(
            picker_choices(true, "F5")
                .iter()
                .any(|(t, _)| t == "key.keyboard.f5")
        );
        assert!(picker_choices(true, "nothing matches this").is_empty());
    }
    #[test]
    fn key_tiles_and_search_keys_stay_inside_panel() {
        for r in (0..KEYS_PER_PAGE)
            .map(key_rect)
            .chain((0..SEARCH_CHARS.len()).map(search_key_rect))
        {
            assert!(r[0] >= 0.05 && r[0] + r[2] < 0.95);
            assert!(r[1] >= 0.14 && r[1] + r[3] < 0.84);
        }
    }
}
