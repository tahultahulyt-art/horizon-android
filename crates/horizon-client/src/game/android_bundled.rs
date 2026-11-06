//! Fixed MIT-licensed Android mod bundle. No downloaded code or phone compiler.

use super::*;

use super::native_hooks::*;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/runtime.rs"]
pub(super) mod official_zoom;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_key.rs"]
mod zoom_key;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_wheel.rs"]
mod zoom_wheel;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_zoom.rs"]
mod zoom_zoom;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_list.rs"]
mod zoom_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_settings.rs"]
mod zoom_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_click.rs"]
mod zoom_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/runtime.rs"]
pub(super) mod official_minimap;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_list.rs"]
mod minimap_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_settings.rs"]
mod minimap_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_click.rs"]
mod minimap_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_minimap.rs"]
mod minimap_minimap;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/runtime.rs"]
pub(super) mod official_recipes;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/glue.rs"]
pub(super) mod official_recipes_glue;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_list.rs"]
mod recipes_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_settings.rs"]
mod recipes_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_click.rs"]
mod recipes_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_frame.rs"]
mod recipes_frame;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_event.rs"]
mod recipes_event;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/runtime.rs"]
pub(super) mod smartf3;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/native.rs"]
pub(super) mod smartf3_native;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_command.rs"]
mod smartf3_command;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_list.rs"]
mod smartf3_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_settings.rs"]
mod smartf3_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_click.rs"]
mod smartf3_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/armorhud/runtime.rs"]
pub(super) mod armorhud;

#[cfg(target_os = "android")]
#[path = "android_bundled/armorhud/settings.rs"]
mod armorhud_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/armorhud/hud.rs"]
mod armorhud_hud;

#[cfg(target_os = "android")]
#[path = "android_bundled/armorhud/key.rs"]
mod armorhud_key;

#[cfg(target_os = "android")]
#[path = "android_bundled/armorhud/click.rs"]
mod armorhud_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/armorhud/list.rs"]
mod armorhud_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/crosshair_indicator/runtime.rs"]
pub(super) mod mod_crosshair_indicator;

#[cfg(target_os = "android")]
#[path = "android_bundled/crosshair_indicator/settings.rs"]
mod crosshair_indicator_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/crosshair_indicator/hud.rs"]
mod crosshair_indicator_hud;

#[cfg(target_os = "android")]
#[path = "android_bundled/crosshair_indicator/glue.rs"]
pub(super) mod mod_crosshair_indicator_glue;

#[cfg(target_os = "android")]
#[path = "android_bundled/crosshair_indicator/key.rs"]
mod crosshair_indicator_key;

#[cfg(target_os = "android")]
#[path = "android_bundled/crosshair_indicator/click.rs"]
mod crosshair_indicator_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/crosshair_indicator/list.rs"]
mod crosshair_indicator_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/status_effect_hud/runtime.rs"]
pub(super) mod mod_status_effect_hud;

#[cfg(target_os = "android")]
#[path = "android_bundled/status_effect_hud/settings.rs"]
mod status_effect_hud_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/status_effect_hud/hud.rs"]
mod status_effect_hud_hud;

#[cfg(target_os = "android")]
#[path = "android_bundled/status_effect_hud/glue.rs"]
pub(super) mod mod_status_effect_hud_glue;

#[cfg(target_os = "android")]
#[path = "android_bundled/status_effect_hud/click.rs"]
mod status_effect_hud_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/status_effect_hud/list.rs"]
mod status_effect_hud_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/keystrokes/runtime.rs"]
pub(super) mod official_keystrokes;

#[cfg(target_os = "android")]
#[path = "android_bundled/keystrokes/settings.rs"]
mod keystrokes_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/keystrokes/hud.rs"]
mod keystrokes_hud;

#[cfg(target_os = "android")]
#[path = "android_bundled/keystrokes/click.rs"]
mod keystrokes_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/keystrokes/list.rs"]
mod keystrokes_list;

pub(super) fn native_key(game: &mut Game<'_>, key: &str, pressed: bool) {
    let _ = (&game, &key, &pressed);
    #[cfg(target_os = "android")]
    {
        zoom_key::run(&mut *game, key, pressed);
        armorhud_key::run(game, key, pressed);
        crosshair_indicator_key::run(game, key, pressed);
    }
}

pub(super) fn native_wheel(game: &mut Game<'_>, steps: f32, claimed: &mut bool) {
    let _ = (&game, &steps, &claimed);
    #[cfg(target_os = "android")]
    {
        zoom_wheel::run(&*game, steps, &mut *claimed);
    }
}

pub(super) fn native_zoom(game: &Game<'_>, zoom: &mut Option<NativeZoom>) {
    let _ = (&game, &zoom);
    #[cfg(target_os = "android")]
    {
        zoom_zoom::run(game, &mut *zoom);
    }
}

pub(super) fn native_mods(game: &mut Game<'_>, mods: &mut Vec<NativeMod>) {
    let _ = (&game, &mods);
    #[cfg(target_os = "android")]
    {
        zoom_list::run(&mut *mods);
        minimap_list::run(&mut *mods);
        recipes_list::run(&mut *mods);
        smartf3_list::run(&mut *mods);
        armorhud_list::run(&mut *mods);
        crosshair_indicator_list::run(&mut *mods);
        status_effect_hud_list::run(&mut *mods);
        keystrokes_list::run(&mut *mods);
    }
}

pub(super) fn native_settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    let _ = (&game, &mod_id, &buttons);
    #[cfg(target_os = "android")]
    {
        zoom_settings::run(&mut *game, mod_id, &mut *buttons);
        minimap_settings::run(&mut *game, mod_id, &mut *buttons);
        recipes_settings::run(&mut *game, mod_id, &mut *buttons);
        smartf3_settings::run(&mut *game, mod_id, &mut *buttons);
        armorhud_settings::run(&mut *game, mod_id, &mut *buttons);
        crosshair_indicator_settings::run(&mut *game, mod_id, &mut *buttons);
        status_effect_hud_settings::run(&mut *game, mod_id, &mut *buttons);
        keystrokes_settings::run(&mut *game, mod_id, &mut *buttons);
    }
}

pub(super) fn native_settings_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    let _ = (&game, &mod_id, &button);
    #[cfg(target_os = "android")]
    {
        zoom_click::run(&mut *game, mod_id, button);
        minimap_click::run(&mut *game, mod_id, button);
        recipes_click::run(&mut *game, mod_id, button);
        smartf3_click::run(&mut *game, mod_id, button);
        armorhud_click::run(&mut *game, mod_id, button);
        crosshair_indicator_click::run(&mut *game, mod_id, button);
        status_effect_hud_click::run(&mut *game, mod_id, button);
        keystrokes_click::run(&mut *game, mod_id, button);
    }
}

pub(super) fn native_minimap(game: &Game<'_>, minimap: &mut Option<NativeMinimap>) {
    let _ = (&game, &minimap);
    #[cfg(target_os = "android")]
    {
        minimap_minimap::run(game, &mut *minimap);
    }
}

pub(super) fn native_frame(game: &mut Game<'_>, elapsed: Duration) {
    let _ = (&game, &elapsed);
    #[cfg(target_os = "android")]
    {
        recipes_frame::run(&mut *game);
    }
}

pub(super) fn native_panel_event(game: &mut Game<'_>, mod_id: &str, event: &NativePanelEvent) {
    let _ = (&game, &mod_id, &event);
    #[cfg(target_os = "android")]
    {
        recipes_event::run(&mut *game, mod_id, event);
    }
}

pub(super) fn native_command(game: &mut Game<'_>, command: &str, ran: &mut bool) {
    let _ = (&game, &command, &ran);
    #[cfg(target_os = "android")]
    {
        smartf3_command::run(&mut *game, command, &mut *ran);
    }
}

#[cfg(target_os = "android")]
pub(super) fn reset() {
    official_zoom::reset();
    official_minimap::reset();
    official_recipes::set_viewer(None);
    smartf3_native::reset();
    armorhud::reset();
    official_keystrokes::reset();
    mod_status_effect_hud::reset();
}

pub(super) fn native_hud(
    game: &mut Game<'_>,
    batch: &mut HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    let _ = (&game, &batch, &corners);
    #[cfg(target_os = "android")]
    {
        armorhud_hud::run(game, batch, corners);
        crosshair_indicator_hud::run(game, batch, corners);
        status_effect_hud_hud::run(game, batch, corners);
        keystrokes_hud::run(game, batch, corners);
    }
}
