//! Explicit optional aim assistance. No retained identity, attack or packet writes.
use super::Game;
use horizon_client::touch::assist::{self, Candidate};
impl Game<'_> {
    pub(super) fn touch_assist_correction(&self, finger: [f32; 2], seconds: f32) -> [f32; 2] {
        if !self.pad.touch.crosshair() || self.spectator() || self.remote_camera_id().is_some() {
            return [0.0; 2];
        }
        let Some(eye) = self.eye_position() else {
            return [0.0; 2];
        };
        let yaw = self.yaw + finger[0];
        let pitch = (self.pitch + finger[1]).clamp(-90.0, 90.0);
        let reach = self.entity_reach.min(3.0);
        let mut best: Option<Candidate> = None;
        let mut offer = |bounds: horizon_core::geometry::Aabb| {
            let lo = bounds.min_position();
            let hi = bounds.max_position();
            let Some(candidate) = assist::candidate(
                eye,
                yaw,
                pitch,
                [lo.x, lo.y, lo.z],
                [hi.x, hi.y, hi.z],
                reach,
            ) else {
                return;
            };
            // Visibility is checked toward the candidate's actual body point, not
            // the central ray. An edge around a wall does not reveal a hidden player.
            let dir = std::array::from_fn(|a| (candidate.point[a] - eye[a]) / candidate.distance);
            if self.clip_ray(eye, dir, candidate.distance).is_some() {
                return;
            }
            let error = candidate.error[0].hypot(candidate.error[1]);
            if best.is_none_or(|b| error < b.error[0].hypot(b.error[1])) {
                best = Some(candidate);
            }
        };
        for m in &self.mobs {
            if m.kind.id == "minecraft:player"
                && !m.removed
                && m.alive()
                && !m.extra.spectator
                && !m.is_invisible()
                && Some(m.id) != self.riding
                && Some(m.id) != self.remote_camera_id()
            {
                offer(m.bounds());
            }
        }
        if let Some(host) = &self.lan {
            for guest in host.hub.guests.iter().filter(|g| {
                host.hub.here(g)
                    && g.loaded
                    && !g.dead
                    && !g.spectator()
                    && g.effects.get("invisibility").is_none()
            }) {
                offer(guest.bounds());
            }
        }
        best.map_or([0.0; 2], |b| {
            assist::correction(
                b.error,
                finger,
                seconds,
                self.pad.touch.settings.aim_assist_strength,
                true,
            )
        })
    }
}
