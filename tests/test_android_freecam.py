"""Freecam port boundaries: no dynamic loader, no Risiya activation or player-action leaks."""
from pathlib import Path
import unittest
ROOT=Path(__file__).resolve().parents[1]
def source(path):return (ROOT/path).read_text()
class FreecamPort(unittest.TestCase):
 def test_license_and_mobile_gate(self):
  glue=source('crates/horizon-client/src/game/mobile_freecam.rs')
  self.assertIn('Copyright (c) 2026 umbx-dot',source('third_party/freecam_plus/LICENSE'))
  self.assertIn('Some("risiya")',glue)
  self.assertIn('self.settings.mods.disabled.insert',glue)
  self.assertIn('Camera: {}',glue)
  self.assertIn('self.pad_move()',glue)
  self.assertIn('freecam::hold(index, down && self.native_playing())',glue)
 def test_camera_and_actions(self):
  p=source('crates/horizon-client/src/game/perspective.rs')
  self.assertIn('super::freecam_runtime::view(partial)',p)
  self.assertIn('super::freecam_runtime::turn(xo, yo)',p)
  game=source('crates/horizon-client/src/game.rs')
  self.assertIn('self.mobile_freecam_move_key(key, pressed)',game)
  self.assertIn('stick.filter(|_| !freecam_runtime::active())',game)
  self.assertIn('self.mobile_freecam_stop();',game)
  self.assertIn('freecam_runtime::reset();',game)
  self.assertIn('if !riding {\n            MotionWorld {',game)
  self.assertIn('if super::freecam_runtime::active()',source('crates/horizon-client/src/game/hand.rs'))
 def test_native_import_still_refused(self):
  self.assertIn('This .hnmod needs a separately compiled client',source('crates/horizon-client/src/android/catalog.rs'))
