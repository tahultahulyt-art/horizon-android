//! Screen-space presentation of a client-only world location marker.
use crate::hud::{HudAtlas, HudBatch};
/// Project a camera-relative point; never mirror markers behind the camera.
pub fn project(point: glam::Vec3, matrix: glam::Mat4, width: f32, height: f32) -> Option<[f32; 2]> {
    let p = matrix * point.extend(1.0);
    if !p.is_finite() || p.w <= 0.05 {
        return None;
    }
    let p = p.truncate() / p.w;
    (p.x.abs() <= 1.0 && p.y.abs() <= 1.0)
        .then_some([(p.x * 0.5 + 0.5) * width, (0.5 - p.y * 0.5) * height])
}

pub fn label(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    name: &str,
    distance: f64,
    rgb: u32,
    [x, y]: [f32; 2],
) {
    let distance = if distance >= 1000.0 {
        format!("{:.1} km", distance / 1000.0)
    } else {
        format!("{distance:.0} m")
    };
    let text = format!("{}  {}", name, distance);
    let width = HudBatch::text_width(atlas, &text) + 12.0;
    let x = (x - width / 2.0).clamp(2.0, (batch.width - width - 2.0).max(2.0));
    let y = (y - 14.0).clamp(2.0, (batch.height - 18.0).max(2.0));
    let [_, r, g, b] = rgb.to_be_bytes();
    let color = crate::hud::rgba(r, g, b, 255);
    batch.fill(atlas, x, y, width, 16.0, crate::hud::rgba(12, 14, 22, 190));
    batch.fill(atlas, x, y, 2.0, 16.0, color);
    batch.text(atlas, &text, x + 6.0, y + 4.0, color, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn markers_project_in_front_and_never_behind() {
        let matrix = glam::Mat4::perspective_rh(90.0f32.to_radians(), 2.0, 0.05, 1024.0);
        assert_eq!(
            project(glam::Vec3::new(0.0, 0.0, -10.0), matrix, 800.0, 400.0),
            Some([400.0, 200.0])
        );
        assert!(project(glam::Vec3::new(0.0, 0.0, 10.0), matrix, 800.0, 400.0).is_none());
        assert!(project(glam::Vec3::new(100.0, 0.0, -10.0), matrix, 800.0, 400.0).is_none());
    }
}
