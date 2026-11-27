//! Bounded scaling of real finger motion only. No world/entity information.
use super::Settings;

pub(super) fn scale(delta: [f32; 2], aspect: f32, seconds: f32, attacking: bool, s: &Settings) -> [f32; 2] {
    let mut gain = 1.0;
    if s.attack_aim_scaling && attacking {
        gain *= s.attack_aim_scale;
    }
    if s.aim_acceleration && s.aim_acceleration_strength > 0.0 && seconds.is_finite() && seconds > 0.0 {
        // Physical speed in screen heights/second, independent of refresh rate.
        let speed = (delta[0] * aspect).hypot(delta[1]) / seconds;
        let t = ((speed - 0.25) / 2.25).clamp(0.0, 1.0);
        let smooth = t * t * (3.0 - 2.0 * t);
        let curve = 0.65 + 0.85 * smooth;
        gain *= 1.0 + s.aim_acceleration_strength * (curve - 1.0);
    }
    if gain == 1.0 { delta } else { delta.map(|v| v * gain) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_and_neutral_settings_are_bit_identical() {
        let mut s=Settings::default();
        for on in [false,true] {
            s.attack_aim_scaling=on;s.aim_acceleration=on;
            for d in [[0.0,-0.0],[0.001,-0.02],[0.21,0.19]] {
                let out=scale(d,2.2,0.016,true,&s);
                assert_eq!(out.map(f32::to_bits),d.map(f32::to_bits));
            }
        }
    }
    #[test]
    fn attack_scale_is_only_applied_while_attacking() {
        let mut s=Settings::default();s.attack_aim_scaling=true;s.attack_aim_scale=0.5;
        assert_eq!(scale([0.02,-0.04],2.0,0.01,true,&s),[0.01,-0.02]);
        assert_eq!(scale([0.02,-0.04],2.0,0.01,false,&s),[0.02,-0.04]);
        s.attack_aim_scaling=false;
        assert_eq!(scale([0.02,-0.04],2.0,0.01,true,&s),[0.02,-0.04]);
    }
    #[test]
    fn acceleration_is_bounded_direction_preserving_and_sample_rate_independent() {
        let mut s=Settings::default();s.aim_acceleration=true;s.aim_acceleration_strength=1.0;
        let slow=scale([0.001,0.0],2.0,0.02,false,&s)[0]/0.001;
        let fast=scale([0.03,0.0],2.0,0.02,false,&s)[0]/0.03;
        assert!((slow-0.65).abs()<1e-5 && (fast-1.5).abs()<1e-5);
        for speed in [0.01,0.25,0.8,1.5,2.5,10.0] {
            let a=scale([speed*0.01,-speed*0.01],2.0,0.01,false,&s);
            let b=scale([speed*0.02,-speed*0.02],2.0,0.02,false,&s);
            assert!((a[0]*2.0-b[0]).abs()<1e-5);
            assert!(a[0]>=0.0 && a[1]<=0.0);
            assert!((a[0]/(speed*0.01))>=0.64999 && (a[0]/(speed*0.01))<=1.50001);
        }
        assert_eq!(scale([0.0,0.0],2.0,0.01,true,&s),[0.0,0.0]);
    }
    #[test]
    fn settings_old_defaults_roundtrip_and_sanitize() {
        let mut s:Settings=serde_json::from_str("{}").unwrap();
        assert!(!s.attack_aim_scaling && !s.aim_acceleration);
        assert_eq!(s.attack_aim_scale,1.0);assert_eq!(s.aim_acceleration_strength,0.0);
        s.attack_aim_scaling=true;s.attack_aim_scale=0.7;s.aim_acceleration=true;s.aim_acceleration_strength=0.3;
        let r:Settings=serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert!(r.attack_aim_scaling && r.aim_acceleration);
        assert_eq!(r.attack_aim_scale,0.7);assert_eq!(r.aim_acceleration_strength,0.3);
        s.attack_aim_scale=f32::NAN;s.aim_acceleration_strength=f32::INFINITY;s.sanitize();
        assert_eq!(s.attack_aim_scale,1.0);assert_eq!(s.aim_acceleration_strength,0.0);
    }
}
