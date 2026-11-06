// Copyright (c) 2026 Thomas-Slot
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// Armor HUD: the mod's own code. This file is both the Cargo test library
// and the module the bundle adds to the game as crate::game::armorhud.
// Use ordinary comments only in bundle files: the bundle rules refuse the
// crate-level (inner) attribute and doc-comment forms.

use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

pub const ID: &str = "armorhud";
pub const VERSION: &str = "1.0.0";

// The player's key for "toggle durability numbers" (rebindable in Options > Mods).
pub const KEY_NAME: &str = "numbers";
pub const DEFAULT_KEY: &str = "key.keyboard.k";

// Setting keys, one file each under the mod's data folder.
pub const NUMBERS: &str = "numbers";
pub const ALERTS: &str = "alerts";

// (setting key, label, default). The order is the order of the settings buttons.
pub const TOGGLES: [(&str, &str, bool); 2] = [
    (NUMBERS, "Durability numbers", false),
    (ALERTS, "Low-durability alerts", true),
];

// A setting that was never saved takes its default.
pub fn enabled(stored: Option<&str>, default: bool) -> bool {
    match stored {
        Some("true") => true,
        Some("false") => false,
        _ => default,
    }
}

pub fn stored(on: bool) -> &'static str {
    if on { "true" } else { "false" }
}

pub fn button_label(label: &str, on: bool) -> String {
    format!("{label}: {}", if on { "ON" } else { "OFF" })
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// Seconds since the mod first asked; the clock for the alert's pulse.
pub fn now_seconds() -> f32 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f32()
}

// ---- the two settings the HUD reads every frame, cached ----
// native_setting reads a file each call, so the HUD asks at most 4 times a
// second, and the key and the settings page drop the cache when they change one.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flags {
    pub numbers: bool,
    pub alerts: bool,
}

static FLAGS: Mutex<Option<(Instant, Flags)>> = Mutex::new(None);
const FLAGS_REFRESH: f32 = 0.25;

pub fn cached_flags(read: impl Fn(&str) -> Option<String>) -> Flags {
    let mut cache = lock(&FLAGS);
    if let Some((at, flags)) = *cache {
        if at.elapsed().as_secs_f32() < FLAGS_REFRESH {
            return flags;
        }
    }
    let flags = Flags {
        numbers: enabled(read(NUMBERS).as_deref(), false),
        alerts: enabled(read(ALERTS).as_deref(), true),
    };
    *cache = Some((Instant::now(), flags));
    flags
}

pub fn invalidate_flags() {
    *lock(&FLAGS) = None;
}

// ---- the notification above the hotbar ----
// Full for a second, then fading out over 0.8 seconds.

pub const NOTICE_HOLD: f32 = 1.0;
pub const NOTICE_FADE: f32 = 0.8;

static NOTICE: Mutex<Option<(Instant, bool)>> = Mutex::new(None);

pub fn notice_text(on: bool) -> String {
    format!("Durability numbers: {}", if on { "ON" } else { "OFF" })
}

// The text's alpha (0-255) this long after the toggle; None once it is gone.
pub fn notice_alpha(seconds: f32) -> Option<u8> {
    let into_fade = seconds - NOTICE_HOLD;
    if into_fade <= 0.0 {
        Some(255)
    } else if into_fade >= NOTICE_FADE {
        None
    } else {
        Some((255.0 * (1.0 - into_fade / NOTICE_FADE)).round() as u8)
    }
}

pub fn show_notice(numbers_on: bool) {
    *lock(&NOTICE) = Some((Instant::now(), numbers_on));
}

// The notification to draw now: its text and alpha, or None.
pub fn current_notice() -> Option<(String, u8)> {
    let mut notice = lock(&NOTICE);
    let (since, on) = (*notice)?;
    match notice_alpha(since.elapsed().as_secs_f32()) {
        Some(alpha) => Some((notice_text(on), alpha)),
        None => {
            *notice = None;
            None
        }
    }
}

// Armor and durability rules as plain data and functions.
#[allow(dead_code)]
pub mod armor {
    // Below this share of durability left, a piece counts as low (10%).
    pub const LOW_FRACTION: f32 = 0.10;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ArmorSlot {
        Boots,
        Leggings,
        Chestplate,
        Helmet,
    }

    // Left to right on screen in the original horizontal plan.
    pub const DISPLAY_ORDER: [ArmorSlot; 4] = [
        ArmorSlot::Boots,
        ArmorSlot::Leggings,
        ArmorSlot::Chestplate,
        ArmorSlot::Helmet,
    ];

    // Top to bottom in the vertical column, as the inventory lists armor.
    // This is also the order of the game's own `armor` array.
    pub const COLUMN_ORDER: [ArmorSlot; 4] = [
        ArmorSlot::Helmet,
        ArmorSlot::Chestplate,
        ArmorSlot::Leggings,
        ArmorSlot::Boots,
    ];

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Durability {
        pub damage: u32,
        pub max_damage: u32,
    }

    impl Durability {
        // None for an item that cannot be damaged (max durability 0).
        pub fn new(damage: u32, max_damage: u32) -> Option<Durability> {
            if max_damage == 0 {
                None
            } else {
                Some(Durability {
                    damage: damage.min(max_damage),
                    max_damage,
                })
            }
        }

        pub fn remaining(self) -> u32 {
            self.max_damage - self.damage
        }

        // Nothing used yet: the bar stays hidden, as vanilla's does.
        pub fn is_full(self) -> bool {
            self.damage == 0
        }

        // 1.0 is new, 0.0 is about to break.
        pub fn fraction(self) -> f32 {
            self.remaining() as f32 / self.max_damage as f32
        }

        // The bar's filled width in pixels, rounded as vanilla's item bar does.
        pub fn bar_pixels(self, full: u32) -> u32 {
            (full as f32 * self.fraction()).round() as u32
        }

        pub fn color(self) -> u32 {
            bar_color(self.fraction())
        }

        pub fn is_low(self) -> bool {
            self.fraction() < LOW_FRACTION
        }

        // The number the numeric overlay shows: durability left.
        pub fn numbers(self) -> String {
            self.remaining().to_string()
        }
    }

    // Green (full) through yellow to red (empty): the hue runs from 120 to 0
    // degrees as the fraction falls, as vanilla's item bar does. 0xRRGGBB.
    pub fn bar_color(fraction: f32) -> u32 {
        hsv_to_rgb(fraction.clamp(0.0, 1.0) / 3.0, 1.0, 1.0)
    }

    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> u32 {
        let h6 = h.clamp(0.0, 1.0) * 6.0;
        let i = h6.floor();
        let f = h6 - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);
        let (r, g, b) = match (i as u32) % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };
        let byte = |c: f32| (c * 255.0 + 0.5) as u32;
        (byte(r) << 16) | (byte(g) << 8) | byte(b)
    }
}

// The low-durability alert: a red tint and a red outline that pulse.
#[allow(dead_code)]
pub mod alert {
    pub const PERIOD: f32 = 0.8; // seconds per pulse
    pub const TINT_MIN: u8 = 0x30;
    pub const TINT_MAX: u8 = 0x80;
    pub const OUTLINE_MIN: u8 = 0x90;
    pub const OUTLINE_MAX: u8 = 0xFF;
    pub const RED: u32 = 0x00FF_3030; // add an alpha in the top byte

    // 0.0 to 1.0, smoothly up and down.
    pub fn pulse(seconds: f32) -> f32 {
        0.5 + 0.5 * (seconds * std::f32::consts::TAU / PERIOD).sin()
    }

    fn lerp(lo: u8, hi: u8, t: f32) -> u8 {
        (lo as f32 + (hi as f32 - lo as f32) * t.clamp(0.0, 1.0)).round() as u8
    }

    // (interior tint alpha, outline alpha) at this time.
    pub fn alphas(seconds: f32) -> (u8, u8) {
        let p = pulse(seconds);
        (
            lerp(TINT_MIN, TINT_MAX, p),
            lerp(OUTLINE_MIN, OUTLINE_MAX, p),
        )
    }

    pub fn argb(alpha: u8) -> u32 {
        ((alpha as u32) << 24) | RED
    }
}

// Where the armor column goes, in GUI pixels, from vanilla's own hotbar numbers
// (hud_layout.rs: the hotbar is 182 wide, centred, 22 tall; its items sit at
// h - 16 - 3; the off-hand sprite is 29 wide, left of the hotbar for a
// right-handed player). The column copies the hotbar's slot look: a 1px black
// frame round the whole column, and per slot a 20x20 cell of two bevel rings
// (outer, inner) round a 16x16 translucent interior.
#[allow(dead_code)]
pub mod layout {
    pub const CELL: i32 = 20; // one slot's pitch, as in the hotbar
    pub const ITEM: i32 = 16;
    pub const BORDER: i32 = 1; // the black frame round the column
    pub const HOTBAR_HALF: i32 = 91;
    pub const OFFHAND_WIDTH: i32 = 29;
    pub const ITEM_BOTTOM_GAP: i32 = 3;
    pub const SLOTS: i32 = 4;
    pub const BAR_W: u32 = 13; // the durability bar's full width, as vanilla's
    // Space between the off-hand slot and the column. The off-hand sprite is 29
    // wide but only its left 22 pixels are visible, so the hotbar sits 7 pixels
    // from the visible slot: the column keeps exactly that distance too.
    pub const COLUMN_GAP: i32 = 7;

    // Colours sampled from the hotbar (0xAARRGGBB). Top and left edges are lit.
    pub const OUTER_LIGHT: u32 = 0xFF93_9393;
    pub const OUTER_DARK: u32 = 0xFF68_6868;
    pub const INNER_DARK: u32 = 0xFF5D_5D5D;
    pub const INNER_LIGHT: u32 = 0xFF7F_7F7F;
    pub const INTERIOR: u32 = 0x6600_0000;
    pub const BLACK: u32 = 0xFF00_0000;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Rect {
        pub x: i32,
        pub y: i32,
        pub w: i32,
        pub h: i32,
    }

    // One armor slot: its 20x20 cell, and where its 16x16 icon goes.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SlotBox {
        pub frame: Rect,
        pub item_x: i32,
        pub item_y: i32,
    }

    // The x where the column's outer edge ends: COLUMN_GAP left of the off-hand
    // sprite's slot (or of the hotbar when the off hand is on the right). It does
    // not depend on whether the off hand holds an item, so the column stays put.
    pub fn column_right_edge(gui_width: i32, offhand_on_left: bool) -> i32 {
        let hotbar_left = gui_width / 2 - HOTBAR_HALF;
        let anchor = if offhand_on_left {
            hotbar_left - OFFHAND_WIDTH
        } else {
            hotbar_left
        };
        anchor - COLUMN_GAP
    }

    // The four slots, top to bottom (see COLUMN_ORDER). The lowest cell sits on
    // the same row as the hotbar's cells. None when it would leave the screen.
    pub fn armor_column(
        gui_width: i32,
        gui_height: i32,
        offhand_on_left: bool,
    ) -> Option<[SlotBox; 4]> {
        let right = column_right_edge(gui_width, offhand_on_left);
        let cell_x = right - BORDER - CELL;
        let bottom_y = gui_height - ITEM - ITEM_BOTTOM_GAP - 2;
        let top_y = bottom_y - (SLOTS - 1) * CELL;
        if cell_x - BORDER < 0 || top_y - BORDER < 0 {
            return None;
        }
        Some(std::array::from_fn(|i| {
            let y = top_y + i as i32 * CELL;
            SlotBox {
                frame: Rect {
                    x: cell_x,
                    y,
                    w: CELL,
                    h: CELL,
                },
                item_x: cell_x + 2,
                item_y: y + 2,
            }
        }))
    }

    // The black frame round the whole column.
    pub fn column_border(slots: &[SlotBox; 4]) -> Rect {
        let first = slots[0].frame;
        let last = slots[3].frame;
        Rect {
            x: first.x - BORDER,
            y: first.y - BORDER,
            w: CELL + 2 * BORDER,
            h: last.y + CELL - first.y + 2 * BORDER,
        }
    }

    // The four black strips of the column frame, as [x, y, w, h].
    pub fn border_strips(b: Rect) -> [[i32; 4]; 4] {
        [
            [b.x, b.y, b.w, 1],
            [b.x, b.y + b.h - 1, b.w, 1],
            [b.x, b.y + 1, 1, b.h - 2],
            [b.x + b.w - 1, b.y + 1, 1, b.h - 2],
        ]
    }

    // One slot's look as (strip, colour) pairs, strips as [x, y, w, h]: a
    // translucent 16x16 interior inside an inner ring and an outer ring, each
    // lit on its top and left edges.
    pub fn cell_strips(x: i32, y: i32) -> [([i32; 4], u32); 9] {
        [
            ([x + 2, y + 2, 16, 16], INTERIOR),
            ([x, y, 20, 1], OUTER_LIGHT),
            ([x, y + 1, 1, 19], OUTER_LIGHT),
            ([x + 1, y + 19, 19, 1], OUTER_DARK),
            ([x + 19, y + 1, 1, 18], OUTER_DARK),
            ([x + 1, y + 1, 18, 1], INNER_DARK),
            ([x + 1, y + 2, 1, 17], INNER_DARK),
            ([x + 2, y + 18, 17, 1], INNER_LIGHT),
            ([x + 18, y + 2, 1, 16], INNER_LIGHT),
        ]
    }

    // The 1px outline of a whole cell, as [x, y, w, h] strips (the alert's red).
    pub fn outline_strips(x: i32, y: i32) -> [[i32; 4]; 4] {
        [
            [x, y, CELL, 1],
            [x, y + CELL - 1, CELL, 1],
            [x, y + 1, 1, CELL - 2],
            [x + CELL - 1, y + 1, 1, CELL - 2],
        ]
    }

    // The durability bar over the icon's bottom edge, as vanilla's item bar sits:
    // (background, filled part). The fill is capped at the full width.
    pub fn bar_rects(slot: &SlotBox, fill_px: u32) -> (Rect, Rect) {
        let x = slot.item_x + 2;
        let y = slot.item_y + 13;
        (
            Rect {
                x,
                y,
                w: BAR_W as i32,
                h: 2,
            },
            Rect {
                x,
                y,
                w: fill_px.min(BAR_W) as i32,
                h: 2,
            },
        )
    }

    // Where a durability number starts: to the left of the column, right-aligned
    // 2px off its frame; inside the slot's bottom right when there is no room.
    pub fn number_x(border_x: i32, frame_x: i32, text_width: i32) -> i32 {
        let outside = border_x - 2 - text_width;
        if outside >= 1 {
            outside
        } else {
            frame_x + CELL - 1 - text_width
        }
    }
}

#[cfg(test)]
mod tests {
    use super::alert;
    use super::armor::*;
    use super::layout::*;
    use super::*;

    // Compare 0xRRGGBB colours channel by channel, allowing 1 off for rounding.
    fn near(got: u32, want: u32) -> bool {
        (0..3).all(|i| {
            let shift = i * 8;
            let a = ((got >> shift) & 0xFF) as i32;
            let b = ((want >> shift) & 0xFF) as i32;
            (a - b).abs() <= 1
        })
    }

    #[test]
    fn settings_use_their_defaults() {
        assert!(!enabled(None, false));
        assert!(enabled(None, true));
        assert!(enabled(Some("true"), false));
        assert!(!enabled(Some("false"), true));
        assert_eq!(stored(true), "true");
        assert_eq!(
            button_label("Durability numbers", false),
            "Durability numbers: OFF"
        );
        assert_eq!(notice_text(true), "Durability numbers: ON");
        assert_eq!(notice_text(false), "Durability numbers: OFF");
    }

    #[test]
    fn display_orders_run_boots_to_helmet_and_back() {
        assert_eq!(DISPLAY_ORDER[0], ArmorSlot::Boots);
        assert_eq!(DISPLAY_ORDER[3], ArmorSlot::Helmet);
        assert_eq!(COLUMN_ORDER[0], ArmorSlot::Helmet);
        assert_eq!(COLUMN_ORDER[3], ArmorSlot::Boots);
    }

    #[test]
    fn unbreakable_items_have_no_durability() {
        assert_eq!(Durability::new(0, 0), None);
    }

    #[test]
    fn damage_is_clamped_to_the_maximum() {
        let d = Durability::new(150, 100).unwrap();
        assert_eq!(d.damage, 100);
        assert_eq!(d.remaining(), 0);
    }

    #[test]
    fn remaining_and_numbers() {
        let d = Durability::new(10, 100).unwrap();
        assert_eq!(d.remaining(), 90);
        assert_eq!(d.numbers(), "90");
    }

    #[test]
    fn only_undamaged_items_are_full() {
        assert!(Durability::new(0, 80).unwrap().is_full());
        assert!(!Durability::new(1, 80).unwrap().is_full());
    }

    #[test]
    fn low_means_under_ten_percent() {
        assert!(!Durability::new(0, 100).unwrap().is_low());
        assert!(!Durability::new(90, 100).unwrap().is_low()); // exactly 10%
        assert!(Durability::new(91, 100).unwrap().is_low());
        assert!(Durability::new(100, 100).unwrap().is_low());
        assert!(Durability::new(79, 80).unwrap().is_low()); // a leather tunic at 1/80
    }

    #[test]
    fn bar_width_rounds_like_vanilla() {
        assert_eq!(Durability::new(0, 100).unwrap().bar_pixels(13), 13);
        assert_eq!(Durability::new(50, 100).unwrap().bar_pixels(13), 7);
        assert_eq!(Durability::new(100, 100).unwrap().bar_pixels(13), 0);
    }

    #[test]
    fn colour_runs_green_yellow_red() {
        assert!(near(bar_color(1.0), 0x00FF00));
        assert!(near(bar_color(0.5), 0xFFFF00));
        assert!(near(bar_color(0.0), 0xFF0000));
        assert!(near(bar_color(0.25), 0xFF8000));
        // out of range inputs are clamped, not wrapped
        assert!(near(bar_color(2.0), 0x00FF00));
        assert!(near(bar_color(-1.0), 0xFF0000));
    }

    #[test]
    fn column_sits_the_gap_left_of_the_offhand() {
        let s = armor_column(960, 540, true).unwrap();
        let b = column_border(&s);
        assert_eq!(b.x + b.w, 353);
        assert_eq!(b.w, 22);
        assert_eq!(360 - (b.x + b.w), COLUMN_GAP); // 360 is where the off-hand sprite starts
        assert_eq!(s[0].frame.x, 332);
        assert_eq!(COLUMN_GAP, 7); // the same 7px that separates the off hand from the hotbar
    }

    #[test]
    fn left_handed_column_uses_the_hotbar_edge() {
        let s = armor_column(960, 540, false).unwrap();
        let b = column_border(&s);
        assert_eq!(389 - (b.x + b.w), COLUMN_GAP); // 389 is the hotbar's left edge
    }

    #[test]
    fn column_stacks_top_to_bottom_and_ends_on_the_hotbar_row() {
        let s = armor_column(960, 540, true).unwrap();
        for i in 1..4 {
            assert_eq!(s[i].frame.y - s[i - 1].frame.y, CELL);
            assert_eq!(s[i].frame.x, s[0].frame.x);
        }
        assert_eq!(s[3].frame.y, 540 - 21); // the hotbar's cells start at h - 21
        let b = column_border(&s);
        assert_eq!(b.y + b.h, 540); // the frame's last row is h - 1, as the hotbar's
        assert_eq!(b.h, 82);
    }

    #[test]
    fn icons_line_up_with_the_hotbar_items() {
        let s = armor_column(960, 540, true).unwrap();
        assert_eq!(s[3].item_y, 540 - 16 - 3);
        assert_eq!(s[3].item_x, s[3].frame.x + 2);
    }

    #[test]
    fn column_is_skipped_only_when_it_leaves_the_screen() {
        assert!(armor_column(298, 200, true).is_some()); // exactly fits
        assert!(armor_column(296, 200, true).is_none());
        assert!(armor_column(426, 240, true).is_some()); // 720p at scale 3
        assert!(armor_column(320, 180, true).is_some()); // 720p at scale 4
        assert!(armor_column(960, 80, true).is_none()); // too short
    }

    #[test]
    fn slot_look_covers_the_cell_exactly_once() {
        // 16x16 interior + 76 outer-ring pixels + 68 inner-ring pixels = 20x20
        let mut seen = [[0u8; 20]; 20];
        for (r, _) in cell_strips(0, 0) {
            for yy in r[1]..r[1] + r[3] {
                for xx in r[0]..r[0] + r[2] {
                    seen[yy as usize][xx as usize] += 1;
                }
            }
        }
        assert!(seen.iter().flatten().all(|&n| n == 1));
    }

    #[test]
    fn frame_strips_trace_the_border() {
        let s = armor_column(960, 540, true).unwrap();
        let strips = border_strips(column_border(&s));
        let pixels: i32 = strips.iter().map(|r| r[2] * r[3]).sum();
        assert_eq!(pixels, 22 * 2 + 80 * 2); // top, bottom, left, right
    }

    #[test]
    fn alert_outline_traces_the_cell() {
        let pixels: i32 = outline_strips(5, 7).iter().map(|r| r[2] * r[3]).sum();
        assert_eq!(pixels, 76); // 20 * 4 - 4 corners counted twice
    }

    #[test]
    fn durability_bar_geometry() {
        let s = armor_column(960, 540, true).unwrap();
        let (bg, fill) = bar_rects(&s[0], 7);
        assert_eq!(
            (bg.x, bg.y, bg.w, bg.h),
            (s[0].item_x + 2, s[0].item_y + 13, 13, 2)
        );
        assert_eq!(fill.w, 7);
        let (_, capped) = bar_rects(&s[0], 99);
        assert_eq!(capped.w, 13);
    }

    #[test]
    fn number_goes_left_of_the_column_or_inside_when_cramped() {
        assert_eq!(number_x(308, 309, 12), 294);
        assert_eq!(number_x(10, 11, 18), 12); // no room outside: inside, right-aligned
    }

    #[test]
    fn alert_pulse_stays_in_range_and_peaks() {
        let mut t = 0.0;
        while t < 5.0 {
            let (tint, outline) = alert::alphas(t);
            assert!((alert::TINT_MIN..=alert::TINT_MAX).contains(&tint));
            assert!((alert::OUTLINE_MIN..=alert::OUTLINE_MAX).contains(&outline));
            t += 0.037;
        }
        let (tint, outline) = alert::alphas(alert::PERIOD / 4.0);
        assert!(tint >= alert::TINT_MAX - 1 && outline >= alert::OUTLINE_MAX - 1);
        let (tint, outline) = alert::alphas(alert::PERIOD * 0.75);
        assert!(tint <= alert::TINT_MIN + 1 && outline <= alert::OUTLINE_MIN + 1);
        assert_eq!(alert::argb(0x80), 0x80FF_3030);
    }

    #[test]
    fn notice_holds_then_fades() {
        assert_eq!(notice_alpha(0.0), Some(255));
        assert_eq!(notice_alpha(NOTICE_HOLD), Some(255));
        let half = notice_alpha(NOTICE_HOLD + NOTICE_FADE / 2.0).unwrap();
        assert!((127..=128).contains(&half));
        let early = notice_alpha(1.2).unwrap();
        let late = notice_alpha(1.7).unwrap();
        assert!(early > late);
        assert_eq!(notice_alpha(NOTICE_HOLD + NOTICE_FADE + 0.01), None);
        assert_eq!(notice_alpha(5.0), None);
    }

    #[test]
    fn a_shown_notice_is_current_at_full_alpha() {
        show_notice(true);
        let (text, alpha) = current_notice().unwrap();
        assert_eq!(text, "Durability numbers: ON");
        assert_eq!(alpha, 255);
    }

    #[test]
    fn flags_are_cached_until_invalidated() {
        use std::cell::Cell;
        invalidate_flags();
        let calls = Cell::new(0);
        let read = |key: &str| {
            calls.set(calls.get() + 1);
            if key == NUMBERS {
                Some("true".to_string())
            } else {
                None
            }
        };
        let first = cached_flags(&read);
        assert_eq!(
            first,
            Flags {
                numbers: true,
                alerts: true
            }
        );
        let after_first = calls.get();
        assert_eq!(cached_flags(&read), first);
        assert_eq!(calls.get(), after_first); // answered from the cache
        invalidate_flags();
        cached_flags(&read);
        assert!(calls.get() > after_first); // read again
    }
}

pub fn reset() {
    invalidate_flags();
    *lock(&NOTICE) = None;
}
