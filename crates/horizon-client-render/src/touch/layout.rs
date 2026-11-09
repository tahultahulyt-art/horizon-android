//! Per-profile touch layouts. Coordinates are normalized; sizes are fractions
//! of surface height, so a button stays square on wide phones and tablets.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    #[default]
    Crosshair,
    Tap,
    Dpad,
}
impl Mode {
    pub const ALL: [Self; 3] = [Self::Crosshair, Self::Tap, Self::Dpad];
    pub fn label(self) -> &'static str {
        match self {
            Self::Crosshair => "Joystick & aim crosshair",
            Self::Tap => "Joystick & tap to interact",
            Self::Dpad => "D-Pad & tap to interact",
        }
    }
    pub const SCHEMES: [(Self, bool, &str); 4] = [
        (Self::Crosshair, false, "Joystick & aim crosshair"),
        (Self::Tap, false, "Joystick & tap to interact"),
        (Self::Dpad, false, "D-Pad & tap to interact"),
        (Self::Dpad, true, "D-Pad & aim crosshair"),
    ];
    pub fn index(self) -> usize {
        match self {
            Self::Crosshair => 0,
            Self::Tap => 1,
            Self::Dpad => 2,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Move,
    Sprint,
    Jump,
    Sneak,
    Attack,
    Use,
    Chat,
    Pause,
    Settings,
    Inventory,
    Debug,
    Perspective,
    Offhand,
    Shield,
}
impl Control {
    pub const ALL: [Self; 14] = [
        Self::Move,
        Self::Sprint,
        Self::Jump,
        Self::Sneak,
        Self::Attack,
        Self::Use,
        Self::Chat,
        Self::Pause,
        Self::Settings,
        Self::Inventory,
        Self::Debug,
        Self::Perspective,
        Self::Offhand,
        Self::Shield,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Move => "Movement",
            Self::Sprint => "Sprint",
            Self::Jump => "Jump / fly up",
            Self::Sneak => "Sneak / fly down",
            Self::Attack => "Attack / mine",
            Self::Use => "Use / place",
            Self::Chat => "Chat",
            Self::Pause => "Pause",
            Self::Settings => "Touch settings",
            Self::Inventory => "Inventory",
            Self::Debug => "Debug overlay (F3)",
            Self::Perspective => "Camera perspective (F5)",
            Self::Offhand => "Swap hands",
            Self::Shield => "Raise shield",
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Placement {
    pub center: [f32; 2],
    pub size: f32,
    pub opacity: f32,
    /// Width as a multiple of the height (`size`); 1.0 is a square button.
    #[serde(default = "one")]
    pub wide: f32,
}
fn one() -> f32 {
    1.0
}
impl Placement {
    pub(super) fn new(x: f32, y: f32, size: f32) -> Self {
        Self {
            center: [x, y],
            size,
            opacity: 0.65,
            wide: 1.0,
        }
    }
    pub fn rect(self, aspect: f32) -> [f32; 4] {
        let sx = self.size * self.wide / aspect.max(0.1);
        [
            self.center[0] - sx / 2.0,
            self.center[1] - self.size / 2.0,
            sx,
            self.size,
        ]
    }
    pub fn sanitize(&mut self) {
        if !self.size.is_finite() {
            self.size = 0.15;
        }
        if !self.opacity.is_finite() {
            self.opacity = 0.65;
        }
        self.wide = if self.wide.is_finite() {
            self.wide.clamp(0.4, 4.0)
        } else {
            1.0
        };
        self.size = self.size.clamp(0.045, 0.45);
        self.opacity = self.opacity.clamp(0.0, 1.0);
        for c in &mut self.center {
            *c = if c.is_finite() {
                c.clamp(0.025, 0.975)
            } else {
                0.5
            };
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: Mode,
    pub split_controls: bool,
    pub contextual_interact: bool,
    pub left_handed: bool,
    pub sprint_with_joystick: bool,
    pub swap_jump_sneak: bool,
    pub sensitivity: f32,
    /// Smooth real finger input, never target-aware.
    pub aim_stabilizer: bool,
    pub aim_assist: bool,
    pub aim_assist_strength: f32,
    pub sensitivity_horizontal: f32,
    pub sensitivity_vertical: f32,
    pub offhand_button: bool,
    pub shield_button: bool,
    pub circle_size: f32,
    pub button_scale: f32,
    /// Multiplies per-button opacity without overwriting custom layouts.
    pub control_opacity: f32,
    /// Game-drawn virtual/controller menu arrow only, not the OS pointer.
    pub cursor_scale: f32,
    pub custom_buttons: Vec<super::CustomButton>,
    pub analog_movement: bool,
    pub sprint_button: bool,
    pub sprint_toggle: bool,
    pub inventory_controls: bool,
    pub virtual_mouse_sensitivity: f32,
    pub crosshair_gestures: bool,
    pub attack_button_look: bool,
    pub gyroscope: bool,
    pub gyro_sensitivity: f32,
    pub attack_indicator_left: bool,
    pub dynamic_joystick: bool,
    /// Fraction of the width, from the movement edge inward, where a dynamic
    /// joystick may start.
    pub dynamic_area_x: f32,
    /// Fraction of the height the dynamic area spans, centred on the movement
    /// control (1.0 is the full surface).
    pub dynamic_area_y: f32,
    #[serde(deserialize_with = "deserialize_visibility")]
    pub button_visibility: [u8; 14],
    pub horizontal_margin: f32,
    pub vertical_margin: f32,
    /// 0 for profiles made before the flat boxed layout; it is applied once.
    #[serde(default)]
    pub layout_rev: u32,
    #[serde(deserialize_with = "deserialize_layouts")]
    pub layouts: [[Placement; 14]; 3],
}
// Append new buttons without resetting a player's existing ten-button layout.
fn deserialize_layouts<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<[[Placement; 14]; 3], D::Error> {
    let rows = Vec::<Vec<Placement>>::deserialize(deserializer)?;
    if rows.len() != 3 || rows.iter().any(|r| !matches!(r.len(), 10 | 12 | 14)) {
        return Err(serde::de::Error::custom(
            "expected three touch layouts of 10, 12 or 14 controls",
        ));
    }
    let mut layouts = Settings::default().layouts;
    for (output, input) in layouts.iter_mut().zip(rows) {
        output[..input.len()].copy_from_slice(&input);
    }
    Ok(layouts)
}
fn deserialize_visibility<'de, D: serde::Deserializer<'de>>(d: D) -> Result<[u8; 14], D::Error> {
    let old = Vec::<u8>::deserialize(d)?;
    if !matches!(old.len(), 12 | 14) {
        return Err(serde::de::Error::custom(
            "expected 12 or 14 visibility values",
        ));
    }
    let mut values = [0; 14];
    values[..old.len()].copy_from_slice(&old);
    Ok(values)
}
impl Default for Settings {
    fn default() -> Self {
        let base = [
            Placement::new(0.13, 0.76, 0.32),
            Placement::new(0.72, 0.78, 0.105),
            Placement::new(0.92, 0.78, 0.16),
            Placement::new(0.81, 0.86, 0.115),
            Placement::new(0.80, 0.64, 0.16),
            Placement::new(0.92, 0.52, 0.16),
            Placement::new(0.46, 0.055, 0.075),
            Placement::new(0.50, 0.055, 0.075),
            Placement::new(0.54, 0.055, 0.075),
            Placement::new(0.73, 0.95, 0.075),
            Placement::new(0.42, 0.055, 0.075),
            Placement::new(0.58, 0.055, 0.075),
            Placement::new(0.80, 0.96, 0.075),
            Placement::new(0.92, 0.36, 0.12),
        ];
        let mut layouts = [base; 3];
        layouts[2][Control::Sneak as usize] = Placement::new(0.13, 0.76, 0.10);
        layouts[2][Control::Jump as usize] = Placement::new(0.90, 0.78, 0.14);
        Self {
            mode: Mode::Crosshair,
            split_controls: false,
            contextual_interact: false,
            left_handed: false,
            sprint_with_joystick: false,
            swap_jump_sneak: false,
            sensitivity: 1.0,
            aim_stabilizer: false,
            aim_assist: false,
            aim_assist_strength: 0.15,
            sensitivity_horizontal: 1.0,
            sensitivity_vertical: 1.0,
            offhand_button: false,
            shield_button: true,
            circle_size: 32.0,
            button_scale: 1.0,
            control_opacity: 1.0,
            cursor_scale: 1.0,
            custom_buttons: Vec::new(),
            analog_movement: false,
            sprint_button: true,
            sprint_toggle: false,
            inventory_controls: false,
            virtual_mouse_sensitivity: 1.0,
            crosshair_gestures: false,
            attack_button_look: true,
            gyroscope: false,
            gyro_sensitivity: 1.0,
            attack_indicator_left: true,
            dynamic_joystick: false,
            dynamic_area_x: 0.25,
            dynamic_area_y: 1.0,
            button_visibility: [0; 14],
            horizontal_margin: 0.0,
            vertical_margin: 0.0,
            layout_rev: 1,
            layouts,
        }
    }
}
impl Settings {
    pub fn load() -> Self {
        Self::load_from(std::path::Path::new("horizon-touch.json"))
    }
    pub fn load_from(path: &std::path::Path) -> Self {
        let mut s: Self = std::fs::read(path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_else(|| Self {
                layout_rev: 0,
                ..Self::default()
            });
        s.sanitize();
        // Restore the original top-row spacing without moving custom layouts.
        for row in &mut s.layouts {
            for (control, old, new) in [
                (Control::Chat, 0.45, 0.46),
                (Control::Settings, 0.55, 0.54),
                (Control::Debug, 0.40, 0.42),
                (Control::Perspective, 0.60, 0.58),
            ] {
                let p = &mut row[control as usize];
                if p.center == [old, 0.055] && p.size == 0.075 {
                    p.center[0] = new;
                }
            }
        }
        // Migrate factory Swap placements only; keep the player's own layout.
        for row in &mut s.layouts {
            let swap = &mut row[Control::Offhand as usize];
            if matches!(swap.center, [0.68, 0.80] | [0.66, 0.38])
                && (swap.size - 0.09).abs() < 0.0001
            {
                swap.center = [0.80, 0.96];
                swap.size = 0.075;
            }
        }
        // Move the old factory sprint position only; keep customized placements.
        for mode in [Mode::Crosshair, Mode::Tap] {
            let sprint = &mut s.layouts[mode.index()][Control::Sprint as usize];
            if sprint.center == [0.13, 0.50] && (sprint.size - 0.105).abs() < 0.0001 {
                sprint.center = [0.72, 0.78];
            }
        }
        if s.layout_rev < 1 {
            s.apply_boxed_layout();
            s.layout_rev = 1;
            s.sanitize();
            let _ = s.save_to(path);
        }
        s
    }
    /// Flat boxed on-screen layout, positioned in a 1598x719 reference frame.
    fn apply_boxed_layout(&mut self) {
        fn at(x: f32, y: f32, w: f32, h: f32) -> Placement {
            let mut p = Placement::new((x + w / 2.0) / 1598.0, (y + h / 2.0) / 719.0, h / 719.0);
            p.wide = w / h;
            p.opacity = 0.85;
            p
        }
        let mut base = Self::default().layouts[0];
        let mut dpad = Placement::new(249.5 / 1598.0, 454.0 / 719.0, 0.45);
        dpad.opacity = 0.85;
        base[Control::Move as usize] = dpad;
        base[Control::Sprint as usize] = at(930.0, 333.0, 150.0, 90.0);
        base[Control::Jump as usize] = at(1470.0, 440.0, 128.0, 100.0);
        base[Control::Sneak as usize] = at(1470.0, 545.0, 128.0, 100.0);
        base[Control::Attack as usize] = at(1096.0, 333.0, 179.0, 240.0);
        base[Control::Use as usize] = at(1275.0, 333.0, 180.0, 240.0);
        base[Control::Chat as usize] = at(180.0, 67.0, 180.0, 66.0);
        base[Control::Pause as usize] = at(0.0, 0.0, 180.0, 67.0);
        base[Control::Settings as usize] = at(1481.0, 309.0, 100.0, 100.0);
        base[Control::Inventory as usize] = at(1096.0, 629.0, 90.0, 90.0);
        base[Control::Debug as usize] = at(180.0, 0.0, 180.0, 67.0);
        base[Control::Perspective as usize] = at(360.0, 0.0, 180.0, 67.0);
        base[Control::Offhand as usize] = at(930.0, 433.0, 150.0, 90.0);
        base[Control::Shield as usize] = at(930.0, 533.0, 150.0, 90.0);
        self.layouts = [base; 3];
        self.mode = Mode::Dpad;
        self.split_controls = true;
        self.button_visibility[Control::Sprint as usize] = 1;
        let keys = [
            ("key.keyboard.tab", "TAB", at(0.0, 67.0, 180.0, 66.0)),
            ("key.keyboard.left.control", "CTRL", at(81.0, 186.0, 180.0, 66.0)),
            ("key.keyboard.left.alt", "ALT", at(261.0, 186.0, 180.0, 66.0)),
        ];
        self.custom_buttons
            .retain(|b| !keys.iter().any(|(token, _, _)| b.key == *token));
        for (token, label, placement) in keys {
            let mut button = super::CustomButton::new(token.into(), 0);
            button.label = Some(label.into());
            button.layouts = [placement; 3];
            self.custom_buttons.push(button);
        }
    }
    pub fn sanitize(&mut self) {
        if self.mode == Mode::Tap && self.split_controls {
            self.mode = Mode::Crosshair;
            self.split_controls = false;
            self.crosshair_gestures = true;
        }
        self.control_opacity = if self.control_opacity.is_finite() {
            self.control_opacity.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.cursor_scale = if self.cursor_scale.is_finite() {
            self.cursor_scale.clamp(0.5, 3.0)
        } else {
            1.0
        };
        self.virtual_mouse_sensitivity = if self.virtual_mouse_sensitivity.is_finite() {
            self.virtual_mouse_sensitivity.clamp(0.25, 3.0)
        } else {
            1.0
        };
        self.aim_assist_strength = if self.aim_assist_strength.is_finite() {
            self.aim_assist_strength.clamp(0.0, 0.3)
        } else {
            0.15
        };
        self.gyro_sensitivity = if self.gyro_sensitivity.is_finite() {
            self.gyro_sensitivity.clamp(0.1, 3.0)
        } else {
            1.0
        };
        for value in [&mut self.horizontal_margin, &mut self.vertical_margin] {
            *value = if value.is_finite() {
                value.clamp(0.0, 0.12)
            } else {
                0.0
            };
        }
        if !self.sensitivity.is_finite() {
            self.sensitivity = 1.0;
        }
        self.sensitivity = self.sensitivity.clamp(0.1, 10.0);
        for value in [
            &mut self.sensitivity_horizontal,
            &mut self.sensitivity_vertical,
        ] {
            *value = if value.is_finite() {
                value.clamp(0.1, 10.0)
            } else {
                1.0
            };
        }
        self.button_scale = if self.button_scale.is_finite() {
            self.button_scale.clamp(0.5, 2.0)
        } else {
            1.0
        };
        self.dynamic_area_x = if self.dynamic_area_x.is_finite() {
            self.dynamic_area_x.clamp(0.0, 1.0)
        } else {
            0.25
        };
        self.dynamic_area_y = if self.dynamic_area_y.is_finite() {
            self.dynamic_area_y.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.circle_size = if self.circle_size.is_finite() {
            self.circle_size.clamp(16.0, 64.0)
        } else {
            32.0
        };
        self.custom_buttons.truncate(32);
        self.custom_buttons
            .retain(|b| super::CustomButton::valid_token(&b.key));
        for button in &mut self.custom_buttons {
            for p in &mut button.layouts {
                p.sanitize();
            }
        }
        for p in self.layouts.iter_mut().flatten() {
            p.sanitize();
        }
    }
    pub fn draw_opacity(&self, opacity: f32, editing: bool, recovery: bool) -> f32 {
        if editing {
            opacity.max(0.25)
        } else {
            let value = opacity * self.control_opacity;
            if recovery { value.max(0.25) } else { value }
        }
    }
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(std::path::Path::new("horizon-touch.json"))
    }
    pub fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(temp, path)
    }
    pub fn crosshair(&self) -> bool {
        self.mode == Mode::Crosshair || self.split_controls
    }
    pub fn visible(&self, c: Control) -> bool {
        match self.button_visibility[c as usize] {
            1 => return true,
            2 => return false,
            _ => {}
        }
        match c {
            Control::Offhand => self.offhand_button,
            Control::Shield => self.shield_button && !self.visible(Control::Use),
            Control::Attack | Control::Use => self.crosshair(),
            Control::Sprint => {
                self.sprint_button && self.mode != Mode::Dpad && !self.sprint_with_joystick
            }
            _ => true,
        }
    }
    pub fn slot(&self, c: Control) -> usize {
        match c {
            Control::Jump if self.mode == Mode::Dpad && self.swap_jump_sneak => {
                Control::Sneak as usize
            }
            Control::Sneak if self.mode == Mode::Dpad && self.swap_jump_sneak => {
                Control::Jump as usize
            }
            _ => c as usize,
        }
    }
    pub fn placement(&self, c: Control) -> Placement {
        let mut p = self.layouts[self.mode.index()][self.slot(c)];
        if self.left_handed {
            p.center[0] = 1.0 - p.center[0];
        }
        p
    }
}
pub fn contains([x, y, w, h]: [f32; 4], p: [f32; 2]) -> bool {
    (x..=x + w).contains(&p[0]) && (y..=y + h).contains(&p[1])
}

#[cfg(test)]
mod tests {
    #[test]
    fn stabilizer_defaults_off_for_existing_profiles_and_roundtrips() {
        let mut settings: super::Settings = serde_json::from_str("{}").unwrap();
        assert!(!settings.aim_stabilizer);
        assert!(!settings.aim_assist);
        settings.aim_stabilizer = true;
        settings.aim_assist = true;
        let restored: super::Settings =
            serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
        assert!(restored.aim_stabilizer);
        assert!(restored.aim_assist);
        assert_eq!(restored.aim_assist_strength, 0.15);
        settings.aim_assist_strength = 10.0;
        settings.sanitize();
        assert_eq!(settings.aim_assist_strength, 0.3);
        settings.aim_assist_strength = f32::NAN;
        settings.sanitize();
        assert_eq!(settings.aim_assist_strength, 0.15);
        assert_eq!(restored.sensitivity, settings.sensitivity);
    }
    #[test]
    fn gyro_preferences_roundtrip_and_reject_bad_values() {
        let mut s = super::Settings::default();
        assert!(!s.gyroscope);
        s.gyroscope = true;
        s.gyro_sensitivity = 2.0;
        let restored: super::Settings =
            serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert!(restored.gyroscope);
        assert_eq!(restored.gyro_sensitivity, 2.0);
        s.gyro_sensitivity = f32::NAN;
        s.sanitize();
        assert_eq!(s.gyro_sensitivity, 1.0);
    }
    #[test]
    fn blank_buttons_and_game_actions_survive_settings_validation() {
        let mut settings = super::Settings::default();
        settings
            .custom_buttons
            .push(super::super::CustomButton::new(
                "key.keyboard.unknown".into(),
                0,
            ));
        settings
            .custom_buttons
            .push(super::super::CustomButton::new("action:key.jump".into(), 1));
        settings.sanitize();
        assert_eq!(settings.custom_buttons.len(), 2);
        let encoded = serde_json::to_string(&settings.custom_buttons[1]).unwrap();
        assert_eq!(
            serde_json::from_str::<super::super::CustomButton>(&encoded)
                .unwrap()
                .key,
            "action:key.jump"
        );
        assert!(!super::super::CustomButton::valid_token(
            "action:no-such-action"
        ));
        assert!(
            super::super::custom::choices(false)
                .iter()
                .any(|(token, _)| token == "action:key.attack")
        );
        assert!(
            super::super::custom::choices(true)
                .iter()
                .any(|(token, _)| token == "key.keyboard.z")
        );
    }

    use super::*;

    #[test]
    fn added_buttons_override_scheme_and_removed_buttons_stay_hidden() {
        let mut settings = Settings {
            mode: Mode::Tap,
            ..Settings::default()
        };
        assert!(!settings.visible(Control::Attack));
        settings.button_visibility[Control::Attack as usize] = 1;
        assert!(settings.visible(Control::Attack));
        settings.button_visibility[Control::Chat as usize] = 2;
        assert!(!settings.visible(Control::Chat));
        let restored: Settings =
            serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
        assert!(restored.visible(Control::Attack));
        assert!(!restored.visible(Control::Chat));
    }

    #[test]
    fn old_layout_preserves_custom_buttons_when_shortcuts_are_added() {
        let mut json = serde_json::to_value(Settings::default()).unwrap();
        json["circle_size"] = 48.into();
        for row in json["layouts"].as_array_mut().unwrap() {
            row.as_array_mut().unwrap().truncate(10);
            row[6]["center"] = serde_json::json!([0.25, 0.2]);
            row[6]["opacity"] = serde_json::json!(0.35);
        }
        let settings: Settings = serde_json::from_value(json).unwrap();
        assert_eq!(settings.circle_size, 48.0);
        for row in settings.layouts {
            assert_eq!(row[6].center, [0.25, 0.2]);
            assert_eq!(row[6].opacity, 0.35);
            assert_eq!(row[Control::Debug as usize].center, [0.42, 0.055]);
            assert_eq!(row[Control::Perspective as usize].center, [0.58, 0.055]);
        }
        let roundtrip: Settings =
            serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
        assert_eq!(roundtrip.layouts[0][6].center, [0.25, 0.2]);
    }
    #[test]
    fn swap_factory_migration_keeps_custom_positions() {
        let file =
            std::env::temp_dir().join(format!("horizon-swap-layout-{}.json", std::process::id()));
        let mut settings = Settings::default();
        settings.layouts[0][Control::Offhand as usize] = super::Placement::new(0.68, 0.80, 0.09);
        settings.layouts[1][Control::Offhand as usize].center = [0.55, 0.43];
        settings.layouts[2][Control::Offhand as usize] = super::Placement::new(0.66, 0.38, 0.09);
        settings.save_to(&file).unwrap();
        let read = Settings::load_from(&file);
        assert_eq!(
            read.layouts[0][Control::Offhand as usize].center,
            [0.80, 0.96]
        );
        assert_eq!(
            read.layouts[1][Control::Offhand as usize].center,
            [0.55, 0.43]
        );
        assert_eq!(
            read.layouts[2][Control::Offhand as usize].center,
            [0.80, 0.96]
        );
        std::fs::remove_file(file).unwrap();
    }
}

#[cfg(test)]
mod quick_controls_tests {
    use super::*;
    #[test]
    fn defaults_and_sanitization_keep_old_profiles_compatible() {
        let mut s: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!((s.control_opacity, s.cursor_scale), (1.0, 1.0));
        s.control_opacity = f32::NAN;
        s.cursor_scale = f32::INFINITY;
        s.sanitize();
        assert_eq!((s.control_opacity, s.cursor_scale), (1.0, 1.0));
        s.control_opacity = -9.0;
        s.cursor_scale = 9.0;
        s.sanitize();
        assert_eq!((s.control_opacity, s.cursor_scale), (0.0, 3.0));
        assert_eq!(s.draw_opacity(0.65, false, false), 0.0);
        assert_eq!(s.draw_opacity(0.0, false, true), 0.25);
        assert_eq!(s.draw_opacity(0.0, true, false), 0.25);
    }
    #[test]
    fn master_opacity_preserves_each_custom_placement() {
        let mut s = Settings::default();
        let before = serde_json::to_value(&s.layouts).unwrap();
        s.control_opacity = 0.5;
        assert_eq!(s.draw_opacity(0.6, false, false), 0.3);
        assert_eq!(serde_json::to_value(&s.layouts).unwrap(), before);
        let roundtrip: Settings =
            serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(roundtrip.control_opacity, 0.5);
    }
}
