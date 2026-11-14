//! Target-independent touch smoothing. Optional aim assistance is a separate module.
#[derive(Default)]
pub struct AimStabilizer {
    pending: [f32; 2],
    moved: bool,
}
impl AimStabilizer {
    /// OFF is exactly the existing input path. ON queues only real finger motion.
    pub fn input(&mut self, delta: [f32; 2], enabled: bool) -> [f32; 2] {
        if !enabled {
            self.clear();
            return delta;
        }
        self.moved |= delta.iter().any(|d| d.is_finite() && *d != 0.0);
        for (p, d) in self.pending.iter_mut().zip(delta) {
            if d.is_finite() {
                *p += d;
            }
        }
        [0.0; 2]
    }
    /// Strength (0-1, from the Shield Mode strength slider) scales the low-pass
    /// window from 2ms (nearly raw) to 40ms (heavy glide). Fast swipes use a
    /// shorter window so flicks stay responsive.
    /// Releasing the look finger drains the exact remainder in one frame, no drift.
    pub fn frame(&mut self, seconds: f32, active: bool, enabled: bool, strength: f32) -> [f32; 2] {
        if !enabled || !seconds.is_finite() || seconds > 0.1 {
            self.clear();
            return [0.0; 2];
        }
        let strength = if strength.is_finite() {
            strength.clamp(0.0, 1.0)
        } else {
            0.25
        };
        let speed = self.pending[0].hypot(self.pending[1]);
        let tau = if speed > 3.0 {
            0.001 + 0.011 * strength
        } else {
            0.002 + 0.038 * strength
        };
        let alpha = if !active {
            1.0
        } else {
            1.0 - (-seconds.max(0.0) / tau).exp()
        };
        let out = self.pending.map(|v| v * alpha);
        for (p, o) in self.pending.iter_mut().zip(out) {
            *p -= o;
        }
        out
    }
    pub fn take_motion(&mut self) -> bool {
        std::mem::take(&mut self.moved)
    }
    pub fn clear(&mut self) {
        self.moved = false;
        self.pending = [0.0; 2];
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const MID: f32 = 0.5;
    #[test]
    fn fresh_motion_is_consumed_once_and_cleared_on_pause() {
        let mut s = AimStabilizer::default();
        s.input([0.1, 0.0], true);
        assert!(s.take_motion());
        assert!(!s.take_motion());
        s.input([0.0; 2], true);
        assert!(!s.take_motion());
        s.input([0.1, 0.0], true);
        s.frame(0.2, true, true, MID);
        assert!(!s.take_motion());
    }
    #[test]
    fn off_is_exact_and_clears_backlog() {
        let mut s = AimStabilizer::default();
        s.input([1.0, 2.0], true);
        assert_eq!(s.input([3.0, -4.0], false), [3.0, -4.0]);
        assert_eq!(s.frame(0.016, true, true, MID), [0.0; 2]);
    }
    #[test]
    fn release_preserves_total_distance_without_drift() {
        for hz in [30, 60, 90, 120, 240] {
            for strength in [0.0, 0.5, 1.0] {
                let mut s = AimStabilizer::default();
                let mut sum = [0.0; 2];
                for _ in 0..hz {
                    s.input([1.0, -0.5], true);
                    let o = s.frame(1.0 / hz as f32, true, true, strength);
                    sum[0] += o[0];
                    sum[1] += o[1];
                }
                let o = s.frame(1.0 / hz as f32, false, true, strength);
                sum[0] += o[0];
                sum[1] += o[1];
                assert!((sum[0] - hz as f32).abs() < 0.001);
                assert!((sum[1] + hz as f32 * 0.5).abs() < 0.001);
                assert_eq!(s.frame(0.016, false, true, strength), [0.0; 2]);
            }
        }
    }
    #[test]
    fn uneven_frames_and_reversal_preserve_both_axes() {
        let mut s = AimStabilizer::default();
        let mut sum = [0.0_f32; 2];
        let mut expected = [0.0_f32; 2];
        for i in 0..160 {
            let delta = if i < 80 { [0.3, -0.2] } else { [-0.2, 0.1] };
            for axis in 0..2 {
                expected[axis] += delta[axis];
            }
            s.input(delta, true);
            let out = s.frame([0.008, 0.025, 0.014, 0.011][i % 4], true, true, MID);
            for axis in 0..2 {
                sum[axis] += out[axis];
            }
        }
        let out = s.frame(0.016, false, true, MID);
        for axis in 0..2 {
            assert!((sum[axis] + out[axis] - expected[axis]).abs() < 0.001);
        }
    }
    #[test]
    fn explicit_clear_and_invalid_frame_drop_pending_motion() {
        let mut s = AimStabilizer::default();
        s.input([1.0, 2.0], true);
        s.clear();
        assert_eq!(s.frame(0.016, true, true, MID), [0.0; 2]);
        s.input([1.0, 2.0], true);
        assert_eq!(s.frame(f32::NAN, true, true, MID), [0.0; 2]);
        assert_eq!(s.frame(0.016, true, true, MID), [0.0; 2]);
    }
    #[test]
    fn pending_input_does_not_cross_pause() {
        let mut s = AimStabilizer::default();
        s.input([3.0, 2.0], true);
        assert_eq!(s.frame(0.5, true, true, MID), [0.0; 2]);
        assert_eq!(s.frame(0.016, true, true, MID), [0.0; 2]);
    }
    #[test]
    fn no_input_never_moves_camera() {
        let mut s = AimStabilizer::default();
        for _ in 0..500 {
            assert_eq!(s.frame(0.016, true, true, MID), [0.0; 2]);
        }
    }
    #[test]
    fn gentle_input_is_filtered_fast_swipe_is_responsive() {
        let mut s = AimStabilizer::default();
        s.input([1.0, 0.0], true);
        let slow = s.frame(0.008, true, true, MID)[0];
        assert!(slow > 0.1 && slow < 0.9);
        s.clear();
        s.input([10.0, 0.0], true);
        assert!(s.frame(0.008, true, true, MID)[0] > 5.0);
    }
    #[test]
    fn higher_strength_smooths_more_and_nan_falls_back() {
        let out = |strength: f32| {
            let mut s = AimStabilizer::default();
            s.input([1.0, 0.0], true);
            s.frame(0.008, true, true, strength)[0]
        };
        assert!(out(0.1) > out(0.9));
        assert_eq!(out(f32::NAN), out(0.25));
        assert_eq!(out(2.0), out(1.0));
        assert_eq!(out(-1.0), out(0.0));
    }
}
