//! Original preset loader for the three control layouts supplied by the user.
//! Preset data is normalized from their files; no launcher code or assets are bundled.
use super::{
    CustomButton, TouchControls,
    layout::{Control, Mode, Placement, Settings},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedLayout {
    buttons: Vec<CustomButton>,
    visibility: [u8; 14],
    layouts: [[Placement; 14]; 3],
    mode: Mode,
    scale: f32,
    left_handed: bool,
}
impl SavedLayout {
    fn capture(settings: &Settings) -> Self {
        Self {
            buttons: settings.custom_buttons.clone(),
            visibility: settings.button_visibility,
            layouts: settings.layouts,
            mode: settings.mode,
            scale: settings.button_scale,
            left_handed: settings.left_handed,
        }
    }
    fn apply(self, settings: &mut Settings) {
        settings.custom_buttons = self.buttons;
        settings.button_visibility = self.visibility;
        settings.layouts = self.layouts;
        settings.mode = self.mode;
        settings.button_scale = self.scale;
        settings.left_handed = self.left_handed;
        settings.sanitize();
    }
}
#[derive(Deserialize)]
struct SourcePreset {
    name: String,
    buttons: Vec<SourceButton>,
}
#[derive(Deserialize)]
struct SourceButton {
    key: String,
    label: String,
    anchor: [f32; 2],
    size: [f32; 2],
    opacity: f32,
    fill_color: u32,
    pressed_fill_color: u32,
    text_color: u32,
    decoration: super::custom::ButtonDecoration,
    toggle: bool,
    swipe: bool,
}
pub const MINE_MACE: &str = "My controls + Mace";
pub const MINE_CRYSTAL: &str = "My controls + Crystal";
pub const NAMES: [&str; 6] = [
    "My controls",
    "PvP",
    "Sword",
    "Mace",
    MINE_MACE,
    MINE_CRYSTAL,
];
/// Captions (from the Mace+/Crystal source styles) added to a copy of the
/// player's own layout. Buttons whose key already exists there are skipped.
const MACE_EXTRAS: [&str; 6] = ["Rocket R", "Mace B", "Wind 2", "Totem G", "HIT", "Use"];
const CRYSTAL_EXTRAS: [&str; 6] = [
    "Obsidian 2",
    "Crystal 3",
    "Hold totem 6",
    "Totem G",
    "CK K",
    "HIT",
];
fn extras_for(name: &str) -> Option<(&'static str, &'static [&'static str])> {
    match name {
        MINE_MACE => Some(("Mace+", &MACE_EXTRAS)),
        MINE_CRYSTAL => Some(("Crystal", &CRYSTAL_EXTRAS)),
        _ => None,
    }
}
fn source_layout(name: &str, aspect: f32) -> Option<SavedLayout> {
    let mut presets: Vec<SourcePreset> = serde_json::from_str(include_str!("presets.json")).ok()?;
    let combat: Vec<SourcePreset> =
        serde_json::from_str(include_str!("combat_presets.json")).ok()?;
    presets.extend(combat);
    let source = presets.into_iter().find(|p| p.name == name)?;
    let aspect = aspect.max(0.1);
    let buttons = source
        .buttons
        .into_iter()
        .enumerate()
        .map(|(i, b)| {
            let mut button = CustomButton::new(b.key, i);
            button.caption = b.label;
            button.toggle = b.toggle;
            button.swipe = b.swipe;
            button.fill_color = Some(b.fill_color);
            button.pressed_fill_color = Some(b.pressed_fill_color);
            button.text_color = Some(b.text_color);
            button.decoration = Some(b.decoration);
            button.shape = b.size[0] / b.size[1].max(0.01);
            let width = (b.size[0] / aspect).min(1.0);
            let height = b.size[1].min(1.0);
            let mut place = Placement::new(
                width / 2.0 + b.anchor[0] * (1.0 - width),
                height / 2.0 + b.anchor[1] * (1.0 - height),
                height,
            );
            place.opacity = b.opacity;
            button.layouts = [place; 3];
            button
        })
        .collect();
    let defaults = Settings::default();
    Some(SavedLayout {
        buttons,
        visibility: [2; 14],
        layouts: defaults.layouts,
        mode: Mode::Crosshair,
        scale: 1.0,
        left_handed: false,
    })
}
/// Copy of the player's own layout with the extra keys added in free spots.
/// Nothing already in the layout is moved, resized or removed.
fn derived_layout(name: &str, mine: SavedLayout, aspect: f32) -> Option<SavedLayout> {
    let (style, extras) = extras_for(name)?;
    let aspect = aspect.max(0.1);
    let source = source_layout(style, aspect)?;
    let mode = mine.mode.index();
    let scale = mine.scale.max(0.1);
    let mut out = mine;
    // Occupied rectangles as [x0, y0, x1, y1] in screen fractions.
    let mut taken: Vec<[f32; 4]> = Vec::new();
    let left = out.left_handed;
    let fx = |x: f32| if left { 1.0 - x } else { x };
    for (i, visible) in out.visibility.iter().enumerate() {
        // 1 = forced on, 0 = automatic (shown), 2 = hidden by the player.
        if *visible == 2 {
            continue;
        }
        let p = out.layouts[mode][i];
        let half_h = p.size / 2.0;
        let half_w = p.size / aspect / 2.0;
        let cx = fx(p.center[0]);
        taken.push([
            cx - half_w,
            p.center[1] - half_h,
            cx + half_w,
            p.center[1] + half_h,
        ]);
    }
    for b in &out.buttons {
        let p = b.layouts[mode];
        let h = (p.size * scale).min(1.0);
        let w = (h * b.shape / aspect).min(1.0);
        let cx = fx(p.center[0]);
        taken.push([
            cx - w / 2.0,
            p.center[1] - h / 2.0,
            cx + w / 2.0,
            p.center[1] + h / 2.0,
        ]);
    }
    let gap = 0.006;
    let free = |r: [f32; 4], taken: &[[f32; 4]]| {
        r[0] >= 0.01
            && r[1] >= 0.01
            && r[2] <= 0.99
            && r[3] <= 0.99
            && taken.iter().all(|t| {
                r[2] + gap <= t[0] || t[2] + gap <= r[0] || r[3] + gap <= t[1] || t[3] + gap <= r[1]
            })
    };
    let have = |out: &SavedLayout, key: &str| out.buttons.iter().any(|b| b.key == key);
    for caption in extras {
        let Some(src) = source.buttons.iter().find(|b| b.caption == *caption) else {
            continue;
        };
        if out.buttons.len() >= 64 || have(&out, &src.key) {
            continue;
        }
        // Built-in attack/use buttons already do the same job as HIT / Use.
        let builtin = |c: Control| match out.visibility[c as usize] {
            1 => true,
            2 => false,
            _ => out.mode == Mode::Crosshair,
        };
        if (src.key == "key.mouse.left" && builtin(Control::Attack))
            || (src.key == "key.mouse.right" && builtin(Control::Use))
        {
            continue;
        }
        let want = src.layouts[0];
        let want_x = fx(want.center[0]);
        let mut size = want.size;
        let mut chosen = None;
        'shrink: for _ in 0..4 {
            let h = size;
            let w = h * src.shape / aspect;
            let mut best: Option<(f32, [f32; 2])> = None;
            let mut y = 0.06;
            while y < 0.95 {
                let mut x = 0.05;
                while x < 0.96 {
                    let r = [x - w / 2.0, y - h / 2.0, x + w / 2.0, y + h / 2.0];
                    // Keep the middle of the screen free for looking around.
                    let middle = x > 0.3 && x < 0.7 && y > 0.22 && y < 0.8;
                    if !middle && free(r, &taken) {
                        let d = (x - want_x).powi(2) + (y - want.center[1]).powi(2);
                        if best.is_none_or(|(bd, _)| d < bd) {
                            best = Some((d, [x, y]));
                        }
                    }
                    x += 0.01;
                }
                y += 0.01;
            }
            if let Some((_, c)) = best {
                chosen = Some((c, size));
                break 'shrink;
            }
            size *= 0.85;
        }
        let (center, size) = chosen.unwrap_or(([want_x, want.center[1]], want.size));
        let stored_x = fx(center[0]);
        let mut button = src.clone();
        let mut place = Placement::new(stored_x, center[1], size);
        place.opacity = want.opacity;
        button.layouts = [place; 3];
        let h = size;
        let w = h * button.shape / aspect;
        taken.push([
            center[0] - w / 2.0,
            center[1] - h / 2.0,
            center[0] + w / 2.0,
            center[1] + h / 2.0,
        ]);
        out.buttons.push(button);
    }
    Some(out)
}
impl TouchControls {
    fn mine_copy(&self, name: &str) -> Option<SavedLayout> {
        let mine = self
            .settings
            .named_layouts
            .get("My controls")
            .cloned()
            .unwrap_or_else(|| SavedLayout::capture(&self.settings));
        derived_layout(name, mine, self.aspect())
    }
    /// Switching captures the current editable layout first. Cancel restores the
    /// entire pre-editor settings, including all preset slots.
    pub(super) fn choose_preset(&mut self, name: &str) -> bool {
        if !NAMES.contains(&name) {
            return false;
        }
        let current = self.settings.active_layout.clone();
        let saved = SavedLayout::capture(&self.settings);
        self.settings.named_layouts.insert(current, saved);
        let selected = self
            .settings
            .named_layouts
            .get(name)
            .cloned()
            .or_else(|| source_layout(name, self.aspect()))
            .or_else(|| self.mine_copy(name));
        let Some(selected) = selected else {
            return false;
        };
        self.clear();
        selected.apply(&mut self.settings);
        self.settings.active_layout = name.into();
        true
    }
    pub(super) fn reset_active_preset(&mut self) -> bool {
        let active = self.settings.active_layout.clone();
        let Some(selected) =
            source_layout(&active, self.aspect()).or_else(|| self.mine_copy(&active))
        else {
            return false;
        };
        self.clear();
        selected.apply(&mut self.settings);
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preset_counts_tokens_and_bounds_match_user_files() {
        for (name, count) in [("PvP", 38), ("Sword", 37), ("Mace", 39)] {
            for aspect in [16.0 / 9.0, 20.0 / 9.0, 9.0 / 16.0] {
                let p = source_layout(name, aspect).unwrap();
                assert_eq!(p.buttons.len(), count);
                assert_eq!(p.buttons.iter().filter(|b| b.toggle).count(), 1);
                for b in p.buttons {
                    assert!(CustomButton::valid_token(&b.key), "{}", b.key);
                    let place = b.layouts[0];
                    let width = (place.size * b.shape / aspect).min(1.0);
                    assert!(place.center[0] - width / 2.0 >= -0.001);
                    assert!(place.center[0] + width / 2.0 <= 1.001);
                }
            }
        }
    }
    #[test]
    fn combat_layouts_use_existing_manual_actions_and_preserve_edits() {
        for name in ["Mace+", "Crystal"] {
            let layout = source_layout(name, 20.0 / 9.0).unwrap();
            assert!(layout.buttons.len() <= 64);
            assert!(
                layout
                    .buttons
                    .iter()
                    .all(|b| CustomButton::valid_token(&b.key))
            );
            assert!(layout.buttons.iter().any(|b| b.key == "key.keyboard.g"));
        }
        let crystal = source_layout("Crystal", 16.0 / 9.0).unwrap();
        assert!(
            crystal.buttons.iter().any(
                |b| b.caption == "Crystal 3" && b.key == "chord:key.keyboard.3+key.mouse.right"
            )
        );
        assert!(
            crystal
                .buttons
                .iter()
                .any(|b| b.caption == "HIT" && b.key == "key.mouse.left")
        );
        let mut t = TouchControls::default();
        assert!(t.choose_preset("Sword"));
        t.settings.custom_buttons[0].caption = "Custom inventory".into();
        assert!(t.choose_preset("Mace"));
        assert!(t.choose_preset("Sword"));
        assert_eq!(t.settings.custom_buttons[0].caption, "Custom inventory");
    }
    #[test]
    fn my_controls_copies_keep_layout_and_add_keys_in_free_spots() {
        for aspect in [16.0 / 9.0, 20.0 / 9.0] {
            for left in [false, true] {
                for (name, wanted) in [
                    (MINE_MACE, vec!["key.keyboard.r", "key.keyboard.b", "chord:key.keyboard.2+key.mouse.right"]),
                    (
                        MINE_CRYSTAL,
                        vec!["chord:key.keyboard.2+key.mouse.right", "chord:key.keyboard.3+key.mouse.right", "key.keyboard.6", "key.keyboard.k"],
                    ),
                ] {
                    let mut t = TouchControls::default();
                    t.viewport((aspect * 1000.0) as u32, 1000);
                    t.settings.left_handed = left;
                    let mut z = CustomButton::new("key.keyboard.z".into(), 0);
                    z.caption = "Mine Z".into();
                    t.settings.custom_buttons.push(z);
                    let before: Vec<_> = t
                        .settings
                        .custom_buttons
                        .iter()
                        .map(|b| (b.key.clone(), b.layouts[0].center))
                        .collect();
                    let builtin_before = t.settings.layouts;
                    assert!(t.choose_preset(name));
                    assert_eq!(t.settings.active_layout, name);
                    for (i, (k, c)) in before.iter().enumerate() {
                        assert_eq!(&t.settings.custom_buttons[i].key, k);
                        assert_eq!(&t.settings.custom_buttons[i].layouts[0].center, c);
                    }
                    assert_eq!(t.settings.layouts[0][0].center, builtin_before[0][0].center);
                    for w in &wanted {
                        assert!(
                            t.settings.custom_buttons.iter().any(|b| b.key == *w),
                            "{name} {w}"
                        );
                    }
                    // No added button overlaps another button.
                    let rect = |b: &CustomButton| {
                        let p = b.layouts[0];
                        let w = p.size * b.shape / aspect;
                        let cx = if left { 1.0 - p.center[0] } else { p.center[0] };
                        [
                            cx - w / 2.0,
                            p.center[1] - p.size / 2.0,
                            cx + w / 2.0,
                            p.center[1] + p.size / 2.0,
                        ]
                    };
                    let all = &t.settings.custom_buttons;
                    for i in 1..all.len() {
                        for j in 0..i {
                            let (a, b) = (rect(&all[i]), rect(&all[j]));
                            let apart =
                                a[2] <= b[0] || b[2] <= a[0] || a[3] <= b[1] || b[3] <= a[1];
                            assert!(
                                apart,
                                "{name}: {} overlaps {}",
                                all[i].caption, all[j].caption
                            );
                        }
                    }
                    // Going back to My controls shows the original, untouched.
                    assert!(t.choose_preset("My controls"));
                    assert_eq!(t.settings.custom_buttons.len(), before.len());
                    assert!(t.choose_preset(name));
                    assert!(t.settings.custom_buttons.len() > before.len());
                }
            }
        }
    }
    #[test]
    fn my_controls_copies_do_not_duplicate_existing_keys() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.g".into(), 0));
        let before = t
            .settings
            .custom_buttons
            .iter()
            .filter(|b| b.key == "key.keyboard.g")
            .count();
        assert!(t.choose_preset(MINE_CRYSTAL));
        let g = t
            .settings
            .custom_buttons
            .iter()
            .filter(|b| b.key == "key.keyboard.g")
            .count();
        assert_eq!(g, before);
        assert!(!NAMES.contains(&"Mace+") && !NAMES.contains(&"Crystal"));
    }
    #[test]
    fn combat_action_buttons_do_not_overlap_in_landscape() {
        for aspect in [16.0 / 9.0, 20.0 / 9.0] {
            for name in ["Mace+", "Crystal"] {
                let layout = source_layout(name, aspect).unwrap();
                for (i, a) in layout.buttons.iter().enumerate() {
                    let pa = a.layouts[0];
                    let wa = pa.size * a.shape / aspect;
                    for b in &layout.buttons[i + 1..] {
                        let pb = b.layouts[0];
                        let wb = pb.size * b.shape / aspect;
                        let dx = (pa.center[0] - pb.center[0]).abs();
                        let dy = (pa.center[1] - pb.center[1]).abs();
                        assert!(
                            dx + 0.001 >= (wa + wb) / 2.0
                                || dy + 0.001 >= (pa.size + pb.size) / 2.0,
                            "{name}: {} overlaps {}",
                            a.caption,
                            b.caption
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn switching_preserves_previous_edits_and_cancel_snapshot() {
        let mut t = TouchControls::default();
        t.settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.z".into(), 0));
        assert!(t.choose_preset("Sword"));
        t.settings.custom_buttons[0].caption = "My sword keyboard".into();
        assert!(t.choose_preset("Mace"));
        assert!(t.choose_preset("Sword"));
        assert_eq!(t.settings.custom_buttons[0].caption, "My sword keyboard");
        assert!(t.choose_preset("My controls"));
        assert_eq!(t.settings.custom_buttons[0].key, "key.keyboard.z");
    }
}
