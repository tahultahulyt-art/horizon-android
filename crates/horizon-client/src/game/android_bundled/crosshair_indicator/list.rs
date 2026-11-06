// Copyright (c) 2026 CayMac
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// Hook helper: native_mods
pub(super) fn run(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::mod_crosshair_indicator_glue::on_list(mods);
}
