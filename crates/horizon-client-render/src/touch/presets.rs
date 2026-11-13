//! Original preset loader for the three control layouts supplied by the user.
//! Preset data is normalized from their files; no launcher code or assets are bundled.
use super::{
    CustomButton, TouchControls,
    layout::{Mode, Placement, Settings},
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
    toggle: bool,
    swipe: bool,
}
pub const NAMES: [&str; 4] = ["My controls", "PvP", "Sword", "Mace"];
fn source_layout(name: &str, aspect: f32) -> Option<SavedLayout> {
    let presets: Vec<SourcePreset> = serde_json::from_str(include_str!("presets.json")).ok()?;
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
impl TouchControls {
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
            .or_else(|| source_layout(name, self.aspect()));
        let Some(selected) = selected else {
            return false;
        };
        self.clear();
        selected.apply(&mut self.settings);
        self.settings.active_layout = name.into();
        true
    }
    pub(super) fn reset_active_preset(&mut self) -> bool {
        let Some(selected) = source_layout(&self.settings.active_layout, self.aspect()) else {
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
