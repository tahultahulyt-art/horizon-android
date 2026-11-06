// Copyright (c) 2026 CayMac
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// Hook helper: native_key
pub(super) fn run(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::mod_crosshair_indicator_glue::on_key(game, key, pressed);
}
