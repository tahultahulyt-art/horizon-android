// Copyright (c) 2026 CayMac
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// Hook helper: native_settings_click
pub(super) fn run(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    crate::game::mod_crosshair_indicator_glue::on_click(game, mod_id, button);
}
