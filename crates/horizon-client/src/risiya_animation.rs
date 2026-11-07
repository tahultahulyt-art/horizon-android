//! Original, local-only first-person motion. Never changes a gameplay action or clock.
use glam::{Mat4, Vec3};

pub const LABELS: [&str; 4] = ["Off", "Lite", "Full", "Reduced motion"];
/// The app variant is an explicit gate, not a preference inferred from saved data.
pub fn available() -> bool {
    option_env!("HORIZON_ANDROID_VARIANT") == Some("risiya")
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Style {
    Empty,
    Sword,
    Tool,
    Light,
    Generic,
    Stock,
}
pub fn style(id: Option<&str>) -> Style {
    let Some(id) = id else { return Style::Empty };
    let Some(n) = id.strip_prefix("minecraft:") else {
        return Style::Stock;
    };
    if [
        "bow",
        "crossbow",
        "trident",
        "filled_map",
        "map",
        "shield",
        "fishing_rod",
        "brush",
        "spyglass",
    ]
    .contains(&n)
        || n.ends_with("_spear")
    {
        Style::Stock
    } else if n.ends_with("_sword") {
        Style::Sword
    } else if ["_axe", "_pickaxe", "_shovel", "_hoe"]
        .iter()
        .any(|s| n.ends_with(s))
    {
        Style::Tool
    } else if n.contains("torch") || n.contains("lantern") {
        Style::Light
    } else {
        Style::Generic
    }
}
#[derive(Default, Debug)]
pub struct State {
    item: Option<String>,
    slot: usize,
    initialized: bool,
    inspect_start: Option<f32>,
    requested: bool,
    ground: bool,
    land: Option<f32>,
    last_time: f32,
}
impl State {
    pub fn request(&mut self) {
        self.requested = true;
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    /// Called once per render, before either hand. Returns inspect progress, or zero.
    pub fn frame(
        &mut self,
        time: f32,
        enabled: bool,
        slot: usize,
        item: Option<&str>,
        blocked: bool,
        ground: bool,
    ) -> f32 {
        if !enabled || !time.is_finite() {
            self.clear();
            return 0.0;
        }
        let changed = self.initialized && (self.slot != slot || self.item.as_deref() != item);
        let reset = time < self.last_time || time - self.last_time > 0.5;
        if !self.initialized || changed {
            self.item = item.map(str::to_owned);
            self.slot = slot;
        }
        if blocked || changed || reset {
            self.inspect_start = None;
            self.requested = false;
        }
        if self.requested && !blocked && !changed && style(item) != Style::Stock {
            self.inspect_start = Some(time);
        }
        self.requested = false;
        if self.initialized && ground && !self.ground && !reset {
            self.land = Some(time);
        }
        if reset || blocked {
            self.land = None;
        }
        self.ground = ground;
        self.last_time = time;
        self.initialized = true;
        let p = self.inspect_start.map_or(0.0, |start| (time - start) / 1.4);
        if p >= 1.0 {
            self.inspect_start = None;
            0.0
        } else {
            p.max(0.0)
        }
    }
    pub fn landing(&self, time: f32) -> f32 {
        self.land.map_or(0.0, |t| {
            let p = (time - t) / 0.25;
            if (0.0..1.0).contains(&p) {
                (p * std::f32::consts::PI).sin()
            } else {
                0.0
            }
        })
    }
}
fn smooth(p: f32) -> f32 {
    let p = p.clamp(0.0, 1.0);
    p * p * (3.0 - 2.0 * p)
}
/// Cosmetic equip easing has identical endpoints; the swap ticker stays untouched.
pub fn equip(mode: u8, original: f32) -> f32 {
    if mode == 0 {
        original
    } else {
        smooth(original)
    }
}
#[derive(Clone, Copy)]
pub struct Motion {
    pub mode: u8,
    pub style: Style,
    pub left: bool,
    pub attack: f32,
    pub time: f32,
    pub speed: f32,
    pub sprint: bool,
    pub vertical: f32,
    pub landing: f32,
    pub inspect: f32,
}
/// Pure bounded matrix evaluation. No allocations, assets, GPU passes or gameplay side effects.
pub fn transform(m: Motion) -> Mat4 {
    if m.mode == 0 || m.style == Style::Stock {
        return Mat4::IDENTITY;
    }
    let side = if m.left { -1.0 } else { 1.0 };
    let strength = match m.mode {
        1 => 0.55,
        2 => 1.0,
        _ => 0.25,
    };
    let a = m.attack.clamp(0.0, 1.0);
    let arc = (a * std::f32::consts::PI).sin();
    let (cut, tilt, reach) = match m.style {
        Style::Sword => (-0.30, 0.20, 0.08),
        Style::Tool => (-0.20, -0.35, 0.07),
        Style::Light => (0.10, 0.15, 0.04),
        Style::Empty => (0.10, -0.20, 0.10),
        _ => (-0.20, 0.25, 0.06),
    };
    let moving = m.speed.clamp(0.0, 1.0);
    let phase = m.time * if m.sprint { 12.0 } else { 8.0 };
    let sway = if m.mode == 2 { moving * 0.018 } else { 0.0 };
    let idle = if m.mode == 2 {
        (m.time * 1.8).sin() * 0.003 * (1.0 - moving)
    } else {
        0.0
    };
    // A stationary, manually requested view turn; attack/use cancels before evaluation.
    let inspect = if m.inspect > 0.0 {
        (m.inspect * std::f32::consts::PI).sin().powi(2)
    } else {
        0.0
    };
    let delta = Vec3::new(
        side * (phase.sin() * sway + arc * 0.055 + inspect * -0.16),
        idle + (phase * 2.0).cos() * sway * 0.6 - arc * 0.035 - m.landing.clamp(0.0, 1.0) * 0.025
            + m.vertical.clamp(-1.0, 1.0) * 0.012
            + inspect * 0.12,
        -arc * reach + inspect * 0.08,
    ) * strength;
    // Rotate about the held hand, not the camera. Avoid sweeping the whole base
    // translation across the view, especially at portrait/wide aspect ratios.
    let pivot = Vec3::new(side * 0.56, -0.52, -0.72);
    Mat4::from_translation(pivot + delta)
        * Mat4::from_rotation_z(side * (arc * cut + inspect * 0.45) * strength)
        * Mat4::from_rotation_x((arc * tilt + inspect * -0.25) * strength)
        * Mat4::from_rotation_y(side * inspect * 0.65 * strength)
        * Mat4::from_translation(-pivot)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn m() -> Motion {
        Motion {
            mode: 2,
            style: Style::Sword,
            left: false,
            attack: 0.4,
            time: 0.2,
            speed: 0.6,
            sprint: false,
            vertical: 0.0,
            landing: 0.0,
            inspect: 0.0,
        }
    }
    #[test]
    fn off_is_exact_identity() {
        let mut x = m();
        x.mode = 0;
        assert_eq!(transform(x), Mat4::IDENTITY);
        for p in [0.0, 0.2, 0.9, 1.0] {
            assert_eq!(equip(0, p), p)
        }
    }
    #[test]
    fn special_items_stock() {
        for id in [
            "minecraft:bow",
            "minecraft:crossbow",
            "minecraft:shield",
            "minecraft:filled_map",
            "minecraft:trident",
            "minecraft:iron_spear",
            "other:custom",
        ] {
            let mut x = m();
            x.style = style(Some(id));
            assert_eq!(transform(x), Mat4::IDENTITY)
        }
    }
    #[test]
    fn endpoints() {
        assert_eq!(equip(2, 0.0), 0.0);
        assert_eq!(equip(2, 1.0), 1.0)
    }
    #[test]
    fn inspect_interrupts() {
        let mut s = State::default();
        s.frame(0.0, true, 0, Some("minecraft:stick"), false, true);
        s.request();
        s.frame(0.1, true, 0, Some("minecraft:stick"), false, true);
        assert!(s.frame(0.2, true, 0, Some("minecraft:stick"), false, true) > 0.0);
        assert_eq!(
            s.frame(0.3, true, 0, Some("minecraft:stick"), true, true),
            0.0
        );
        s.request();
        s.frame(0.4, true, 0, Some("minecraft:stick"), false, true);
        assert_eq!(
            s.frame(0.5, true, 1, Some("minecraft:stick"), false, true),
            0.0
        )
    }
    #[test]
    fn reset_and_expire() {
        let mut s = State::default();
        s.request();
        s.frame(0.0, true, 0, None, false, true);
        assert!(s.frame(0.1, true, 0, None, false, true) > 0.0);
        assert_eq!(s.frame(5.0, true, 0, None, false, true), 0.0);
        s.request();
        s.frame(5.1, true, 0, None, false, true);
        assert_eq!(s.frame(5.2, false, 0, None, false, true), 0.0)
    }
    #[test]
    fn bounded_all_modes() {
        for mode in 0..4 {
            for style in [
                Style::Sword,
                Style::Tool,
                Style::Light,
                Style::Empty,
                Style::Generic,
            ] {
                for n in 0..100 {
                    let mut x = m();
                    x.mode = mode;
                    x.style = style;
                    x.attack = n as f32 / 99.0;
                    x.time = n as f32;
                    x.inspect = x.attack;
                    let t = transform(x);
                    assert!(
                        t.to_cols_array()
                            .iter()
                            .all(|v| v.is_finite() && v.abs() < 2.0)
                    );
                }
            }
        }
    }
    #[test]
    fn lite_has_no_continuous_sway() {
        let mut x = m();
        x.mode = 1;
        x.attack = 0.0;
        assert_eq!(transform(x), Mat4::IDENTITY)
    }
}

/// Logical HUD coordinates, below the stock top controls, shared draw and hit test.
pub fn inspect_rect(width: f32) -> [f32; 4] {
    [width / 2.0 - 26.0, 32.0, 52.0, 22.0]
}
pub fn inspect_hit(width: f32, p: [f32; 2]) -> bool {
    let [x, y, w, h] = inspect_rect(width);
    p[0] >= x && p[0] < x + w && p[1] >= y && p[1] < y + h
}

#[cfg(test)]
mod input_geometry_tests {
    use super::*;
    #[test]
    fn shared_button_rect() {
        for width in [160.0, 320.0, 640.0, 1000.0] {
            let [x, y, w, h] = inspect_rect(width);
            assert!(inspect_hit(width, [x + w / 2.0, y + h / 2.0]));
            assert!(!inspect_hit(width, [x - 1.0, y]));
            assert!(!inspect_hit(width, [x, y + h + 1.0]));
        }
    }
    #[test]
    fn left_hand_is_mirrored() {
        let make = |left| Motion {
            mode: 2,
            style: Style::Sword,
            left,
            attack: 0.5,
            time: 0.0,
            speed: 0.0,
            sprint: false,
            vertical: 0.0,
            landing: 0.0,
            inspect: 0.0,
        };
        let flip = Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0));
        assert!(transform(make(true)).abs_diff_eq(flip * transform(make(false)) * flip, 0.00001));
    }
    #[test]
    fn landing_is_short_and_non_latching() {
        let mut s = State::default();
        s.frame(0.0, true, 0, None, false, false);
        s.frame(0.1, true, 0, None, false, true);
        assert!(s.landing(0.2) > 0.0);
        assert_eq!(s.landing(0.4), 0.0);
    }
}
