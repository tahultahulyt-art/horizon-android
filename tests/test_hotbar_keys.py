"""Source-level regression checks. These do not replace an in-game test."""
from pathlib import Path
import unittest
ROOT = Path(__file__).resolve().parents[1]
REMOTE = (ROOT / 'crates/horizon-client/src/game/remote.rs').read_text()
class HotbarKeys(unittest.TestCase):
    def test_rb_have_no_main_inventory_fallback(self):
        body = REMOTE.split('fn rocket_launch_tick', 1)[1].split('/// CK is K', 1)[0]
        self.assertNotIn('tweak_click', body)
        self.assertNotIn('inventory_find', body)
        self.assertNotIn('0..36', body)
    def test_lookup_is_only_nine_hotbar_slots(self):
        body = REMOTE.split('fn hotbar_find', 1)[1].split('fn hotbar_select', 1)[0]
        self.assertIn('(0..9).find', body)
    def test_ck_requires_all_three_items(self):
        body = REMOTE.split('fn anchor_combo_key_press', 1)[1].split('fn anchor_combo_allowed', 1)[0]
        self.assertIn('.map(|id| self.hotbar_find(id))', body)
        self.assertIn('let Some(slots) = slots else', body)
        self.assertLess(body.index('let Some(slots)'), body.index('self.anchor_combo = AnchorCombo'))
    def test_ck_uses_only_place_and_charge(self):
        body = REMOTE.split('fn anchor_combo_tick', 1)[1].split('fn auto_mlg_prepare', 1)[0]
        self.assertEqual(body.count('self.anchor_use_at('), 2)
        helper = REMOTE.split('fn anchor_use_at', 1)[1].split('fn anchor_combo_tick', 1)[0]
        self.assertEqual(helper.count('self.remote_use_block(Hand::Main)'), 1)
        self.assertIn('self.forced_hit = previous', helper)
        totem = body.split('AnchorStage::SelectTotem =>', 1)[1]
        self.assertNotIn('remote_use', totem)
        self.assertNotIn('anchor_use_at', totem)
        self.assertIn('self.selected = self.anchor_combo.slots[2]', totem)
        self.assertNotIn('tweak_click', body)
        self.assertNotIn('UseItemOn', body)
    def test_ck_has_no_crosshair_recheck_or_use_delay_wait(self):
        body = REMOTE.split('fn anchor_combo_tick', 1)[1].split('fn auto_mlg_prepare', 1)[0]
        self.assertIn('self.anchor_combo.ticks > 10', body)
        self.assertIn('self.anchor_takes(target, self.stack_in(self.selected))', body)
        self.assertIn('self.anchor_combo.stage = AnchorStage::SelectTotem', body)
        self.assertNotIn('self.use_delay', body)
        self.assertNotIn('hit.pos == target', body)
        self.assertNotIn('self.pick()', body)
        self.assertNotIn('charges', body)
    def test_ck_captures_both_hits_at_key_press(self):
        body = REMOTE.split('fn anchor_combo_key_press', 1)[1].split('fn anchor_combo_allowed', 1)[0]
        self.assertIn('place_hit: Some(hit)', body)
        self.assertIn('charge_hit: Some(super::Hit { pos: target, face, point })', body)
    def test_ck_selection_precedes_use_in_separate_stage(self):
        body = REMOTE.split('fn anchor_combo_tick', 1)[1].split('fn auto_mlg_prepare', 1)[0]
        stage = body.split('AnchorStage::SelectAnchor =>', 1)[1].split('AnchorStage::Place =>', 1)[0]
        self.assertIn('self.selected = self.anchor_combo.slots[0]', stage)
        self.assertNotIn('remote_use', stage)
    def test_ck_touch_button_is_momentary(self):
        layout = (ROOT / 'crates/horizon-client-render/src/touch/layout.rs').read_text()
        self.assertIn('ck.caption = "CK"', layout)
        self.assertIn('CustomButton::new("key.keyboard.k"', layout)
        self.assertNotIn('ck.toggle = true', layout)
if __name__ == '__main__': unittest.main()
