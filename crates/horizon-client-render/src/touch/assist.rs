//! Bounded edge assistance, not a retained target or full lock.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub point: [f64; 3],
    pub distance: f64,
    pub error: [f32; 2],
}
/// Nearest point in the box to the view ray at its centre depth. A view already
/// inside the body produces no correction, so it never snaps to the centre/head.
pub fn candidate(
    eye: [f64; 3],
    yaw: f32,
    pitch: f32,
    lo: [f64; 3],
    hi: [f64; 3],
    reach: f64,
) -> Option<Candidate> {
    if eye.into_iter().chain(lo).chain(hi).any(|v| !v.is_finite())
        || !yaw.is_finite()
        || !pitch.is_finite()
        || !reach.is_finite()
    {
        return None;
    }
    if (0..3).any(|a| lo[a] > hi[a]) {
        return None;
    }
    let (y, p) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    let dir = [-y.sin() * p.cos(), -p.sin(), y.cos() * p.cos()];
    let centre: [f64; 3] = std::array::from_fn(|a| (lo[a] + hi[a]) * 0.5 - eye[a]);
    let depth: f64 = (0..3).map(|a| centre[a] * dir[a]).sum();
    if depth <= 0.0 {
        return None;
    }
    let point: [f64; 3] = std::array::from_fn(|a| (eye[a] + dir[a] * depth).clamp(lo[a], hi[a]));
    let offset: [f64; 3] = std::array::from_fn(|a| point[a] - eye[a]);
    let distance = offset.iter().map(|v| v * v).sum::<f64>().sqrt();
    if distance < 0.001 || distance > reach {
        return None;
    }
    let target_yaw = (-offset[0]).atan2(offset[2]).to_degrees();
    let target_pitch = (-offset[1] / distance).clamp(-1.0, 1.0).asin().to_degrees();
    let yaw_error = (target_yaw - f64::from(yaw) + 180.0).rem_euclid(360.0) - 180.0;
    let error = [yaw_error as f32, (target_pitch - f64::from(pitch)) as f32];
    (error[0].hypot(error[1]) <= 2.5).then_some(Candidate {
        point,
        distance,
        error,
    })
}
/// A correction requires fresh finger movement and is at most 8 degrees/second,
/// 30% of edge error and half of the current finger turn. Fast deliberate swipes
/// and input against the correction disengage it. No input means zero output.
pub fn correction(
    error: [f32; 2],
    finger: [f32; 2],
    seconds: f32,
    strength: f32,
    moved: bool,
) -> [f32; 2] {
    if !moved
        || !seconds.is_finite()
        || seconds <= 0.0
        || seconds > 0.1
        || !strength.is_finite()
        || error.into_iter().chain(finger).any(|v| !v.is_finite())
    {
        return [0.0; 2];
    }
    let length = error[0].hypot(error[1]);
    let motion = finger[0].hypot(finger[1]);
    if length < 0.0001 || length > 2.5 || motion < 0.0001 || motion / seconds > 150.0 {
        return [0.0; 2];
    }
    let limit = 8.0 * seconds;
    std::array::from_fn(|axis| {
        // A horizontal drag must never manufacture vertical camera movement.
        // Each axis is assisted only in the user's current direction on that axis.
        if error[axis] * finger[axis] <= 0.0 {
            return 0.0;
        }
        let amount = (error[axis].abs() * strength.clamp(0.0, 0.3))
            .min(limit * (error[axis].abs() / length))
            .min(finger[axis].abs() * 0.5);
        error[axis].signum() * amount
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn on_body_does_not_snap_to_centre() {
        let c = candidate(
            [0.0, 1.6, 0.0],
            0.0,
            0.0,
            [-0.3, 0.0, 2.0],
            [0.3, 1.8, 2.6],
            3.0,
        )
        .unwrap();
        assert!(c.error[0].hypot(c.error[1]) < 0.0001);
    }
    #[test]
    fn near_edge_qualifies_but_far_side_and_behind_do_not() {
        assert!(
            candidate(
                [0.0, 1.6, 0.0],
                8.0,
                0.0,
                [-0.3, 0.0, 2.0],
                [0.3, 1.8, 2.6],
                3.0
            )
            .is_some()
        );
        assert!(
            candidate(
                [0.0, 1.6, 0.0],
                30.0,
                0.0,
                [-0.3, 0.0, 2.0],
                [0.3, 1.8, 2.6],
                3.0
            )
            .is_none()
        );
        assert!(
            candidate(
                [0.0, 1.6, 0.0],
                180.0,
                0.0,
                [-0.3, 0.0, 2.0],
                [0.3, 1.8, 2.6],
                3.0
            )
            .is_none()
        );
    }
    #[test]
    fn distance_and_invalid_data_rejected() {
        assert!(candidate([0.0; 3], 0.0, 0.0, [-0.3, -1.0, 7.0], [0.3, 1.0, 7.6], 3.0).is_none());
        assert!(candidate([f64::NAN; 3], 0.0, 0.0, [-0.3; 3], [0.3; 3], 3.0).is_none());
    }
    #[test]
    fn no_fresh_input_or_disabled_strength_never_moves() {
        for e in [[1.0, 1.0], [-1.0, -1.0]] {
            assert_eq!(correction(e, [0.0; 2], 0.016, 0.3, true), [0.0; 2]);
            assert_eq!(correction(e, [0.1; 2], 0.016, 0.3, false), [0.0; 2]);
            assert_eq!(correction(e, [0.1; 2], 0.016, 0.0, true), [0.0; 2]);
        }
    }
    #[test]
    fn correction_is_capped_does_not_overshoot_and_user_can_leave() {
        for hz in [30, 60, 120, 240] {
            let dt = 1.0 / hz as f32;
            let c = correction([2.0, 0.0], [1.0, 0.0], dt, 0.3, true);
            assert!(c[0] <= 8.0 * dt + 0.00001 && c[0] <= 0.5 && c[0] <= 0.6);
            assert_eq!(c[1], 0.0);
        }
        assert_eq!(
            correction([1.0, 0.0], [-0.1, 0.0], 0.016, 0.3, true),
            [0.0; 2]
        );
        assert_eq!(
            correction([1.0, 0.0], [10.0, 0.0], 0.016, 0.3, true),
            [0.0; 2]
        );
    }
    #[test]
    fn horizontal_drag_never_adds_vertical_and_axes_release_independently() {
        let c = correction([1.0, -1.0], [0.2, 0.0], 0.016, 0.3, true);
        assert!(c[0] > 0.0);
        assert_eq!(c[1], 0.0);
        let c = correction([1.0, -1.0], [0.2, 0.1], 0.016, 0.3, true);
        assert!(c[0] > 0.0);
        assert_eq!(c[1], 0.0);
        let c = correction([1.0, -1.0], [0.0, -0.1], 0.016, 0.3, true);
        assert_eq!(c[0], 0.0);
        assert!(c[1] < 0.0);
    }
    #[test]
    fn yaw_wrap_does_not_turn_the_long_way() {
        let c = candidate(
            [0.0, 1.0, 0.0],
            179.0,
            0.0,
            [-0.3, 0.0, -2.6],
            [0.3, 1.8, -2.0],
            3.0,
        )
        .unwrap();
        assert!(c.error[0].abs() < 0.001);
    }
}
