// Copyright (c) 2026 Thomas-Slot
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// A click on a settings button: flip that toggle and keep it.
pub(super) fn run(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::armorhud as m;
    if mod_id != m::ID {
        return;
    }
    let toggles = m::TOGGLES;
    let Some(&(key, _, default)) = toggles.get(button) else {
        return;
    };
    let on = m::enabled(game.native_setting(m::ID, key).as_deref(), default);
    game.native_set_setting(m::ID, key, m::stored(!on));
    m::invalidate_flags();
}
