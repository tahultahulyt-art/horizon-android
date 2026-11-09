//! Client-only retained location markers. Empty/off mods do no geometry work.
use super::Game;
use horizon_client::hud::HudBatch;

#[cfg(not(feature = "mods"))]
impl Game<'_> {
    pub(super) fn draw_waypoint_labels(&self, _: &mut HudBatch, _: glam::DVec3, _: glam::Mat4) {}
    pub(super) fn waypoint_beams(
        &self,
        _: glam::DVec3,
        _: f64,
        _: &mut Vec<horizon_client::mesher::Vertex>,
    ) {
    }
}

#[cfg(feature = "mods")]
impl Game<'_> {
    pub(super) fn draw_waypoint_labels(
        &self,
        batch: &mut HudBatch,
        camera: glam::DVec3,
        matrix: glam::Mat4,
    ) {
        if self.title.is_some()
            || self.ui.hide_gui
            || !matches!(self.screen, super::Screen::Playing)
        {
            return;
        }
        // Lazily compute the identity only if a mod actually retained a marker.
        let mut world = None;
        let atlas = &self.gui.atlas;
        self.mods.mods.visit_waypoints(|p| {
            let id = world.get_or_insert_with(|| self.mod_world_id());
            if &p.world_id != id || p.dimension != self.dim.id() {
                return;
            }
            let position = glam::DVec3::from_array(p.position);
            let distance = self.player.as_ref().map_or(0.0, |player| {
                (position
                    - glam::DVec3::new(player.position.x, player.position.y, player.position.z))
                .length()
            });
            let Some([x, y]) = horizon_client_render::waypoint_labels::project(
                (position + glam::DVec3::Y * 1.2 - camera).as_vec3(),
                matrix,
                batch.width,
                batch.height,
            ) else {
                return;
            };
            horizon_client_render::waypoint_labels::label(
                batch,
                atlas,
                &p.name,
                distance,
                p.color,
                [x, y],
            );
        });
    }
    /// Real world-space beacon geometry. Respects terrain depth and F1; no real blocks/entities are created.
    pub(super) fn waypoint_beams(
        &self,
        camera: glam::DVec3,
        alpha: f64,
        out: &mut Vec<horizon_client::mesher::Vertex>,
    ) {
        if self.title.is_some() || self.ui.hide_gui {
            return;
        }
        let Some(sprite) = self
            .gui
            .atlas
            .sprites
            .get("entity/beacon/beacon_beam")
            .copied()
        else {
            return;
        };
        let mut world = None;
        let time = self.world.game_time.rem_euclid(40) as f32 + alpha as f32;
        self.mods.mods.visit_waypoints(|p| {
            if !p.beam || p.dimension != self.dim.id() {
                return;
            }
            let id = world.get_or_insert_with(|| self.mod_world_id());
            if &p.world_id != id {
                return;
            }
            let offset = glam::DVec3::from_array(p.position) - camera;
            // Bounded cost, no terrain/chunk lookup or per-frame disk I/O.
            if offset.length_squared() > 1024.0 * 1024.0 {
                return;
            }
            let origin = (offset - glam::DVec3::new(0.5, 0.0, 0.5)).as_vec3();
            horizon_client::block_entity_render::beam(
                glam::Mat4::from_translation(origin),
                horizon_client::block_entity_render::BeamShape {
                    scale: 1.0,
                    time,
                    start: 0,
                    height: 256,
                    color: p.color,
                    solid_radius: 0.10,
                    glow_radius: 0.16,
                },
                sprite,
                out,
            );
        });
    }
}
