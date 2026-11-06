# Opt-in Android HUD ports

Four source ports are compiled into the APK: Armor HUD, Crosshair Indicator,
Status Effect HUD and Keystrokes. They do not load native source from a file.
All are off until enabled on the local Mods page or Options > Mods. Each has
settings in Options > Mods. Existing Zoom, Minimap, Recipes and SmartF3 remain.

- Armor HUD displays worn equipment and optional durability numbers and alerts.
- Crosshair Indicator displays a reach marker and optional attack cooldown.
  It does not change reach or attack behaviour.
- Status Effect HUD displays effects and countdown timers in a chosen corner.
- Keystrokes displays movement/actions and optional clicks per second, including
  held touch/controller movement and actions.

Original package checksums, authors and MIT licenses are recorded in
THIRD_PARTY_NOTICES.md and alongside each module. Touch settings, GLES and the
1000% touch-look limit are unchanged. No loader, shader or quick-settings panel
is introduced by these ports.

Device validation still needs to check small-screen layout, touch highlighting,
actual effect timers/equipped items and toggle/settings persistence after restart.
