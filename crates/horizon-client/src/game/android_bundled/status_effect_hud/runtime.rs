// Copyright (c) 2026 CayMac
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// Status Effect HUD runtime state & pure formatting/bar calculations.
// Compiles both standalone (`cargo test`) and inside `horizon-client` via `horizon-mixins`.

use std::collections::BTreeMap;
use std::sync::Mutex;

pub const ID: &str = "status_effect_hud";
pub const NAME: &str = "Status Effect HUD";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "1:1 Status Effect Bars & Inventory HUD+ Potion HUD: displays active potion/beacon effects with icons, Roman numeral amplifiers, color-coded mm:ss countdown timers, and potion-colored duration bars.";

pub const BACKGROUND: u32 = 0x7800_0000;
pub const BAR_TRACK_BG: u32 = 0x9022_2222;
pub const COLOR_NAME: u32 = 0x00FF_FFFF;
pub const COLOR_TIMER_NORMAL: u32 = 0x00EE_EEEE;
pub const COLOR_TIMER_WARN: u32 = 0x00FF_AA00;
pub const COLOR_TIMER_URGENT: u32 = 0x00FF_5555;
pub const COLOR_TIMER_FLASH: u32 = 0x00FF_BBBB;
pub const COLOR_INFINITE: u32 = 0x0055_FFFF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub enabled: bool,
    /// 0 = Top-Right, 1 = Top-Left, 2 = Bottom-Left, 3 = Bottom-Right
    pub position: u8,
    /// 0 = Full (Icon + Name + Amplifier + Timer + Bar), 1 = Compact (Icon + Amplifier + Timer + Bar)
    pub style: u8,
    pub show_bars: bool,
    pub background: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            position: 0,
            style: 0,
            show_bars: true,
            background: true,
        }
    }
}

impl Settings {
    pub fn load<'a>(mut get: impl FnMut(&str) -> Option<String> + 'a) -> Self {
        let mut s = Self::default();
        if let Some(v) = get("enabled") {
            s.enabled = v != "false" && v != "0";
        }
        if let Some(v) = get("position").and_then(|v| v.parse::<u8>().ok()) {
            s.position = v % 4;
        }
        if let Some(v) = get("style").and_then(|v| v.parse::<u8>().ok()) {
            s.style = v % 2;
        }
        if let Some(v) = get("show_bars") {
            s.show_bars = v != "false" && v != "0";
        }
        if let Some(v) = get("background") {
            s.background = v != "false" && v != "0";
        }
        s
    }

    pub fn stored(&self) -> [(&'static str, String); 5] {
        let b = |v: bool| if v { "true" } else { "false" }.to_owned();
        [
            ("enabled", b(self.enabled)),
            ("position", self.position.to_string()),
            ("style", self.style.to_string()),
            ("show_bars", b(self.show_bars)),
            ("background", b(self.background)),
        ]
    }

    pub fn position_name(pos: u8) -> &'static str {
        match pos % 4 {
            0 => "Top-Right",
            1 => "Top-Left",
            2 => "Bottom-Left",
            _ => "Bottom-Right",
        }
    }

    pub fn style_name(style: u8) -> &'static str {
        match style % 2 {
            0 => "Full (Name + Timer)",
            _ => "Compact (Icon + Timer)",
        }
    }

    pub fn labels(&self) -> Vec<String> {
        let on_off = |v: bool| if v { "ON" } else { "OFF" };
        vec![
            format!("Status Effect HUD: {}", on_off(self.enabled)),
            format!("HUD Position: {}", Self::position_name(self.position)),
            format!("Display Mode: {}", Self::style_name(self.style)),
            format!("Duration Progress Bars: {}", on_off(self.show_bars)),
            format!("Background Box: {}", on_off(self.background)),
        ]
    }

    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enabled = !self.enabled,
            1 => self.position = (self.position + 1) % 4,
            2 => self.style = (self.style + 1) % 2,
            3 => self.show_bars = !self.show_bars,
            4 => self.background = !self.background,
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RuntimeState {
    pub loaded: bool,
    pub settings: Settings,
    pub max_durations: BTreeMap<String, i32>,
}

static STATE: Mutex<RuntimeState> = Mutex::new(RuntimeState {
    loaded: false,
    settings: Settings {
        enabled: true,
        position: 0,
        style: 0,
        show_bars: true,
        background: true,
    },
    max_durations: BTreeMap::new(),
});

pub fn settings<'a>(get: impl FnMut(&str) -> Option<String> + 'a) -> Settings {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if !guard.loaded {
        guard.settings = Settings::load(get);
        guard.loaded = true;
    }
    guard.settings
}

pub fn set_settings(s: Settings) {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    guard.settings = s;
    guard.loaded = true;
}

/// Updates the tracked maximum duration for `effect_id` and returns the progress fraction `0.0..=1.0`.
pub fn update_and_compute_progress(effect_id: &str, duration: i32) -> f32 {
    if duration < 0 {
        return 1.0;
    }
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let entry = guard
        .max_durations
        .entry(effect_id.to_owned())
        .or_insert(duration.max(1));
    if duration > *entry {
        *entry = duration;
    }
    let max_d = (*entry).max(1) as f32;
    (duration.max(0) as f32 / max_d).clamp(0.0, 1.0)
}

/// Converts amplifier `0..=9` to Roman numeral suffix (`""` for 0, `"II"` for 1, `"III"` for 2, etc.).
pub fn roman_amplifier(amplifier: i32) -> &'static str {
    match amplifier {
        0 => "",
        1 => "II",
        2 => "III",
        3 => "IV",
        4 => "V",
        5 => "VI",
        6 => "VII",
        7 => "VIII",
        8 => "IX",
        9 => "X",
        _ => "",
    }
}

/// Formats tick duration (`mm:ss` or `hh:mm:ss` or `∞` for `-1`) and returns `(text, rgb_color)`.
pub fn format_effect_timer(duration_ticks: i32, tick_rate: f32, game_ticks: u64) -> (String, u32) {
    if duration_ticks < 0 {
        return ("∞".to_owned(), COLOR_INFINITE);
    }
    let rate = if tick_rate > 0.0 { tick_rate } else { 20.0 };
    let secs = (duration_ticks.max(0) as f32 / rate).floor() as i32;
    let h = secs / 3600;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    let text = if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    };
    let color = if secs <= 10 {
        if (game_ticks / 10).is_multiple_of(2) {
            COLOR_TIMER_URGENT
        } else {
            COLOR_TIMER_FLASH
        }
    } else if secs <= 30 {
        COLOR_TIMER_WARN
    } else {
        COLOR_TIMER_NORMAL
    };
    (text, color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_formatting_and_warning_colors_match_java_mod() {
        let (inf, c_inf) = format_effect_timer(-1, 20.0, 0);
        assert_eq!(inf, "∞");
        assert_eq!(c_inf, COLOR_INFINITE);

        let (normal, c_norm) = format_effect_timer(20 * 90, 20.0, 0);
        assert_eq!(normal, "01:30");
        assert_eq!(c_norm, COLOR_TIMER_NORMAL);

        let (warn, c_warn) = format_effect_timer(20 * 20, 20.0, 0);
        assert_eq!(warn, "00:20");
        assert_eq!(c_warn, COLOR_TIMER_WARN);

        let (urgent1, c_urg1) = format_effect_timer(20 * 5, 20.0, 0);
        let (_, c_urg2) = format_effect_timer(20 * 5, 20.0, 10);
        assert_eq!(urgent1, "00:05");
        assert_eq!(c_urg1, COLOR_TIMER_URGENT);
        assert_eq!(c_urg2, COLOR_TIMER_FLASH);
    }

    #[test]
    fn progress_tracking_scales_from_initial_max_duration() {
        let p1 = update_and_compute_progress("speed", 600);
        assert!((p1 - 1.0).abs() < 1e-5);
        let p2 = update_and_compute_progress("speed", 300);
        assert!((p2 - 0.5).abs() < 1e-5);
        let p3 = update_and_compute_progress("speed", 1200);
        assert!((p3 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn settings_roundtrip() {
        let mut s = Settings::default();
        s.click(1);
        s.click(2);
        assert_eq!(s.position, 1);
        assert_eq!(s.style, 1);
        let map: std::collections::HashMap<_, _> = s.stored().into_iter().collect();
        let loaded = Settings::load(|k| map.get(k).cloned());
        assert_eq!(s, loaded);
    }
}

pub fn reset() {
    let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
    state.loaded = false;
    state.max_durations.clear();
}
