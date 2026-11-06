// Copyright (c) 2026 Thomas-Slot
// SPDX-License-Identifier: MIT; see LICENSE. Android static-bundle adaptation.
// The mod's row in Options > Mods, with its rebindable key.
pub(super) fn run(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::armorhud as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("Armor HUD {}", m::VERSION),
        description: "Shows equipped armor and its durability beside the hotbar.".into(),
        keys: vec![[
            m::KEY_NAME.into(),
            "Toggle durability numbers".into(),
            m::DEFAULT_KEY.into(),
        ]],
    });
}
