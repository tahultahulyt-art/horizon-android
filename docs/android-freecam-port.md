# Freecam+ Android port

Requested optional Freecam+ 1.0.1 source port for the standard development APK only. Original MIT runtime source: https://github.com/umbx-dot/horizon-freecam at c524e2d6d027150d7aad3b2abe68a720cb2065d3. The supplied source-only native bundle's exact desktop commit pin is not used as an Android build pin: its camera helpers are adapted to this fork and compiled into the APK. No arbitrary .hnmod loader or on-phone compiler is added. No changes to VoidCraft content files or Risiya options.

## Use

1. In a world, open Options > Mods and enable Freecam+.
2. Its settings page has Camera: OFF/ON. Set ON then return to the world. Camera flight itself is never persisted.
3. Existing touch joystick/directional controls move the camera. Jump rises, Sneak descends, Sprint accelerates. Swipe look turns the camera, not the player's head. Desktop U is also a rebindable shortcut; an existing custom touch-key button can be assigned U without a physical keyboard.
4. Settings cycle speed and Through walls. A physical mouse wheel changes speed while active.
5. Return to the settings page and turn Camera OFF, disable the mod, or use its toggle key to return to the body's view.

The mod is disabled on first discovery unless the user already explicitly enabled that same ID. Turning it off or leaving/backgrounding the app ends camera flight. The camera restores the former perspective. Riding, sleeping, death and an in-progress item use prevent entry or end flight. Menus pause camera motion; input must be pressed again on return.

## Limits

Only already-loaded terrain is visible; flying the camera does not load distant chunks. Body movement commands and attack/use are blocked while detached, but gravity, damage, mobs and server corrections still apply. This is not invulnerability or a server teleport. Use in a backed-up local world first. Check server rules before using it online. No promise of no-cheat detection, unlimited world view, shader compatibility or low-end performance.

Portable runtime tests and source integration checks do not constitute a physical-phone or in-game camera test. The signed build is experimental until those paths are exercised.
