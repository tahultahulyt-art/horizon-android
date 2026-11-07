# Android: loader-only mods

All eight previously compiled Android mods have been removed: Zoom, Minimap,
Recipes, SmartF3, Armor HUD, Crosshair Indicator, Status Effect HUD and Keystrokes.
Their runtime source, native dispatch hooks and launcher enable rows are gone.
The standard debug screen remains. This does not remove normal game armor,
potions, crosshair, maps, crafting or spyglass behavior.

Kusobo Loader v1 still manages compatible sandboxed `.hmod` / WebAssembly packages.
Import a package, enable it in Installed, then Play. The optional Coordinates
sample and source template remain. Fabric/Forge Java `.jar` mods are not supported.
No replacement HUD packages are included in this update.

Old built-in preference entries are left alone. They do not load code and can be
reused if the player later imports a compatible package with the same id.
Android no longer auto-downloads formerly built-in extras from old settings.
Existing imported packages and their on/off choices are preserved.

Launcher Quick controls, the in-game gear panel, the touch editor, the 1000%
sensitivity cap and OpenGL ES mode are unchanged. Install over the existing
Horizon Android Dev app; do not uninstall it or clear its data.

There is no measured FPS claim. Check launch, Options > Mods, local world load,
existing imported packages and the touch editor on the phone after installing.
