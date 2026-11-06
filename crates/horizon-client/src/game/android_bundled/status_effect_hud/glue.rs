// Copyright (c) 2026 CayMac
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// Glue between Horizon's `Game` and `mod_status_effect_hud` runtime state.
use crate::game::Game;
use crate::game::mod_status_effect_hud as m;
use crate::game::native_hooks::NativeMod;
use horizon_client::{extras::Corner, extras_hud::Corners, hud::HudBatch};

struct EffectRow {
    id: &'static str,
    title: String,
    timer: String,
    timer_color: u32,
    bar_color: u32,
    progress: f32,
}

pub(super) fn on_hud(game: &mut Game<'_>, batch: &mut HudBatch, corners: &mut Corners) {
    if !game.native_on(m::ID) || game.potions.effects.list.is_empty() {
        return;
    }
    let s = {
        let g = &*game;
        m::settings(|k| g.native_setting(m::ID, k))
    };
    if !s.enabled {
        return;
    }

    let corner = match s.position % 4 {
        0 => Corner::TopRight,
        1 => Corner::TopLeft,
        2 => Corner::BottomLeft,
        _ => Corner::BottomRight,
    };
    if !game.native_corner_shown(corner) {
        return;
    }

    let rate = game.clock.rate();
    let ticks = game.ticks;
    let mut rows = Vec::with_capacity(game.potions.effects.list.len());
    let mut max_text_w: f32 = 0.0;

    for e in &game.potions.effects.list {
        let base_name = game
            .pack
            .names
            .display_get(&format!("effect.minecraft.{}", e.id))
            .unwrap_or_else(|| e.id.replace('_', " "));
        let rom = m::roman_amplifier(e.amplifier);
        let title = if s.style == 0 {
            if rom.is_empty() {
                base_name
            } else {
                format!("{base_name} {rom}")
            }
        } else {
            rom.to_owned()
        };

        let (timer, timer_color) = m::format_effect_timer(e.duration, rate, ticks);
        let bar_color = horizon_client::brewing::effect(e.id).map_or(0x0055_FF55, |d| d.color);
        let progress = m::update_and_compute_progress(e.id, e.duration);

        let line_w = if title.is_empty() {
            game.native_text_width(&timer)
        } else if s.style == 0 {
            game.native_text_width(&title)
                .max(game.native_text_width(&timer))
        } else {
            game.native_text_width(&format!("{title} {timer}"))
        };
        if line_w > max_text_w {
            max_text_w = line_w;
        }

        rows.push(EffectRow {
            id: e.id,
            title,
            timer,
            timer_color,
            bar_color,
            progress,
        });
    }

    let row_h = if s.style == 0 { 24.0 } else { 20.0 };
    let box_w = 24.0 + max_text_w + 6.0;
    let box_h = (rows.len() as f32 * row_h) + 2.0;
    let (x, y) = corners.place(corner, box_w, box_h);

    if s.background {
        game.native_fill(batch, [x, y, box_w, box_h], m::BACKGROUND);
    }

    for (i, row) in rows.iter().enumerate() {
        let ry = y + 2.0 + (i as f32 * row_h);
        let icon_y = if s.style == 0 { ry + 1.0 } else { ry };

        batch.sprite_scaled(
            &game.gui.atlas,
            &format!("mob_effect/{}", row.id),
            x + 3.0,
            icon_y,
            18.0,
            18.0,
            u32::MAX,
        );

        let tx = x + 24.0;
        if s.style == 0 {
            game.native_text(batch, &row.title, [tx, ry + 1.0], m::COLOR_NAME, true);
            game.native_text(batch, &row.timer, [tx, ry + 10.0], row.timer_color, true);
        } else if row.title.is_empty() {
            game.native_text(batch, &row.timer, [tx, ry + 5.0], row.timer_color, true);
        } else {
            let combined = format!("{} {}", row.title, row.timer);
            game.native_text(batch, &combined, [tx, ry + 5.0], row.timer_color, true);
        }

        if s.show_bars {
            let bar_x = x + 3.0;
            let bar_y = ry + row_h - 3.0;
            let bar_w = (box_w - 6.0).max(16.0);
            game.native_fill(batch, [bar_x, bar_y, bar_w, 2.0], m::BAR_TRACK_BG);
            let fill_w = (bar_w * row.progress).clamp(0.0, bar_w);
            if fill_w > 0.5 {
                let argb = 0xFF00_0000 | (row.bar_color & 0x00FF_FFFF);
                game.native_fill(batch, [bar_x, bar_y, fill_w, 2.0], argb);
            }
        }
    }
}

pub(super) fn on_list(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}

pub(super) fn on_settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id != m::ID {
        return;
    }
    let g = &*game;
    let settings = m::settings(|key| g.native_setting(m::ID, key));
    buttons.extend(settings.labels());
}

pub(super) fn on_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    if mod_id != m::ID {
        return;
    }
    let mut settings = {
        let g = &*game;
        m::settings(|key| g.native_setting(m::ID, key))
    };
    settings.click(button);
    m::set_settings(settings);
    for (key, value) in settings.stored() {
        game.native_set_setting(m::ID, key, &value);
    }
}
