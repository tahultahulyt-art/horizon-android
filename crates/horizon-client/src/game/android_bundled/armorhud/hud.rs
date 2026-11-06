// Copyright (c) 2026 Thomas-Slot
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// The HUD: a notification above the hotbar, and the armor column beside it:
// vanilla-looking slots, the worn icons, durability bars, numbers and alerts.
pub(super) fn run(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::armorhud as m;
    let _ = corners;
    if !game.native_on(m::ID) || game.player.is_none() {
        return;
    }
    let width = batch.width as i32;
    let height = batch.height as i32;

    // The notification above the hotbar: full for a second, then fading out.
    if let Some((text, alpha)) = m::current_notice() {
        let x = (width as f32 - game.native_text_width(&text)) / 2.0;
        let y = (height - 68) as f32;
        let color = horizon_client::hud::rgba(0xFF, 0xFF, 0xFF, alpha);
        let line = horizon_client::text::Text::literal(&text);
        batch.text_styled(&game.gui.atlas, &line, x, y, color, true);
    }

    // ASSUMPTION: the off hand is drawn on the left (a right-handed player).
    let Some(slots) = m::layout::armor_column(width, height, !game.settings.main_hand_left) else {
        return;
    };
    let border = m::layout::column_border(&slots);
    let flags = m::cached_flags(|key| game.native_setting(m::ID, key));
    let seconds = m::now_seconds();

    // The black frame round the whole column.
    for [x, y, w, h] in m::layout::border_strips(border) {
        game.native_fill(
            batch,
            [x as f32, y as f32, w as f32, h as f32],
            m::layout::BLACK,
        );
    }
    // Game.armor is head, chest, legs, feet: the column's top-to-bottom order.
    let [head, chest, legs, feet] = &game.armor;
    let worn = [head, chest, legs, feet];
    for (slot, stack) in slots.iter().zip(worn) {
        let (cx, cy) = (slot.frame.x, slot.frame.y);
        for ([x, y, w, h], color) in m::layout::cell_strips(cx, cy) {
            game.native_fill(batch, [x as f32, y as f32, w as f32, h as f32], color);
        }
        let Some(stack) = stack else {
            continue;
        };
        let durability = game
            .res
            .items()
            .get(&stack.item)
            .and_then(|info| m::armor::Durability::new(stack.damage, info.max_damage));
        let low = flags.alerts && durability.is_some_and(|d| d.is_low());
        let (tint, outline) = m::alert::alphas(seconds);
        if low {
            let inside = [(cx + 2) as f32, (cy + 2) as f32, 16.0, 16.0];
            game.native_fill(batch, inside, m::alert::argb(tint));
        }
        game.draw_item_model(batch, stack, slot.item_x as f32, slot.item_y as f32);
        if low {
            for [x, y, w, h] in m::layout::outline_strips(cx, cy) {
                let rect = [x as f32, y as f32, w as f32, h as f32];
                game.native_fill(batch, rect, m::alert::argb(outline));
            }
        }
        let Some(d) = durability else {
            continue;
        };
        // The bar only shows once the piece has been used.
        if !d.is_full() {
            let (bg, fill) = m::layout::bar_rects(slot, d.bar_pixels(m::layout::BAR_W));
            let bg = [bg.x as f32, bg.y as f32, bg.w as f32, bg.h as f32];
            let fill = [fill.x as f32, fill.y as f32, fill.w as f32, fill.h as f32];
            game.native_fill(batch, bg, m::layout::BLACK);
            game.native_fill(batch, fill, 0xFF00_0000 | d.color());
        }
        if flags.numbers {
            let text = d.numbers();
            let tw = game.native_text_width(&text).ceil() as i32;
            let x = m::layout::number_x(border.x, cx, tw) as f32;
            game.native_text(batch, &text, [x, (cy + 6) as f32], d.color(), true);
        }
    }
}
