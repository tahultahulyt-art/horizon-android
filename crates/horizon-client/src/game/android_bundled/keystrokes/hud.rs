// Copyright (c) 2026 Ludovic Marie and the Horizon contributors
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// The keys in their corner, as the extra drew them; the debug screen takes the
// top left.
pub(super) fn run(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::official_keystrokes as m;
    // Turned off in Options > Mods: nothing until the next build leaves it out.
    if !game.native_on(m::ID) {
        return;
    }
    use horizon_client::extras::Corner;
    let settings = {
        let game = &*game;
        m::settings(|key| game.native_setting(m::ID, key))
    };
    let corner = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ][settings.corner];
    if !game.native_corner_shown(corner) {
        return;
    }
    let state = game.pad.state;
    let mut down = |name: &str| {
        let (held, label) = game.native_mapping(name).unwrap_or_default();
        use horizon_client::gamepad::Button;
        let touch = game.pad.touch.state();
        let extra = match name {
            "key.forward" => state.left[1] > 0.2 || touch.left[1] > 0.2,
            "key.back" => state.left[1] < -0.2 || touch.left[1] < -0.2,
            "key.left" => state.left[0] < -0.2 || touch.left[0] < -0.2,
            "key.right" => state.left[0] > 0.2 || touch.left[0] > 0.2,
            "key.jump" => state.held(Button::South) || touch.held(Button::South),
            "key.attack" => state.triggers[1] > 0.5 || touch.triggers[1] > 0.5,
            "key.use" => state.triggers[0] > 0.5 || touch.triggers[0] > 0.5,
            _ => false,
        };
        (held || extra, label)
    };
    let view = m::View {
        moves: m::MOVES.map(&mut down),
        jump: down("key.jump").0,
        attack: down("key.attack").0,
        use_: down("key.use").0,
        cps: None,
    };
    let view = m::View {
        cps: settings.cps.then(|| game.native_clicks()),
        ..view
    };
    let (w, h) = m::SIZE;
    let (x, y) = corners.place(corner, w, h);
    let game = &*game;
    for shape in m::shapes(&view, &|text| game.native_text_width(text)) {
        match shape {
            m::Shape::Fill([dx, dy, w, h], argb) => {
                game.native_fill(batch, [x + dx, y + dy, w, h], argb)
            }
            m::Shape::Text(text, [dx, dy], rgb, shadow) => {
                game.native_text(batch, &text, [x + dx, y + dy], rgb, shadow)
            }
        }
    }
}
