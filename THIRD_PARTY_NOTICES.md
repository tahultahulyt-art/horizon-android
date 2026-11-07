# Third-party notices

## wgpu-hal

`third_party/wgpu-hal` is [wgpu-hal](https://github.com/gfx-rs/wgpu) 30.0.1 by the gfx-rs
developers, with upstream fixes applied
([gfx-rs/wgpu#10124](https://github.com/gfx-rs/wgpu/pull/10124),
[gfx-rs/wgpu#10714](https://github.com/gfx-rs/wgpu/pull/10714)). It is licensed under MIT or
Apache-2.0; both licenses are in that folder (`LICENSE.MIT`, `LICENSE.APACHE`).

## naga

`third_party/naga` is [naga](https://github.com/gfx-rs/wgpu) 30.0.1 by the gfx-rs developers, with
fixes to its GLSL frontend that shader packs need, each applied in the commit that needed it. It is licensed under MIT or Apache-2.0; both licenses
are in that folder (`LICENSE.MIT`, `LICENSE.APACHE`).

## Pixelify Sans

`crates/horizon-theme/fonts/PixelifySans-*.ttf` is [Pixelify Sans](https://github.com/eifetx/Pixelify-Sans)
by The Pixelify Sans Project Authors, licensed under the SIL Open Font License 1.1
(`crates/horizon-theme/fonts/OFL-PixelifySans.txt`).

## Pixelarticons

`crates/horizon-theme/icons/pixel/*.svg` are icons from
[Pixelarticons](https://github.com/halfmage/pixelarticons) by Gerrit Halfmann, under the MIT
license (`crates/horizon-theme/icons/pixel/LICENSE-pixelarticons.txt`). `star.svg` is Horizon's own, drawn on
the same grid.

## winit

`third_party/winit` is [winit](https://github.com/rust-windowing/winit) 0.30.13,
licensed under Apache-2.0 (see `third_party/winit/LICENSE`). The Android backend
has local mouse and modifier-event fixes described in `third_party/winit/PATCHES.md`.

## AMD FidelityFX SDK (FSR 3.1 frame generation)

`crates/horizon-client-render/src/gfx/frame_gen/optical_flow.wgsl` and `interpolation.wgsl` are
ports to WGSL of the optical flow and frame interpolation shaders of the
[AMD FidelityFX SDK](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK) 1.1.4
(`sdk/include/FidelityFX/gpu/{opticalflow,frameinterpolation}`, at commit
[c6efa6bf7f20](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/tree/c6efa6bf7f2027b3ec94f28578bb5965eabb9e55)),
and `optical_flow.rs` follows the SDK's `sdk/src/components/opticalflow/ffx_opticalflow.cpp`
(its passes, their order and their dispatch sizes). Copyright (C) 2024 Advanced Micro Devices,
Inc., under the MIT license, whose text heads each file. AMD and FidelityFX are AMD's
trademarks; Horizon is not affiliated with AMD.

## Mod loader logos

The launcher shows the logo of the mod loader an instance plays with.

- `launcher/assets/logos/quilt.png` is the Quilt mini icon (`brand/128png/quilt_mini_icon_transparent.png`)
  from [QuiltMC/art](https://github.com/QuiltMC/art), by the Quilt Project, dedicated to the public
  domain under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
- `launcher/assets/logos/neoforge.png` is NeoForged's icon (`art/neoforged_icon_16x16.png`) from
  [neoforged/.github](https://github.com/neoforged/.github/tree/main/art), © the NeoForged team,
  created by Ridanisaurus, under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). It is
  unmodified (the launcher scales it up by whole pixels).
- Fabric's and Forge's icons are not shipped: neither publishes a license for its logo. The launcher
  fetches them from their own sites when an instance needs them
  ([fabric-loader's icon](https://github.com/FabricMC/fabric-loader/blob/master/src/main/resources/assets/fabricloader/icon.png),
  [files.minecraftforge.net's](https://files.minecraftforge.net/)).

Each logo belongs to its project. Showing it says which loader an instance runs; it does not mean
the project endorses Horizon.

## Sign in with Microsoft

The launcher's sign-in buttons follow Microsoft's published branding guidelines for apps that sign in
with Microsoft accounts
([Sign in with Microsoft branding](https://learn.microsoft.com/en-us/entra/identity-platform/howto-add-branding-in-apps)).
The four-square Microsoft symbol in `launcher/src/microsoft.rs` reproduces the squares, positions and
colours of Microsoft's official `ms-symbollockup_mssymbol_19.svg` from that page, unmodified. Microsoft
and its logo are trademarks of the Microsoft group of companies; Horizon is not endorsed by Microsoft.


## Statically bundled Android mods

Zoom, Minimap and Recipes are adapted from [horizon-rust/horizon-mods](https://github.com/horizon-rust/horizon-mods/tree/e3ee0a7c450b28c3ed58fa59a4881ec78c5ee2e0/native),
copyright 2026 Ludovic Marie and the Horizon contributors, MIT.
SmartF3 is adapted from [d3grando/smartf3](https://github.com/d3grando/smartf3),
copyright 2026 d3grando, MIT. Exact revisions and adaptation notes are recorded in
`crates/horizon-client/src/game/android_bundled/PROVENANCE.json`; each bundled
module retains its full MIT LICENSE. Changes provide static Android hook
adapters, opt-in configuration, profile-state reset and compatibility guards.
These are a fixed compiled set, not support for loading arbitrary native mods.

### Additional opt-in Android HUD ports

The fixed Android bundle includes source adaptations of the following native
Horizon mods. They are not Java/Fabric mods and do not load downloaded native
source on the phone. Their original MIT license texts are retained beside each
module in `crates/horizon-client/src/game/android_bundled/`.

- Armor HUD 1.0.0, Copyright (c) 2026 Thomas-Slot.
  Source: https://github.com/Thomas-Slot/horizon-armor-hud
  Package SHA-256: `8bd85e1584f307201bce5de2e60466d603b61d4807803d4ebc02772b4f6b704b`.
- Crosshair Indicator 1.0.0, Copyright (c) 2026 CayMac.
  Source: https://github.com/CayMac97/crosshair-indicator
  Package SHA-256: `96beba1f7c94fca492b552fb12af0d20047dc1799f0801607841d56b12786226`.
- Status Effect HUD 1.0.0, Copyright (c) 2026 CayMac.
  Source: https://github.com/CayMac97/status-effect-hud
  Package SHA-256: `afa30125bb8b402892f1719e550b66d4eb4d7264895f71bb9c98a4caf5c7b432`.
- Keystrokes 2.0.0, Copyright (c) 2026 Ludovic Marie and the Horizon contributors.
  Source: https://github.com/horizon-rust/horizon-mods/tree/main/native/keystrokes
  Package SHA-256: `bed34d2f8049e92c52cd7c91c0c4c8c9d165faaf0183500cf4021d9e04783a7d`.

The package hashes were checked against the Horizon registry. Android adaptations
use the existing opt-in bundled-mod switches, persist the original settings,
respect left-handed armor layout and include touch/controller input in the
Keystrokes display. Original authors have not tested or endorsed this APK.

## Kusobo Coordinates sample

The sandboxed coordinates sample and source template derive from Horizon's MIT-licensed coords-hud SDK example. Copyright (c) 2026 Ludovic Marie and the Horizon contributors. The original MIT license is included in each archive. Kusobo is a management layer, not a replacement or renaming of the Horizon SDK/API.
