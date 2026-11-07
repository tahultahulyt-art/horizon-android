//! Attack presses stay immediate. Only camera dragging waits for a small slop.
#[derive(Clone, Copy)]
pub struct AttackDrag {
    start: [f32; 2],
    active: bool,
}
impl AttackDrag {
    pub fn new(start: [f32; 2]) -> Self {
        Self {
            start,
            active: false,
        }
    }
    pub fn motion(&mut self, point: [f32; 2], previous: [f32; 2], aspect: f32) -> [f32; 2] {
        if point.into_iter().chain(previous).any(|v| !v.is_finite())
            || !aspect.is_finite()
            || aspect <= 0.0
        {
            return [0.0; 2];
        }
        if self.active {
            return [point[0] - previous[0], point[1] - previous[1]];
        }
        let offset = [point[0] - self.start[0], point[1] - self.start[1]];
        let distance = (offset[0] * aspect).hypot(offset[1]);
        const SLOP: f32 = 0.008;
        if distance <= SLOP {
            return [0.0; 2];
        }
        self.active = true;
        // Discard the slop itself. Never replay tap jitter as a sudden jump.
        let fraction = (distance - SLOP) / distance;
        offset.map(|v| v * fraction)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upward_tap_jitter_never_turns() {
        let mut d = AttackDrag::new([0.5, 0.5]);
        let mut prev = [0.5, 0.5];
        for point in [[0.501, 0.497], [0.499, 0.495], [0.5, 0.499]] {
            assert_eq!(d.motion(point, prev, 2.0), [0.0; 2]);
            prev = point;
        }
    }
    #[test]
    fn threshold_crossing_discards_slop_not_attack_drag() {
        let mut d = AttackDrag::new([0.5, 0.5]);
        let v = d.motion([0.5, 0.49], [0.5, 0.5], 2.0);
        assert!((v[1] + 0.002).abs() < 0.000001);
        assert_eq!(v[0], 0.0);
        let v = d.motion([0.51, 0.48], [0.5, 0.49], 2.0);
        assert!((v[0] - 0.01).abs() < 0.000001);
        assert!((v[1] + 0.01).abs() < 0.000001);
    }
    #[test]
    fn threshold_is_equal_physical_distance_on_each_axis() {
        let mut x = AttackDrag::new([0.5, 0.5]);
        let mut y = x;
        assert_eq!(x.motion([0.503, 0.5], [0.5, 0.5], 2.0), [0.0; 2]);
        assert_eq!(y.motion([0.5, 0.506], [0.5, 0.5], 2.0), [0.0; 2]);
        assert!(x.motion([0.505, 0.5], [0.503, 0.5], 2.0)[0] > 0.0);
        assert!(y.motion([0.5, 0.51], [0.5, 0.506], 2.0)[1] > 0.0);
    }
    #[test]
    fn new_tap_does_not_inherit_drag() {
        let mut d = AttackDrag::new([0.5, 0.5]);
        d.motion([0.6, 0.5], [0.5, 0.5], 2.0);
        let mut next = AttackDrag::new([0.6, 0.5]);
        assert_eq!(next.motion([0.6, 0.497], [0.6, 0.5], 2.0), [0.0; 2]);
    }
}
