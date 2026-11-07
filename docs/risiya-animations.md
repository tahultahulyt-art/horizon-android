# Risiya Animations

An original built-in, optional first-person cosmetic feature for the Risiya app only. It is not Hold My Items, Fresh Animations, EMF/ETF or a CEM loader. No files or code from those projects are included. Rahul's standard Horizon app never enables it or shows these controls.

Default **Off**. Choose Off / Lite / Full / Reduced motion in launcher Settings > Quick controls > Risiya Animations, or in-game gear > Quick controls. Launcher choices apply on next Play. In-game changes apply immediately and save with the app's touch settings. Existing preferences and custom touch layouts are preserved. Off bypasses the extra transforms and equip easing.

- Lite: original class-specific hand/item swing, short equip easing, jump/landing settle and manual inspect. No continuous idle or walking sway.
- Full: adds gentle idle breathing and movement sway, with faster sprint rhythm.
- Reduced motion: smaller optional motion, no constant sway.
- Inspect: a separate touch button below the top controls while in first person with animations enabled. Turns the held item toward view, then back. No repeated auto-inspect. Attack, use, slot/item changes, menus, focus loss or camera change cancel it immediately. It never delays a gameplay action.

Sword, tool, light, generic vanilla item and empty-hand curves are original. Both handednesses are mirrored. Maps, bow/crossbow, trident, shield, fishing rod, brush, spyglass, spears and unknown modded namespaces retain stock transforms. Active use states keep stock item poses, including food/drink. Mob models, third-person player animation, chests, boats and capes are unchanged.

The code changes visual matrices only. It does not change reach, damage, cooldown, mining progress, attack/use packets, input ownership of attack/use controls or server rules. It adds no full-screen shader, high-resolution textures, asset downloads or per-frame file I/O. Motion state covers two hands; the remembered item name only allocates when it changes. This is a low-overhead design, not a measured phone benchmark. Motion blur remains the user's separate preference.

`tools/test_risiya_animations.py` tests pure state/matrices and emits fixture poses. Fixture storyboard/GIF is not a game screenshot or evidence of Android performance. Android compilation, release audit, game-render verification and physical phone testing are separate checks. If performance or comfort is poor, turn the feature Off. No compatibility/FPS guarantee.
