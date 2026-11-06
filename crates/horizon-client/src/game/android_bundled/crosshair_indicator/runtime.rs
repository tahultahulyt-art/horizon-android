// Copyright (c) 2026 CayMac
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// Crosshair Indicator runtime module
use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "crosshair_indicator";
pub const NAME: &str = "Crosshair Indicator";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Dynamic attack reach awareness: visual feedback on your crosshair when hovering over an entity in range.";

pub const KEY_TOGGLE: &str = "toggle";
pub const KEY_TOGGLE_LABEL: &str = "Toggle Crosshair Indicator";
pub const KEY_TOGGLE_DEFAULT: &str = "key.keyboard.unknown";

// Color constants (0xAARRGGBB)
pub const COLOR_GREEN: u32 = 0xFF00FF7F; // Spring Green (ready)
pub const COLOR_ORANGE: u32 = 0xFFFF8C00; // Dark Orange (charging)
pub const COLOR_RED: u32 = 0xFFFF3333; // Vivid Red
pub const COLOR_PINK: u32 = 0xFFFF69B4; // Hot Pink
pub const COLOR_CYAN: u32 = 0xFF00FFFF; // Diamond Cyan
pub const COLOR_GOLD: u32 = 0xFFFFD700; // Rich Gold
pub const COLOR_GRAY: u32 = 0x80202020; // Cooldown background

static ENABLED: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Brackets,
    CenterDot,
    Diamond,
    Chevron,
}

impl Style {
    pub fn from_index(i: usize) -> Self {
        match i % 4 {
            0 => Self::Brackets,
            1 => Self::CenterDot,
            2 => Self::Diamond,
            _ => Self::Chevron,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Brackets => "Brackets [ + ]",
            Self::CenterDot => "Center Dot",
            Self::Diamond => "Diamond",
            Self::Chevron => "Chevron",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorTheme {
    Dynamic,
    Red,
    Pink,
    Cyan,
    Gold,
}

impl ColorTheme {
    pub fn from_index(i: usize) -> Self {
        match i % 5 {
            0 => Self::Dynamic,
            1 => Self::Red,
            2 => Self::Pink,
            3 => Self::Cyan,
            _ => Self::Gold,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Dynamic => "Dynamic (Green/Orange)",
            Self::Red => "Combat Red",
            Self::Pink => "Pink Aesthetic",
            Self::Cyan => "Diamond Cyan",
            Self::Gold => "Gold",
        }
    }

    pub fn resolve(self, charged: bool) -> u32 {
        match self {
            Self::Dynamic => {
                if charged {
                    COLOR_GREEN
                } else {
                    COLOR_ORANGE
                }
            }
            Self::Red => COLOR_RED,
            Self::Pink => COLOR_PINK,
            Self::Cyan => COLOR_CYAN,
            Self::Gold => COLOR_GOLD,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enabled: bool,
    pub style: Style,
    pub theme: ColorTheme,
    pub show_cooldown: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            style: Style::Brackets,
            theme: ColorTheme::Dynamic,
            show_cooldown: true,
        }
    }
}

impl Settings {
    pub fn labels(&self) -> Vec<String> {
        vec![
            format!(
                "Crosshair Indicator: {}",
                if self.enabled { "ON" } else { "OFF" }
            ),
            format!("Style: {}", self.style.name()),
            format!("Color: {}", self.theme.name()),
            format!(
                "Cooldown Bar: {}",
                if self.show_cooldown { "ON" } else { "OFF" }
            ),
        ]
    }

    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enabled = !self.enabled,
            1 => self.style = Style::from_index((self.style as usize) + 1),
            2 => self.theme = ColorTheme::from_index((self.theme as usize) + 1),
            3 => self.show_cooldown = !self.show_cooldown,
            _ => {}
        }
        ENABLED.store(self.enabled, Ordering::Relaxed);
    }

    pub fn stored(&self) -> Vec<(&'static str, String)> {
        vec![
            ("enabled", if self.enabled { "1" } else { "0" }.into()),
            ("style", (self.style as usize).to_string()),
            ("theme", (self.theme as usize).to_string()),
            (
                "cooldown",
                if self.show_cooldown { "1" } else { "0" }.into(),
            ),
        ]
    }
}

pub fn settings(mut read: impl FnMut(&str) -> Option<String>) -> Settings {
    let mut s = Settings::default();
    if let Some(v) = read("enabled") {
        s.enabled = v == "1" || v.eq_ignore_ascii_case("true");
    }
    if let Some(v) = read("style").and_then(|v| v.parse::<usize>().ok()) {
        s.style = Style::from_index(v);
    }
    if let Some(v) = read("theme").and_then(|v| v.parse::<usize>().ok()) {
        s.theme = ColorTheme::from_index(v);
    }
    if let Some(v) = read("cooldown") {
        s.show_cooldown = v == "1" || v.eq_ignore_ascii_case("true");
    }
    ENABLED.store(s.enabled, Ordering::Relaxed);
    s
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn set_settings(s: Settings) {
    ENABLED.store(s.enabled, Ordering::Relaxed);
}
