"""Android must never compile/register or silently install the removed built-ins."""
from pathlib import Path
import unittest
ROOT = Path(__file__).resolve().parents[1]

def read(path):
    return (ROOT / path).read_text()

class LoaderOnly(unittest.TestCase):
    def test_no_compiled_bundle(self):
        base = ROOT / 'crates/horizon-client/src/game'
        self.assertFalse((base / 'android_bundled.rs').exists())
        self.assertFalse((base / 'android_bundled').exists())
        for path in [ROOT/'crates/horizon-client/src/game.rs', *base.glob('*.rs')]:
            self.assertNotIn('android_bundled', path.read_text(), str(path))
        game = read('crates/horizon-client/src/game.rs')
        self.assertNotIn('smartf3_hud', game)
        self.assertNotIn('android_reset_bundled_mods', game)

    def test_launcher_is_loader_only(self):
        pages = read('crates/horizon-client/src/android/launcher_pages.rs')
        self.assertNotIn('ToggleBundled', pages)
        self.assertNotIn('ANDROID_BUNDLED', pages)
        self.assertNotIn('Built-in mods below', pages)
        for label in ['Kusobo Loader v1', 'Install sample', 'Export template', 'Import file']:
            self.assertIn(label, pages)
        catalog = read('crates/horizon-client/src/android/catalog.rs')
        self.assertNotIn('bundled_enabled', catalog)
        self.assertIn('let on = horizon_client::mod_list::ModList::load(&game);', catalog)
        self.assertIn('on.enabled.contains(id)', catalog)
        self.assertNotIn('ANDROID_BUNDLED', read('crates/horizon-client-render/src/mod_list.rs'))

    def test_no_android_automatic_migration(self):
        src = read('crates/horizon-client/src/game/moved_extras.rs')
        self.assertIn('#[cfg(target_os = "android")]\n        pub(in crate::game) fn start_moved_extras(&mut self) {}', src)
        self.assertIn('#[cfg(not(target_os = "android"))]\n        pub(in crate::game) fn start_moved_extras', src)

    def test_keep_user_controls_and_loader_assets(self):
        pages = read('crates/horizon-client/src/android/launcher_pages.rs')
        for name in ['Action::QuickControls', 'Action::EditControls']:
            self.assertIn(name, pages)
        for file in ['kusobo_coordinates.hmod', 'kusobo-template.zip']:
            self.assertTrue((ROOT/'packaging/android/kusobo'/file).is_file())
        self.assertIn('include_bytes!', read('crates/horizon-client/src/android/kusobo.rs'))

if __name__ == '__main__':
    unittest.main()
