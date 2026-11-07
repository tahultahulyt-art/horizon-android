//! Android adaptation of Freecam+ (MIT, umbx-dot). No dynamic native-mod loader.

mod android {
    use super::super::{
        Game, Screen, freecam_runtime as freecam, native_hooks::NativeMod, perspective::CameraType,
    };
    use winit::keyboard::KeyCode;
    const MOVES: [KeyCode; 7] = [
        KeyCode::KeyW,
        KeyCode::KeyS,
        KeyCode::KeyA,
        KeyCode::KeyD,
        KeyCode::Space,
        KeyCode::ShiftLeft,
        KeyCode::ControlLeft,
    ];
    const VIEWS: [CameraType; 3] = [
        CameraType::FirstPerson,
        CameraType::ThirdPersonBack,
        CameraType::ThirdPersonFront,
    ];
    fn available() -> bool {
        cfg!(target_os = "android") && option_env!("HORIZON_ANDROID_VARIANT") != Some("risiya")
    }
    impl Game<'_> {
        fn mobile_freecam_settings(&self) -> freecam::Settings {
            freecam::settings(|key| self.native_setting(freecam::ID, key))
        }
        fn mobile_freecam_store(&mut self, settings: freecam::Settings) {
            freecam::set_settings(settings);
            for (key, value) in settings.stored() {
                self.native_set_setting(freecam::ID, key, &value);
            }
        }
        pub(in crate::game) fn mobile_freecam_stop(&mut self) {
            if let Some(view) = freecam::stop() {
                self.camera_type = VIEWS[usize::from(view).min(2)];
                for key in MOVES {
                    self.keys.remove(&key);
                }
                freecam::analog([0.0; 2]);
            }
        }
        fn mobile_freecam_toggle(&mut self) {
            if freecam::active() {
                self.mobile_freecam_stop();
                return;
            }
            if !available()
                || !self.native_window()
                || !self.native_on(freecam::ID)
                || self.title.is_some()
                || self.sleeping.is_some()
                || self.riding.is_some()
                || self.using.is_some()
                || self.screen == Screen::Dead
            {
                return;
            }
            let Some(player) = self.player.as_ref() else {
                return;
            };
            let eye = [
                player.position.x,
                player.position.y + f64::from(self.eye),
                player.position.z,
            ];
            let view = VIEWS
                .iter()
                .position(|v| *v == self.camera_type)
                .unwrap_or(0) as u8;
            self.mobile_freecam_settings();
            freecam::start(eye, self.yaw, self.pitch, view);
            for key in MOVES {
                self.keys.remove(&key);
            }
            self.left = false;
            self.right = false;
            self.stop_mining();
            self.using = None;
            self.camera_type = CameraType::ThirdPersonBack;
        }
        pub(in crate::game) fn mobile_freecam_move_key(
            &mut self,
            key: KeyCode,
            down: bool,
        ) -> bool {
            if !freecam::active() {
                return false;
            }
            let Some(index) = MOVES.iter().position(|k| *k == key) else {
                return false;
            };
            freecam::hold(index, down && self.native_playing());
            self.keys.remove(&key);
            true
        }
        pub(in crate::game) fn mobile_freecam_key(&mut self, key: &str, down: bool) {
            if available()
                && down
                && self.native_playing()
                && key == self.native_mod_key(freecam::ID, freecam::KEY, freecam::DEFAULT_KEY)
            {
                self.mobile_freecam_toggle();
            }
        }
        pub(in crate::game) fn mobile_freecam_tick(&mut self) {
            if !self.native_window() || !freecam::active() {
                return;
            }
            if self.player.is_none()
                || !self.native_on(freecam::ID)
                || !available()
                || !self.focused
                || self.screen == Screen::Dead
                || self.using.is_some()
                || self.riding.is_some()
                || self.sleeping.is_some()
            {
                self.mobile_freecam_stop();
                return;
            }
            self.camera_type = CameraType::ThirdPersonBack;
            self.left = false;
            self.right = false;
            if !self.native_playing() {
                for i in 0..7 {
                    freecam::hold(i, false);
                }
                freecam::analog([0.0; 2]);
                return;
            }
            let stick = self.pad_move().unwrap_or([0.0; 2]);
            // pad_move is [left/right, forward/back] in Java's input coordinates.
            freecam::analog([f64::from(stick[0]), f64::from(stick[1])]);
            let noclip = self.mobile_freecam_settings().noclip;
            freecam::tick(|[x, y, z]| {
                !noclip
                    && self
                        .res
                        .facts()
                        .get(self.world.block(x, y, z))
                        .full_collision
            });
        }
        pub(in crate::game) fn mobile_freecam_wheel(&mut self, steps: f32, claimed: &mut bool) {
            if self.native_on(freecam::ID)
                && let Some(settings) = freecam::wheel(steps)
            {
                self.mobile_freecam_store(settings);
                *claimed = true;
            }
        }
        pub(in crate::game) fn mobile_freecam_list(&mut self, mods: &mut Vec<NativeMod>) {
            if !available() {
                return;
            }
            if !self.settings.mods.enabled.contains(freecam::ID)
                && !self.settings.mods.disabled.contains(freecam::ID)
            {
                self.settings.mods.disabled.insert(freecam::ID.into());
                if let Some(dir) = Self::options_path().parent() {
                    self.settings.mods.save(dir);
                }
            }
            mods.push(NativeMod{id:freecam::ID.into(),title:"Freecam+ 1.0.1 Android port".into(),description:"Optional camera flight. Options > Mods > Freecam+ settings: Camera ON/OFF. Touch stick moves, Jump rises, Sneak descends. Body still obeys gravity, damage and server corrections. Only already-loaded terrain is visible.".into(),keys:vec![[freecam::KEY.into(),"Toggle freecam".into(),freecam::DEFAULT_KEY.into()]]});
        }
        pub(in crate::game) fn mobile_freecam_page(&self, id: &str, buttons: &mut Vec<String>) {
            if id == freecam::ID && available() {
                buttons.push(format!(
                    "Camera: {}",
                    if freecam::active() { "ON" } else { "OFF" }
                ));
                buttons.extend(self.mobile_freecam_settings().labels());
            }
        }
        pub(in crate::game) fn mobile_freecam_click(&mut self, id: &str, button: usize) {
            if id != freecam::ID || !available() {
                return;
            }
            if button == 0 {
                self.mobile_freecam_toggle();
            } else {
                let mut settings = self.mobile_freecam_settings();
                settings.click(button - 1);
                self.mobile_freecam_store(settings);
            }
        }
    }
}
