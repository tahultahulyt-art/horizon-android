//! User-added keys share drawing/hit geometry and retain per-finger ownership.
use super::*;
use layout::Placement;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomButton {
    pub key: String,
    pub layouts: [Placement; 3],
    #[serde(default)]
    pub caption: String,
    #[serde(default = "square_shape")]
    pub shape: f32,
    #[serde(default)]
    pub toggle: bool,
    #[serde(default)]
    pub swipe: bool,
    #[serde(default)]
    pub fill_color: Option<u32>,
    #[serde(default)]
    pub pressed_fill_color: Option<u32>,
    #[serde(default)]
    pub text_color: Option<u32>,
    #[serde(default)]
    pub decoration: Option<ButtonDecoration>,
}
/// Colors use the renderer's packed RGBA order. Corner sizes are source pixels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonDecoration {
    pub corners: [f32; 4],
    pub pressed_corners: [f32; 4],
    pub border: f32,
    pub pressed_border: f32,
    pub border_color: u32,
    pub pressed_border_color: u32,
    pub pressed_text_color: u32,
}
impl CustomButton {
    pub fn new(key: String, index: usize) -> Self {
        Self {
            key,
            caption: String::new(),
            shape: 1.0,
            toggle: false,
            swipe: false,
            fill_color: None,
            pressed_fill_color: None,
            text_color: None,
            decoration: None,
            layouts: [Placement::new(
                0.68 + (index % 3) as f32 * 0.1,
                0.35 + (index / 3 % 3) as f32 * 0.12,
                0.105,
            ); 3],
        }
    }
    pub fn valid_token(token: &str) -> bool {
        if let Some(keys) = token.strip_prefix("chord:") {
            let parts: Vec<_> = keys.split('+').collect();
            return (2..=8).contains(&parts.len())
                && parts
                    .iter()
                    .all(|key| Self::valid_single_token(key) && *key != "key.keyboard.unknown")
                && parts
                    .iter()
                    .enumerate()
                    .all(|(i, k)| !parts[..i].contains(k));
        }
        Self::valid_single_token(token)
    }
    fn valid_single_token(token: &str) -> bool {
        token == "launcher:keyboard"
            || token == "key.keyboard.unknown"
            || crate::keybinds::key_code(token).is_some()
            || crate::keybinds::mouse_button(token).is_some()
            || token
                .strip_prefix("action:")
                .is_some_and(|name| crate::keybinds::MAPPINGS.iter().any(|m| m.0 == name))
    }
    pub fn label(&self) -> String {
        if !self.caption.is_empty() {
            return self.caption.clone();
        }
        token_parts(&self.key)
            .map(single_label)
            .collect::<Vec<_>>()
            .join(" + ")
    }
}
fn square_shape() -> f32 {
    1.0
}
fn token_parts(token: &str) -> impl DoubleEndedIterator<Item = &str> {
    token.strip_prefix("chord:").unwrap_or(token).split('+')
}
fn single_label(token: &str) -> String {
    if token == "launcher:keyboard" {
        return "Keyboard".into();
    }
    if token == "key.keyboard.unknown" {
        return "Button".into();
    }
    if let Some(name) = token.strip_prefix("action:") {
        return crate::text::Text::translate(name, &[]).plain();
    }
    crate::keybinds::display_name(token).plain()
}
pub(super) fn choices(raw_keys: bool) -> Vec<(String, String)> {
    if raw_keys {
        crate::keybinds::available_keys()
            .iter()
            .map(|(name, _)| ((*name).into(), crate::keybinds::display_name(name).plain()))
            .chain(
                [
                    "key.mouse.left",
                    "key.mouse.right",
                    "key.mouse.middle",
                    "key.mouse.4",
                    "key.mouse.5",
                ]
                .into_iter()
                .map(|name| (name.to_owned(), single_label(name))),
            )
            .collect()
    } else {
        crate::keybinds::MAPPINGS
            .iter()
            .filter(|m| m.1 != "key.keyboard.unknown")
            .map(|(name, _, _)| {
                (
                    format!("action:{name}"),
                    crate::text::Text::translate(name, &[]).plain(),
                )
            })
            .collect()
    }
}
impl TouchControls {
    pub(super) fn inventory_token(control: Control) -> Option<&'static str> {
        match control {
            Control::Sneak => Some("key.keyboard.left.shift"),
            Control::Attack => Some("action:key.attack"),
            Control::Use => Some("action:key.use"),
            _ => None,
        }
    }
    pub fn inventory_update(
        &mut self,
        id: u64,
        point: [f32; 2],
        started: bool,
        ended: bool,
    ) -> bool {
        if self.custom_fingers.contains_key(&id) {
            return self.custom_update(id, point, false, ended);
        }
        if started
            && self.custom_at(point).is_none()
            && let Some(token) = self.control_at(point).and_then(Self::inventory_token)
        {
            let pressed = !self.held_custom_tokens().any(|v| v == token);
            self.custom_fingers.insert(id, token.into());
            self.fingers.insert(id, (Role::Ignore, point));
            if pressed {
                self.custom_events.push((token.into(), true));
            }
            return true;
        }
        false
    }
    pub(super) fn fit_placement(&self, mut p: Placement) -> Placement {
        p.size = (p.size * self.settings.button_scale)
            .min(self.aspect())
            .min(1.0);
        let half = [p.size / self.aspect() / 2.0, p.size / 2.0];
        for (i, half) in half.into_iter().enumerate() {
            p.center[i] = p.center[i].clamp(half, 1.0 - half);
        }
        p
    }
    pub(super) fn custom_placement(&self, index: usize) -> Placement {
        let mut p = self.settings.custom_buttons[index].layouts[self.settings.mode.index()];
        if self.settings.left_handed {
            p.center[0] = 1.0 - p.center[0];
        }
        p.size = (p.size * self.settings.button_scale).min(1.0);
        let height = p.size;
        let width = (height * self.settings.custom_buttons[index].shape / self.aspect()).min(1.0);
        p.center[0] = p.center[0].clamp(width / 2.0, 1.0 - width / 2.0);
        p.center[1] = p.center[1].clamp(height / 2.0, 1.0 - height / 2.0);
        p
    }
    pub(super) fn custom_rect(&self, index: usize) -> [f32; 4] {
        let button = &self.settings.custom_buttons[index];
        let place = self.custom_placement(index);
        let height = place.size.min(1.0);
        let width = (height * button.shape / self.aspect()).min(1.0);
        let x = place.center[0].clamp(width / 2.0, 1.0 - width / 2.0);
        let y = place.center[1].clamp(height / 2.0, 1.0 - height / 2.0);
        [x - width / 2.0, y - height / 2.0, width, height]
    }
    pub fn custom_at(&self, point: [f32; 2]) -> Option<usize> {
        (0..self.settings.custom_buttons.len())
            .rev()
            .find(|&i| layout::contains(self.custom_rect(i), point))
    }
    fn held_input_snapshot(&self) -> Vec<String> {
        self.held_custom_tokens().map(str::to_owned).collect()
    }
    fn custom_input_changed(&mut self, before: Vec<String>) {
        let after = self.held_input_snapshot();
        let mut emitted = std::collections::BTreeSet::new();
        for token in before.iter().rev() {
            if !after.contains(token) && emitted.insert(token.clone()) {
                self.custom_events.push((token.clone(), false));
            }
        }
        emitted.clear();
        for token in after {
            if !before.contains(&token) && emitted.insert(token.clone()) {
                self.custom_events.push((token, true));
            }
        }
    }
    pub(super) fn custom_update(
        &mut self,
        id: u64,
        point: [f32; 2],
        started: bool,
        ended: bool,
    ) -> bool {
        if self.custom_fingers.contains_key(&id) {
            let before = self.held_input_snapshot();
            if ended {
                self.custom_fingers.remove(&id);
                self.custom_owners.remove(&id);
                self.fingers.remove(&id);
            } else if let Some(old) = self.custom_owners.get(&id).copied()
                && self
                    .settings
                    .custom_buttons
                    .get(old)
                    .is_some_and(|b| b.swipe && !b.toggle)
                && let Some(next) = self.custom_at(point)
                && next != old
                && self.settings.custom_buttons[next].swipe
                && !self.settings.custom_buttons[next].toggle
            {
                self.custom_fingers
                    .insert(id, self.settings.custom_buttons[next].key.clone());
                self.custom_owners.insert(id, next);
            }
            self.custom_input_changed(before);
            return true;
        }
        if started && let Some(index) = self.custom_at(point) {
            let button = &self.settings.custom_buttons[index];
            let key = button.key.clone();
            if key == "key.keyboard.unknown" {
                return false;
            }
            let before = self.held_input_snapshot();
            if button.toggle {
                if self.custom_latched.remove(&index).is_none() {
                    self.custom_latched.insert(index, key);
                }
                // Empty finger entry consumes the release without unlatching the key.
                self.custom_fingers.insert(id, String::new());
            } else {
                self.custom_fingers.insert(id, key);
            }
            self.custom_owners.insert(id, index);
            self.fingers.insert(id, (Role::Ignore, point));
            self.custom_input_changed(before);
            self.clicked = true;
            return true;
        }
        false
    }
    pub fn held_custom_tokens(&self) -> impl Iterator<Item = &str> {
        self.custom_fingers
            .values()
            .chain(self.custom_latched.values())
            .flat_map(|token| token_parts(token))
            .filter(|token| !token.is_empty())
    }
    pub fn key_events(&mut self) -> Vec<(String, bool)> {
        std::mem::take(&mut self.custom_events)
    }
    pub(super) fn clear_custom(&mut self) {
        let before = self.held_input_snapshot();
        self.custom_fingers.clear();
        self.custom_latched.clear();
        self.custom_owners.clear();
        self.custom_input_changed(before);
    }
    pub(super) fn draw_custom(&self, b: &mut HudBatch, a: &HudAtlas) {
        for (i, button) in self.settings.custom_buttons.iter().enumerate() {
            let p = self.custom_placement(i);
            let bounds = self.custom_rect(i);
            let rect = [
                bounds[0] * b.width,
                bounds[1] * b.height,
                bounds[2] * b.width,
                bounds[3] * b.height,
            ];
            let pressed = self.custom_fingers.values().any(|key| *key == button.key)
                || self.custom_latched.contains_key(&i);
            let opacity = self
                .settings
                .draw_opacity(p.opacity, self.editor.is_some(), false);
            if opacity <= 0.0 {
                continue;
            }
            if let Some(background) = button.fill_color {
                let tint = if pressed {
                    button.pressed_fill_color.unwrap_or(background)
                } else {
                    background
                };
                let color = faded(tint, opacity);
                if let Some(style) = &button.decoration {
                    let corners = if pressed {
                        style.pressed_corners
                    } else {
                        style.corners
                    };
                    let border = if pressed {
                        style.pressed_border
                    } else {
                        style.border
                    };
                    let stroke = if pressed {
                        style.pressed_border_color
                    } else {
                        style.border_color
                    };
                    decorated_button(b, a, rect, corners, border, color, faded(stroke, opacity));
                } else {
                    b.fill(a, rect[0], rect[1], rect[2], rect[3], color);
                }
                let label = button.label();
                let text_color = if pressed {
                    button
                        .decoration
                        .as_ref()
                        .map(|s| s.pressed_text_color)
                        .or(button.text_color)
                } else {
                    button.text_color
                };
                let [red, green, blue, alpha] = text_color.unwrap_or(u32::MAX).to_le_bytes();
                let start = b.vertices.len();
                let width = HudBatch::text_width(a, &label).max(1.0);
                b.text(
                    a,
                    &label,
                    rect[0] + (rect[2] - width) / 2.0,
                    rect[1] + (rect[3] - 8.0) / 2.0,
                    rgba(red, green, blue, (alpha as f32 * opacity) as u8),
                    false,
                );
                let scale = ((rect[2] - 4.0).max(1.0) / width).min(1.0);
                b.scale_since(
                    start,
                    rect[0] + rect[2] / 2.0,
                    rect[1] + rect[3] / 2.0,
                    scale,
                    scale,
                );
            } else {
                text_button(
                    b,
                    a,
                    rect,
                    &button.label(),
                    pressed,
                    rgba(255, 255, 255, (opacity * 255.0) as u8),
                    true,
                );
            }
        }
    }
}
fn faded(color: u32, opacity: f32) -> u32 {
    let [r, g, b, a] = color.to_le_bytes();
    rgba(r, g, b, (a as f32 * opacity) as u8)
}
/// One convex perimeter shared by the fill and border, with no translucent overlap.
fn rounded_points([x, y, w, h]: [f32; 4], mut radii: [f32; 4]) -> Vec<[f32; 2]> {
    for radius in &mut radii {
        *radius = if radius.is_finite() {
            radius.max(0.0)
        } else {
            0.0
        };
    }
    let mut scale = 1.0_f32;
    for (edge, sum) in [
        (w, radii[0] + radii[1]),
        (h, radii[1] + radii[2]),
        (w, radii[2] + radii[3]),
        (h, radii[3] + radii[0]),
    ] {
        if sum > 0.0 {
            scale = scale.min(edge / sum);
        }
    }
    radii = radii.map(|r| r * scale.max(0.0));
    let centers = [
        [x + radii[0], y + radii[0]],
        [x + w - radii[1], y + radii[1]],
        [x + w - radii[2], y + h - radii[2]],
        [x + radii[3], y + h - radii[3]],
    ];
    let mut points = Vec::with_capacity(36);
    for i in 0..4 {
        for step in 0..=8 {
            let angle = (180.0 + i as f32 * 90.0 + step as f32 * 90.0 / 8.0).to_radians();
            points.push([
                centers[i][0] + radii[i] * angle.cos(),
                centers[i][1] + radii[i] * angle.sin(),
            ]);
        }
    }
    points
}
fn decorated_button(
    b: &mut HudBatch,
    a: &HudAtlas,
    rect: [f32; 4],
    corners: [f32; 4],
    border: f32,
    fill: u32,
    stroke: u32,
) {
    let Some(white) = a.sprites.get("horizon/white") else {
        b.fill(a, rect[0], rect[1], rect[2], rect[3], fill);
        return;
    };
    let uv = [[(white.u0 + white.u1) / 2.0, (white.v0 + white.v1) / 2.0]; 4];
    let scale = b.scale.max(0.1);
    let radii = corners.map(|r| r / scale);
    let thickness = (border / scale).max(0.0).min(rect[2].min(rect[3]) / 2.0);
    let outer = rounded_points(rect, radii);
    let inner = rounded_points(
        [
            rect[0] + thickness,
            rect[1] + thickness,
            (rect[2] - 2.0 * thickness).max(0.0),
            (rect[3] - 2.0 * thickness).max(0.0),
        ],
        radii.map(|r| (r - thickness).max(0.0)),
    );
    let center = [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0];
    for i in 0..outer.len() {
        let j = (i + 1) % outer.len();
        if thickness > 0.0 {
            b.quad([outer[i], outer[j], inner[j], inner[i]], uv, stroke);
        }
        b.quad([center, inner[i], inner[j], center], uv, fill);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rounded_perimeter_keeps_asymmetric_corners_inside_bounds() {
        let p = rounded_points([10.0, 20.0, 100.0, 60.0], [20.0, 0.0, 15.0, 0.0]);
        assert_eq!(p.len(), 36);
        assert!((p[0][0] - 10.0).abs() < 0.001);
        assert!((p[0][1] - 40.0).abs() < 0.001);
        assert!(
            p.iter()
                .all(|q| q[0] >= 9.999 && q[0] <= 110.001 && q[1] >= 19.999 && q[1] <= 80.001)
        );
        let large = rounded_points([0.0, 0.0, 10.0, 10.0], [40.0; 4]);
        assert!(
            large
                .iter()
                .all(|q| q[0] >= -0.001 && q[0] <= 10.001 && q[1] >= -0.001 && q[1] <= 10.001)
        );
    }
    #[test]
    fn toggle_and_swipe_release_without_stuck_inputs() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        let mut toggle = CustomButton::new("key.keyboard.left.control".into(), 0);
        toggle.toggle = true;
        t.settings.custom_buttons.push(toggle);
        let p = t.custom_placement(0).center;
        t.custom_update(1, p, true, false);
        assert_eq!(t.key_events(), [("key.keyboard.left.control".into(), true)]);
        t.custom_update(1, p, false, true);
        assert!(t.key_events().is_empty());
        t.custom_update(2, p, true, false);
        assert_eq!(
            t.key_events(),
            [("key.keyboard.left.control".into(), false)]
        );
        t.clear();
        t.settings.custom_buttons.clear();
        for (i, key) in [
            "chord:key.keyboard.w+key.keyboard.a",
            "chord:key.keyboard.w+key.keyboard.d",
        ]
        .into_iter()
        .enumerate()
        {
            let mut b = CustomButton::new(key.into(), i);
            b.swipe = true;
            t.settings.custom_buttons.push(b);
        }
        let p0 = t.custom_placement(0).center;
        let p1 = t.custom_placement(1).center;
        t.custom_update(1, p0, true, false);
        t.key_events();
        t.custom_update(1, p1, false, false);
        assert_eq!(
            t.key_events(),
            [
                ("key.keyboard.a".into(), false),
                ("key.keyboard.d".into(), true)
            ]
        );
        t.clear();
        assert_eq!(t.held_custom_tokens().count(), 0);
    }
    #[test]
    fn narrow_rectangles_keep_their_source_center() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        let mut b = CustomButton::new("key.keyboard.a".into(), 0);
        b.shape = 0.2;
        b.layouts = [Placement::new(0.02, 0.5, 0.2); 3];
        t.settings.custom_buttons.push(b);
        let rect = t.custom_rect(0);
        assert!((rect[0] + rect[2] / 2.0 - 0.02).abs() < 0.001);
    }
    #[test]
    fn chords_validate_and_shared_keys_release_after_last_owner() {
        assert!(CustomButton::valid_token(
            "chord:key.keyboard.f3+key.keyboard.b"
        ));
        assert!(!CustomButton::valid_token(
            "chord:key.keyboard.f3+key.keyboard.f3"
        ));
        assert!(!CustomButton::valid_token("chord:key.keyboard.f3+invalid"));
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.settings.custom_buttons.push(CustomButton::new(
            "chord:key.keyboard.f3+key.keyboard.b".into(),
            0,
        ));
        t.settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.f3".into(), 1));
        let p0 = t.custom_placement(0).center;
        let p1 = t.custom_placement(1).center;
        assert!(t.custom_update(1, p0, true, false));
        assert_eq!(
            t.key_events(),
            [
                ("key.keyboard.f3".into(), true),
                ("key.keyboard.b".into(), true)
            ]
        );
        assert!(t.custom_update(2, p1, true, false));
        assert!(t.key_events().is_empty());
        assert!(t.custom_update(1, p0, false, true));
        assert_eq!(t.key_events(), [("key.keyboard.b".into(), false)]);
        t.clear();
        assert_eq!(t.key_events(), [("key.keyboard.f3".into(), false)]);
    }
    #[test]
    fn inventory_modifier_and_mouse_hold_independently_and_cancel_cleanly() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.inventory_mode = true;
        t.settings.mode = Mode::Crosshair;
        for (id, control, token) in [
            (1, Control::Sneak, "key.keyboard.left.shift"),
            (2, Control::Attack, "action:key.attack"),
            (3, Control::Use, "action:key.use"),
        ] {
            assert!(t.visible(control));
            assert!(t.inventory_update(id, t.placement(control).center, true, false));
            assert_eq!(t.key_events(), [(token.into(), true)]);
        }
        t.clear_world_gesture();
        assert_eq!(t.held_custom_tokens().count(), 3);
        assert_eq!(t.poll_state(), PadState::default());
        assert!(t.inventory_update(2, [0.0, 0.0], false, true));
        assert_eq!(t.key_events(), [("action:key.attack".into(), false)]);
        t.clear();
        let released = t.key_events();
        assert!(released.contains(&("key.keyboard.left.shift".into(), false)));
        assert!(released.contains(&("action:key.use".into(), false)));
    }
    #[test]
    fn inventory_overlay_preserves_configured_visibility_and_placements() {
        let mut touch = TouchControls::default();
        for mode in Mode::ALL {
            touch.settings.mode = mode;
            touch.settings.button_visibility[Control::Attack as usize] = 2;
            let before: Vec<_> = Control::ALL
                .into_iter()
                .map(|c| (c, touch.visible(c), touch.placement(c).center))
                .collect();
            touch.inventory_mode = true;
            for (control, visible, center) in before {
                assert_eq!(touch.visible(control), visible);
                assert_eq!(touch.placement(control).center, center);
            }
            touch.inventory_mode = false;
        }
    }
    #[test]
    fn unassigned_button_emits_no_input() {
        let mut touch = TouchControls::default();
        touch.viewport(1920, 1080);
        touch
            .settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.unknown".into(), 0));
        let point = touch.custom_placement(0).center;
        touch.update(1, point, true, false);
        assert!(touch.key_events().is_empty());
    }
    #[test]
    fn custom_keys_hold_through_motion_and_shared_keys_release_only_after_last_finger() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.z".into(), 0));
        let p = t.custom_placement(0).center;
        t.update(1, p, true, false);
        t.update(2, p, true, false);
        assert_eq!(t.key_events(), [("key.keyboard.z".into(), true)]);
        t.update(1, [0.99, 0.1], false, false);
        assert!(t.key_events().is_empty());
        t.update(1, p, false, true);
        assert!(t.key_events().is_empty());
        t.update(2, p, false, true);
        assert_eq!(t.key_events(), [("key.keyboard.z".into(), false)]);
    }
    #[test]
    fn cancellation_releases_all_custom_keys_and_scaling_preserves_saved_sizes() {
        let mut t = TouchControls::default();
        t.viewport(1920, 1080);
        t.settings
            .custom_buttons
            .push(CustomButton::new("key.keyboard.z".into(), 0));
        let stored = t.settings.custom_buttons[0].layouts[0].size;
        t.settings.button_scale = 2.0;
        assert_eq!(t.custom_placement(0).size, stored * 2.0);
        t.update(1, t.custom_placement(0).center, true, false);
        t.key_events();
        t.clear();
        assert_eq!(t.key_events(), [("key.keyboard.z".into(), false)]);
        assert_eq!(t.settings.custom_buttons[0].layouts[0].size, stored);
    }
}
