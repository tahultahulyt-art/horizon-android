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
    def test_ck_requires_anchor_and_glowstone_but_not_the_totem(self):
        body = REMOTE.split('fn anchor_combo_key_press', 1)[1].split('fn anchor_combo_allowed', 1)[0]
        self.assertIn('ANCHOR_ITEMS[..2]', body)
        self.assertIn('let totem = self.hotbar_find(ANCHOR_ITEMS[2]);', body)
        self.assertIn('let Some(slots) = slots else', body)
        self.assertLess(body.index('let Some(slots)'), body.index('self.anchor_combo = AnchorCombo'))
    def test_ck_uses_only_place_and_charge(self):
        body = REMOTE.split('fn anchor_combo_tick', 1)[1].split('fn auto_mlg_prepare', 1)[0]
        self.assertEqual(body.count('self.anchor_use_at('), 2)
        helper = REMOTE.split('fn anchor_use_at', 1)[1].split('fn anchor_combo_allowed', 1)[0]
        self.assertEqual(helper.count('self.remote_use_block(Hand::Main)'), 1)
        self.assertIn('self.forced_hit = previous', helper)
        totem = body.split('AnchorStage::SelectTotem =>', 1)[1]
        self.assertNotIn('remote_use', totem)
        self.assertNotIn('anchor_use_at', totem)
        self.assertIn('self.anchor_combo_end()', totem)
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
    def test_ck_selection_sends_carried_item_at_once(self):
        helper = REMOTE.split('fn anchor_select', 1)[1].split('fn anchor_combo_end', 1)[0]
        self.assertIn('self.selected = slot', helper)
        self.assertIn('r.carried.replace(slot)', helper)
        self.assertIn('self.send(S::SetCarriedItem(play::SetCarriedItem { slot: slot as i16 }))', helper)
    def test_ck_one_visible_swap_per_tick(self):
        body = REMOTE.split('fn anchor_combo_tick', 1)[1].split('fn auto_mlg_prepare', 1)[0]
        self.assertIn('for _ in 0..6', body)
        anchor = body.split('AnchorStage::SelectAnchor =>', 1)[1].split('AnchorStage::Place =>', 1)[0]
        self.assertIn('self.anchor_select(self.anchor_combo.slots[0])', anchor)
        self.assertNotIn('remote_use', anchor)
        glow = body.split('AnchorStage::SelectGlowstone =>', 1)[1].split('AnchorStage::Charge =>', 1)[0]
        self.assertIn('self.anchor_select(self.anchor_combo.slots[1])', glow)
        # Each hotbar swap is the last step of its tick, so the selection
        # renders on the hotbar before the next one: anchor, then glowstone,
        # then the totem the sequence ends on.
        self.assertIn('break;', anchor.split('self.anchor_select', 1)[1])
        self.assertIn('break;', glow.split('self.anchor_select', 1)[1])
        # The all-three-items recheck happens only before the first swap: the
        # place spends its own stack, so later stages require just the two.
        self.assertEqual(body.count('(0..2).all(|i| self.anchor_combo_has(i))'), 1)
    def test_ck_attack_taps_cannot_abort(self):
        guard = REMOTE.split('fn anchor_combo_running', 1)[1].split('/// One hotbar swap per tick', 1)[0]
        self.assertNotIn('self.right', guard)
        self.assertNotIn('self.left', guard)
        self.assertNotIn('hands_busy', guard)
        self.assertIn('Screen::Playing', guard)
        self.assertIn('dead_or_dying', guard)
        body = REMOTE.split('fn anchor_combo_tick', 1)[1].split('fn auto_mlg_prepare', 1)[0]
        self.assertIn('self.anchor_combo_running()', body)
        self.assertNotIn('anchor_combo_allowed()', body)
    def test_ck_every_end_lands_on_the_totem(self):
        end = REMOTE.split('fn anchor_combo_end', 1)[1].split('fn anchor_combo_tick', 1)[0]
        self.assertIn('AnchorStage::SelectAnchor', end)
        self.assertIn('self.anchor_combo_totem_slot()', end)
        self.assertIn('self.anchor_select(slot)', end)
        self.assertNotIn('remote_use', end)
        self.assertNotIn('anchor_use_at', end)
        body = REMOTE.split('fn anchor_combo_tick', 1)[1].split('fn auto_mlg_prepare', 1)[0]
        # The guard, both mid-sequence failures and the finished sequence.
        self.assertGreaterEqual(body.count('self.anchor_combo_end()'), 4)
    def test_ck_runs_without_a_totem(self):
        body = REMOTE.split('fn anchor_combo_tick', 1)[1].split('fn auto_mlg_prepare', 1)[0]
        self.assertNotIn('anchor_combo_has(2)', body)
        self.assertNotIn('slots[2]', REMOTE.split('fn anchor_combo_key_press', 1)[1].split('fn auto_mlg_prepare', 1)[0])
        self.assertIn('Option<usize>', REMOTE.split('pub(super) struct AnchorCombo', 1)[1].split('}', 1)[0])
    def test_ck_touch_button_is_momentary(self):
        layout = (ROOT / 'crates/horizon-client-render/src/touch/layout.rs').read_text()
        self.assertIn('ck.caption = "CK"', layout)
        self.assertIn('CustomButton::new("key.keyboard.k"', layout)
        self.assertNotIn('ck.toggle = true', layout)
if __name__ == '__main__': unittest.main()
