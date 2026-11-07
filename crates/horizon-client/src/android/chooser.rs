//! Explicit, scrollable selections. Values never cycle on repeated button taps.
use super::{
    Launcher,
    pages::{Action, button},
};
use crate::catalog::RendererMode;
use horizon_client::ui_theme::{Face, Ui, color};

pub(super) enum Kind {
    Aspect,
    DisplayHz,
    CacheBudget,
    WorkerLimit,
    Renderer,
    RenderScale,
    Profile,
    Orientation,
    InputHost,
    TouchLook,
    QuickControls,
    TouchValue(TouchValue),
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum TouchValue {
    Opacity,
    Sensitivity,
    Stabilizer,
    Animations,
    Assist,
    AssistStrength,
    Mouse,
    Cursor,
    Buttons,
    Circle,
}
impl TouchValue {
    pub fn label(self) -> &'static str {
        match self {
            Self::Opacity => "Control opacity",
            Self::Sensitivity => "Touch sensitivity",
            Self::Stabilizer => "Shield Mode",
            Self::Animations => "Risiya Animations",
            Self::Assist => "Shield Mode: Target Stick",
            Self::AssistStrength => "Target Stick strength",
            Self::Mouse => "Virtual mouse sensitivity",
            Self::Cursor => "Virtual cursor size",
            Self::Buttons => "Control button size",
            Self::Circle => "Touch circle size",
        }
    }
    pub fn current(self, s: &horizon_client::touch::Settings) -> u32 {
        (match self {
            Self::Animations => s.risiya_animations as f32,
            Self::Opacity => s.control_opacity * 100.0,
            Self::Sensitivity => s.sensitivity * 100.0,
            Self::AssistStrength => s.aim_assist_strength * 100.0,
            Self::Assist => {
                if s.aim_assist {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Stabilizer => {
                if s.aim_stabilizer {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Mouse => s.virtual_mouse_sensitivity * 100.0,
            Self::Cursor => s.cursor_scale * 100.0,
            Self::Buttons => s.button_scale * 100.0,
            Self::Circle => s.circle_size,
        })
        .round() as u32
    }
    pub fn value_label(self, value: u32) -> String {
        if self == Self::Animations {
            return horizon_client::risiya_animation::LABELS[value.min(3) as usize].into();
        }
        if matches!(self, Self::Stabilizer | Self::Assist) {
            return if value == 0 { "Off" } else { "On" }.into();
        }
        format!("{value}{}", if self == Self::Circle { " px" } else { "%" })
    }
    pub fn presets(self) -> Vec<u32> {
        match self {
            Self::Animations => vec![0, 1, 2, 3],
            Self::Stabilizer | Self::Assist => vec![0, 1],
            Self::AssistStrength => vec![0, 5, 10, 15, 20, 25, 30],
            Self::Opacity => (0..=100).step_by(10).collect(),
            Self::Sensitivity => vec![10, 25, 50, 75, 100, 150, 200, 300, 400, 500, 600, 800, 1000],
            Self::Mouse => vec![25, 50, 75, 100, 125, 150, 175, 200, 250, 300],
            Self::Cursor => vec![50, 75, 100, 125, 150, 175, 200, 250, 300],
            Self::Buttons => vec![50, 75, 100, 125, 150, 175, 200],
            Self::Circle => vec![16, 24, 32, 40, 48, 56, 64],
        }
    }
    pub fn apply(self, s: &mut horizon_client::touch::Settings, value: u32) {
        let v = value as f32 / 100.0;
        match self {
            Self::Animations => s.risiya_animations = value.min(3) as u8,
            Self::Stabilizer => s.aim_stabilizer = value != 0,
            Self::Assist => s.aim_assist = value != 0,
            Self::AssistStrength => s.aim_assist_strength = v,
            Self::Opacity => s.control_opacity = v,
            Self::Sensitivity => s.sensitivity = v,
            Self::Mouse => s.virtual_mouse_sensitivity = v,
            Self::Cursor => s.cursor_scale = v,
            Self::Buttons => s.button_scale = v,
            Self::Circle => s.circle_size = value as f32,
        }
        s.sanitize();
    }
}

pub(super) struct Chooser {
    title: &'static str,
    help: &'static str,
    device: Option<String>,
    choices: Vec<(String, String, Action, bool)>,
    pub scroll: f32,
}
impl Launcher {
    pub(super) fn open_chooser(&mut self, kind: Kind) {
        crate::platform::hide();
        self.catalog.editing = None;
        let device = if matches!(kind, Kind::Renderer) {
            self.renderer.as_ref().map(|r| {
                format!(
                    "GPU: {} / {} {}",
                    r.info.name, r.info.driver, r.info.driver_info
                )
            })
        } else {
            None
        };
        let c = &self.catalog.config;
        let (title, help, choices) = match kind {
            Kind::Aspect => (
                "Aspect ratio",
                "Fits the entire picture without stretching. Touch coordinates follow the picture. Portrait swaps width and height.",
                [
                    crate::display::Aspect::Native,
                    crate::display::Aspect::Wide,
                    crate::display::Aspect::Classic,
                    crate::display::Aspect::Square,
                ]
                .into_iter()
                .map(|a| {
                    (
                        a.label().into(),
                        String::new(),
                        Action::ChooseAspect(a),
                        c.screen.aspect == a,
                    )
                })
                .collect(),
            ),
            Kind::CacheBudget => (
                "Reusable RAM cache",
                "Decoded sounds (80%) and empty mesh buffers (20%). Recreated at the same quality. Smaller budgets can increase CPU work. Not total RAM/VRAM. Applies on next Play.",
                [0, 16, 32, 64, 128, 256]
                    .into_iter()
                    .map(|mib| {
                        (
                            if mib == 0 {
                                "Engine default (80 MiB)".into()
                            } else {
                                format!("{mib} MiB")
                            },
                            String::new(),
                            Action::ChooseCacheBudget(mib),
                            c.cache_budget_mib == mib,
                        )
                    })
                    .collect(),
            ),
            Kind::WorkerLimit => (
                "Chunk worker limit",
                "Ceiling for generation and meshing workers, not all app threads. Lower values can reduce CPU contention but slow loading. Applies on next Play.",
                (0..=64u32)
                    .map(|limit| {
                        (
                            if limit == 0 {
                                "No additional limit".into()
                            } else {
                                format!("{limit} worker(s)")
                            },
                            String::new(),
                            Action::ChooseWorkerLimit(limit),
                            c.worker_limit == limit,
                        )
                    })
                    .collect(),
            ),
            Kind::DisplayHz => (
                "Display refresh rate",
                "An Android display preference, not a game FPS cap. Unsupported rates may be ignored.",
                crate::device_preferences::DISPLAY_RATES
                    .into_iter()
                    .map(|hz| {
                        (
                            if hz == 0 {
                                "System default".into()
                            } else {
                                format!("{hz} Hz")
                            },
                            String::new(),
                            Action::ChooseDisplayHz(hz),
                            c.device.display_hz == hz,
                        )
                    })
                    .collect(),
            ),
            Kind::RenderScale => {
                let mut choices: Vec<_> = crate::display::RenderScale::PRESETS
                    .into_iter()
                    .map(|percent| {
                        (
                            format!("{percent}%"),
                            if percent == 100 {
                                "Native resolution".into()
                            } else {
                                "Lower resolution for the whole picture".into()
                            },
                            Action::ChooseRenderScale(crate::display::RenderScale::new(percent)),
                            c.render_scale.percent() == percent,
                        )
                    })
                    .collect();
                choices.push((
                    "Custom percentage...".into(),
                    "Enter a value from 25 to 100".into(),
                    Action::Field("render_scale"),
                    false,
                ));
                (
                    "Client resolution",
                    "Small screens retain a usable menu size.",
                    choices,
                )
            }
            Kind::QuickControls => {
                let touch = horizon_client::touch::Settings::load_from(
                    &c.game_home(&self.home).join("horizon-touch.json"),
                );
                let mut choices: Vec<_> = [
                    TouchValue::Opacity,
                    TouchValue::Sensitivity,
                    TouchValue::Stabilizer,
                    TouchValue::Assist,
                    TouchValue::AssistStrength,
                    TouchValue::Mouse,
                    TouchValue::Cursor,
                    TouchValue::Buttons,
                    TouchValue::Circle,
                ]
                .into_iter()
                .map(|v| {
                    (
                        v.label().into(),
                        v.value_label(v.current(&touch)),
                        Action::TouchValue(v),
                        false,
                    )
                })
                .collect();
                if horizon_client::risiya_animation::available() {
                    let v = TouchValue::Animations;
                    choices.insert(
                        0,
                        (
                            v.label().into(),
                            v.value_label(v.current(&touch)),
                            Action::TouchValue(v),
                            false,
                        ),
                    );
                }
                choices.insert(
                    0,
                    (
                        "Edit controls".into(),
                        String::new(),
                        Action::EditControls,
                        false,
                    ),
                );
                ("Quick controls", "", choices)
            }
            Kind::TouchValue(value) => {
                let touch = horizon_client::touch::Settings::load_from(
                    &c.game_home(&self.home).join("horizon-touch.json"),
                );
                let current = value.current(&touch);
                let mut presets = value.presets();
                if !presets.contains(&current) {
                    presets.push(current);
                    presets.sort_unstable();
                }
                (
                    value.label(),
                    if value == TouchValue::Animations {
                        "Original cosmetic first-person motion. Off restores stock. Lite avoids idle sway; Full adds sway; Reduced uses gentler motion. Inspect cancels for attack/use. No gameplay changes. Applies on next Play."
                    } else if value == TouchValue::Stabilizer {
                        "Smooths small touch movements. Adds a small input delay. No target tracking, lock-on or auto-attack. Default off."
                    } else if matches!(value, TouchValue::Assist | TouchValue::AssistStrength) {
                        "Requires Shield Mode and crosshair controls. Small automatic corrections near visible other players, only while your finger moves. No attacks or full lock. Strength 0-30%. Default off. Follow server rules; aim assist may be prohibited."
                    } else {
                        ""
                    },
                    presets
                        .into_iter()
                        .map(|p| {
                            (
                                value.value_label(p),
                                String::new(),
                                Action::ChooseTouchValue(value, p),
                                p == current,
                            )
                        })
                        .collect(),
                )
            }
            Kind::TouchLook => {
                let current = horizon_client::touch::Settings::load_from(
                    &c.game_home(&self.home).join("horizon-touch.json"),
                )
                .sensitivity;
                let percent = (current * 100.0).round().clamp(10.0, 1000.0) as u32;
                (
                    "Touch look speed",
                    "Camera speed when dragging to look around. Applies on next Play. Fine tune in Options > Controls > Touch Controls; the in-game sliders now reach 1000%.",
                    [100u32, 200, 300, 500, 800, 1000]
                        .into_iter()
                        .map(|p| {
                            (
                                format!("{p}%"),
                                match p {
                                    100 => "Game default".into(),
                                    300 => "Previous maximum".into(),
                                    _ => String::new(),
                                },
                                Action::ChooseTouchLook(p),
                                percent == p,
                            )
                        })
                        .collect(),
                )
            }
            Kind::InputHost => (
                "Input mode",
                "Mouse mode uses a physical keyboard. Touch mode keeps the Android keyboard.",
                crate::catalog::InputHost::ALL
                    .into_iter()
                    .map(|mode| {
                        (
                            mode.label().into(),
                            String::new(),
                            Action::ChooseInputHost(mode),
                            c.input_host == mode,
                        )
                    })
                    .collect(),
            ),
            Kind::Orientation => (
                "Screen orientation",
                "Applies to the launcher and game. Does not change touch bindings.",
                crate::catalog::Orientation::ALL
                    .into_iter()
                    .map(|mode| {
                        (
                            mode.label().into(),
                            String::new(),
                            Action::ChooseOrientation(mode),
                            c.orientation == mode,
                        )
                    })
                    .collect(),
            ),
            Kind::Renderer => (
                "Rendering mode",
                "Uses the system GPU driver. Applies on the next game launch.",
                vec![
                    (
                        "OpenGL ES / experimental".into(),
                        "For older GPUs. Requires ES 3.0+ and sufficient driver limits.".into(),
                        Action::ChooseRenderer(RendererMode::OpenGl),
                        c.renderer == RendererMode::OpenGl,
                    ),
                    (
                        "Vulkan / GPU batching".into(),
                        "Groups terrain draws and enables supported GPU culling.".into(),
                        Action::ChooseRenderer(RendererMode::Vulkan),
                        c.renderer == RendererMode::Vulkan,
                    ),
                    (
                        "Vulkan / compatibility".into(),
                        "Direct terrain draws for drivers with missing or transparent blocks."
                            .into(),
                        Action::ChooseRenderer(RendererMode::VulkanCompatibility),
                        c.renderer == RendererMode::VulkanCompatibility,
                    ),
                ],
            ),
            Kind::Profile => {
                let mut ids = vec![0, c.profile];
                ids.extend(
                    std::fs::read_dir(self.home.join("profiles"))
                        .into_iter()
                        .flatten()
                        .flatten()
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| {
                            e.file_name()
                                .to_str()?
                                .strip_prefix("profile-")?
                                .parse::<u32>()
                                .ok()
                        }),
                );
                ids.sort_unstable();
                ids.dedup();
                let next = ids.last().copied().unwrap_or(0).checked_add(1);
                let mut choices: Vec<_> = ids
                    .into_iter()
                    .map(|id| {
                        (
                            format!("Profile {id}"),
                            if id == 0 {
                                "Default profile".into()
                            } else {
                                "Separate saves, mods and packs".into()
                            },
                            Action::ChooseProfile(id),
                            id == c.profile,
                        )
                    })
                    .collect();
                if let Some(id) = next {
                    choices.push((
                        "Create profile".into(),
                        format!("Create Profile {id}"),
                        Action::ChooseProfile(id),
                        false,
                    ));
                }
                (
                    "Choose profile",
                    "Each profile keeps its own game settings and saves.",
                    choices,
                )
            }
        };
        self.chooser = Some(Chooser {
            title,
            help,
            choices,
            device,
            scroll: 0.0,
        });
    }
}
impl Chooser {
    /// Free-form option list (file manager row actions, delete confirmations).
    pub(super) fn custom(
        title: &'static str,
        help: &'static str,
        detail: Option<String>,
        choices: Vec<(String, String, Action, bool)>,
    ) -> Self {
        Self {
            title,
            help,
            device: detail,
            choices,
            scroll: 0.0,
        }
    }
    pub(super) fn confirm_delete(name: String) -> Self {
        Self {
            title: "Remove add-on?",
            help: "Deletes only the selected file from this profile. This cannot be undone.",
            device: Some(name),
            choices: vec![
                (
                    "Remove".into(),
                    String::new(),
                    Action::ConfirmDeleteLocal,
                    false,
                ),
                ("Cancel".into(), String::new(), Action::Dismiss, true),
            ],
            scroll: 0.0,
        }
    }
    pub(super) fn navigation(&self, width: f32) -> Vec<([f32; 4], Action)> {
        self.choices
            .iter()
            .enumerate()
            .map(|(i, (_, _, action, _))| {
                (
                    [
                        32.0,
                        137.0 + i as f32 * 84.0 - self.scroll,
                        width - 64.0,
                        76.0,
                    ],
                    *action,
                )
            })
            .collect()
    }
    pub(super) fn viewport(&self, height: f32) -> [f32; 3] {
        [
            137.0,
            height - 100.0,
            (self.choices.len() as f32 * 84.0 - (height - 237.0)).max(0.0),
        ]
    }
    pub fn draw(&mut self, ui: &mut Ui, targets: &mut Vec<([f32; 4], Action)>) {
        let (w, h) = (ui.w, ui.h);
        targets.clear();
        ui.rounded(0.0, 0.0, w, h, 0.0, color::BG);
        ui.text(Face::Medium, 28.0, 32.0, 35.0, color::TEXT, self.title);
        ui.text(Face::Regular, 15.0, 32.0, 85.0, color::MUTED, self.help);
        if let Some(device) = &self.device {
            ui.text(Face::Regular, 13.0, 32.0, 111.0, color::MUTED, device);
        }
        let top = 137.0;
        let bottom = h - 100.0;
        self.scroll = self.scroll.clamp(
            0.0,
            (self.choices.len() as f32 * 84.0 - (bottom - top)).max(0.0),
        );
        for (i, (label, detail, action, selected)) in self.choices.iter().enumerate() {
            let y = top + i as f32 * 84.0 - self.scroll;
            if y < top || y + 76.0 > bottom {
                continue;
            }
            ui.rounded(34.0, y + 10.0, 22.0, 22.0, 11.0, color::TEXT);
            ui.rounded(36.0, y + 12.0, 18.0, 18.0, 9.0, color::BG);
            if *selected {
                ui.rounded(40.0, y + 16.0, 10.0, 10.0, 5.0, color::TEXT);
            }
            ui.text(Face::Medium, 19.0, 80.0, y + 8.0, color::TEXT, label);
            ui.text(Face::Regular, 14.0, 80.0, y + 40.0, color::MUTED, detail);
            ui.rounded(32.0, y + 75.0, w - 64.0, 1.0, 0.0, color::BORDER);
            targets.push(([32.0, y, w - 64.0, 76.0], *action));
        }
        button(
            ui,
            targets,
            [32.0, h - 78.0, 160.0, 44.0],
            "Cancel",
            Action::Dismiss,
            false,
        );
    }
}

#[cfg(test)]
mod quick_value_tests {
    use super::*;
    #[test]
    fn values_share_game_settings_and_preserve_layouts() {
        let mut s = horizon_client::touch::Settings::default();
        let before = serde_json::to_value(&s.layouts).unwrap();
        TouchValue::Opacity.apply(&mut s, 30);
        TouchValue::Sensitivity.apply(&mut s, 1000);
        TouchValue::Cursor.apply(&mut s, 250);
        assert_eq!(TouchValue::Opacity.current(&s), 30);
        assert_eq!(s.sensitivity, 10.0);
        assert_eq!(s.cursor_scale, 2.5);
        assert_eq!(serde_json::to_value(&s.layouts).unwrap(), before);
        TouchValue::Sensitivity.apply(&mut s, 9000);
        assert_eq!(s.sensitivity, 10.0);
    }
}
