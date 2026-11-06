// Copyright (c) 2026 Thomas-Slot
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// The numbers keybind: flip the setting and show a notification above the hotbar.
pub(super) fn run(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::armorhud as m;
    if !pressed || !game.native_on(m::ID) || !game.native_playing() {
        return;
    }
    let wanted = game.native_mod_key(m::ID, m::KEY_NAME, m::DEFAULT_KEY);
    if key != wanted.as_str() {
        return;
    }
    let on = m::enabled(game.native_setting(m::ID, m::NUMBERS).as_deref(), false);
    game.native_set_setting(m::ID, m::NUMBERS, m::stored(!on));
    m::invalidate_flags();
    m::show_notice(!on);
}
