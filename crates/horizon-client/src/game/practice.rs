//! Original practice opponent. It owns a server-side survival player, not the
//! human's input, and can exist only inside a marked private practice save.
use super::{Dim, Game, Screen, lan::LanHost};
use horizon_client::{
    Result,
    items::ItemStack,
    lan::{Joined, NetEvent, Out, PlayerSender},
    motion::MotionWorld,
};
use horizon_core::geometry::Vec3;
use horizon_protocol::{
    crypto::offline_uuid, packets::configuration::ClientInformation, text::literal,
};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicI32, AtomicUsize, Ordering},
        mpsc::{Receiver, channel},
    },
};

const MARKER: &str = ".risiya-practice-v1";
const BOT_NAME: &str = "PracticePartner";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}
impl Difficulty {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
        }
    }
    fn next(self) -> Self {
        match self {
            Self::Easy => Self::Normal,
            Self::Normal => Self::Hard,
            Self::Hard => Self::Easy,
        }
    }
    fn reaction(self) -> u64 {
        match self {
            Self::Easy => 10,
            Self::Normal => 5,
            Self::Hard => 3,
        }
    }
    fn cooldown(self) -> u64 {
        match self {
            Self::Easy => 25,
            Self::Normal => 17,
            Self::Hard => 14,
        }
    }
    fn speed(self) -> f32 {
        match self {
            Self::Easy => 0.60,
            Self::Normal => 0.82,
            Self::Hard => 1.0,
        }
    }
}

#[derive(Default)]
pub(super) struct Practice {
    allowed: bool,
    running: bool,
    restart: bool,
    pub(super) difficulty: Difficulty,
    bot: Option<Bot>,
    center: Option<[i32; 3]>,
}
struct Bot {
    conn: u32,
    receiver: Receiver<Out>,
    pending: Arc<AtomicUsize>,
    ready: bool,
    ticks: u64,
    last_attack: u64,
    desired: [f32; 3], // strafe, forward, yaw
}
impl Practice {
    pub(super) fn for_world(root: &Path) -> Self {
        let allowed = root.join(MARKER).is_file();
        Self {
            allowed,
            running: allowed,
            ..Self::default()
        }
    }
}
fn permitted(marked: bool, serving: bool, dedicated: bool, listening: bool, humans: usize) -> bool {
    marked && serving && !dedicated && !listening && humans == 1
}
/// A deterministic decision at human-sized reaction intervals. Normal walking
/// inputs are fed through the engine's collision and knockback physics.
fn decision(at: [f64; 3], target: [f64; 3], tick: u64, d: Difficulty) -> [f32; 3] {
    let (dx, dz) = (target[0] - at[0], target[2] - at[2]);
    let distance = dx.hypot(dz);
    let forward: f64 = if distance > 2.55 {
        1.0
    } else if distance < 1.65 {
        -0.45
    } else {
        0.0
    };
    let strafe: f64 = if distance < 6.0 {
        if (tick / 34).is_multiple_of(2) {
            0.42
        } else {
            -0.42
        }
    } else {
        0.0
    };
    let norm = (forward * forward + strafe * strafe).sqrt().max(1.0) as f32;
    [
        strafe as f32 * d.speed() / norm,
        forward as f32 * d.speed() / norm,
        (-dx).atan2(dz).to_degrees() as f32,
    ]
}
impl Game<'_> {
    // This name is for menu presentation only. The server independently checks
    // the private marker and execution role before any world or player change.
    pub(super) fn practice_menu(&self) -> bool {
        self.integrated
            .as_ref()
            .is_some_and(|s| s.world_name == "Risiya Practice")
    }
    pub(super) fn practice_menu_action(&mut self, action: u8) {
        if action == 9 {
            self.practice.difficulty = self.practice.difficulty.next();
        }
        let difficulty = self.practice.difficulty;
        let success = self
            .integrated
            .as_ref()
            .and_then(|server| {
                server.call(move |g| {
                    if !g.practice.allowed || g.serving.is_none() || g.options.dedicated.is_some() {
                        return false;
                    }
                    g.practice.difficulty = difficulty;
                    if action == 3 {
                        g.practice.running = false;
                    } else {
                        g.practice.running = true;
                        g.practice.restart = true;
                    }
                    true
                })
            })
            .unwrap_or(false);
        if !success {
            self.system_message(horizon_client::text::Text::literal(
                "Practice is available only in the private PvP Practice world.",
            ));
        }
        if action != 9 {
            self.screen = Screen::Playing;
            self.pause_focus = None;
            self.capture(true);
        }
    }
    fn practice_remove(&mut self, host: &mut LanHost) {
        if let Some(bot) = self.practice.bot.take() {
            let _ = host
                .hub
                .lan
                .shared
                .events
                .send(NetEvent::Left(bot.conn, "practice stopped".into()));
        }
    }
    pub(super) fn tick_practice(&mut self, host: &mut LanHost) {
        if !self.practice.allowed {
            return;
        }
        let owner = self.serving.as_ref().map(|s| s.owner);
        let Some(owner) = owner else {
            return;
        };
        let humans = host
            .hub
            .guests
            .iter()
            .filter(|g| g.uuid != offline_uuid(BOT_NAME))
            .count();
        if humans == 0 {
            return;
        }
        if !permitted(
            self.practice.allowed,
            self.serving.is_some(),
            self.options.dedicated.is_some(),
            host.hub.lan.listening(),
            humans,
        ) {
            self.practice_remove(host);
            self.practice.running = false;
            return;
        }
        if self.dim != Dim::Overworld {
            return;
        }
        if self.practice.restart || !self.practice.running {
            self.practice_remove(host);
            self.practice.restart = false;
            // Let the normal disconnect path remove the old player first.
            return;
        }
        let Some(oi) = host
            .hub
            .guests
            .iter()
            .position(|g| g.uuid == owner && g.loaded && g.placed)
        else {
            return;
        };
        let owner_dead = host.hub.guests[oi].dead;
        if self.practice.bot.is_none() {
            if owner_dead {
                return;
            }
            if host
                .hub
                .guests
                .iter()
                .any(|g| g.uuid == offline_uuid(BOT_NAME))
            {
                return;
            }
            if self.practice.center.is_none() {
                let p = host.hub.guests[oi].position;
                let center = [p[0].floor() as i32, p[1].ceil() as i32, p[2].floor() as i32];
                if let Err(error) = self.practice_arena(center) {
                    log::warn!("Practice arena waits for its chunks: {error}");
                    return;
                }
                self.practice.center = Some(center);
            }
            let shared = &host.hub.lan.shared;
            let conn = shared.next_id();
            let (tx, receiver) = channel();
            let pending = Arc::new(AtomicUsize::new(0));
            let j = Joined {
                conn,
                uuid: offline_uuid(BOT_NAME),
                name: BOT_NAME.into(),
                properties: Vec::new(),
                info: ClientInformation::new(2),
                brand: "RisiyaPractice".into(),
                sender: PlayerSender::new(tx, pending.clone()),
                latency: Arc::new(AtomicI32::new(0)),
                address: ([127, 0, 0, 1], 0).into(),
            };
            if shared.events.send(NetEvent::Joined(Box::new(j))).is_ok() {
                self.practice.bot = Some(Bot {
                    conn,
                    receiver,
                    pending,
                    ready: false,
                    ticks: 0,
                    last_attack: 0,
                    desired: [0.0; 3],
                });
            }
            return;
        }
        let mut bot = self.practice.bot.take().expect("checked above");
        // The virtual player's output has no network writer and no chunk
        // renderer. Discard it every tick, with exact queue accounting.
        for out in bot.receiver.try_iter() {
            let size = match out {
                Out::Packet(p) => p.len(),
                Out::Packets(p) => p.iter().map(Vec::len).sum(),
                _ => 0,
            };
            let _ = bot
                .pending
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                    Some(n.saturating_sub(size))
                });
        }
        let Some(bi) = host
            .hub
            .guests
            .iter()
            .position(|g| g.conn == bot.conn && g.placed)
        else {
            self.practice.bot = Some(bot);
            return;
        };
        {
            let g = &mut host.hub.guests[bi];
            g.loaded = true;
            g.loading = false;
            g.awaiting_teleport = None;
            g.load_timeout = 0;
            // No chunk-view work is needed for a server-owned opponent.
            g.chunk_queue.clear();
            g.far_queue.clear();
            g.max_unacknowledged = 0;
        }
        if !bot.ready {
            let center = self.practice.center.expect("built before join");
            for (i, offset) in [(oi, -4.0), (bi, 4.0)] {
                self.practice_kit(host, i);
                let p = [
                    f64::from(center[0]) + 0.5,
                    f64::from(center[1]),
                    f64::from(center[2]) + offset + 0.5,
                ];
                if let Some(m) = host.hub.guests[i].motion.as_mut() {
                    m.position = Vec3::new(p[0], p[1], p[2]);
                    m.velocity = Vec3::ZERO;
                }
                host.hub.guests[i].yaw = if i == oi { 0.0 } else { 180.0 };
                host.hub.teleport(i, Some(p));
                host.hub.guests[i].spawn = Some(p);
            }
            bot.ready = true;
            host.hub.system_message(literal("Private sword practice. Pause: Restart / Stop / Bot difficulty. Your attacks and aim stay manual."));
        }
        bot.ticks += 1;
        let dead = host.hub.guests[bi].dead || host.hub.guests[bi].health <= 0.0;
        if dead || owner_dead {
            host.hub.overlay_message(literal(if owner_dead {
                "Round lost. Respawn, then Pause > Restart practice."
            } else {
                "Round won! Pause > Restart practice for a fresh round."
            }));
            self.practice.bot = Some(bot);
            return;
        }
        let target = host.hub.guests[oi].position;
        let at = host.hub.guests[bi].position;
        let target_id = host.hub.guests[oi].entity_id;
        let difficulty = self.practice.difficulty;
        if bot.ticks.is_multiple_of(difficulty.reaction()) {
            bot.desired = decision(at, target, bot.ticks, difficulty);
        }
        // Three seconds to raise a shield or get ready. No player input is
        // injected. Only this guest's own body receives walking input.
        if bot.ticks > 60 {
            let input = bot.desired;
            let grounded = self
                .as_guest(host, bot.conn, |g| {
                    g.yaw = input[2];
                    g.pitch = 0.0;
                    let p = g.player.as_mut()?;
                    p.yaw = input[2];
                    p.pitch = 0.0;
                    let _ = MotionWorld {
                        entities: &[],
                        player: p,
                        scene: &mut g.world,
                        assets: &g.pack.assets,
                        collision_binding: g.collision,
                        resources: g.motion,
                    }
                    .travel_in_air(Vec3::new(
                        f64::from(input[0]) * 0.98,
                        0.0,
                        f64::from(input[1]) * 0.98,
                    ));
                    Some(p.ground.on_ground)
                })
                .flatten();
            if let Some(on_ground) = grounded {
                host.hub.guests[bi].on_ground = on_ground;
            }
            let dist = (target[0] - at[0]).hypot(target[2] - at[2]);
            if dist <= 2.8
                && (target[1] - at[1]).abs() <= 1.5
                && bot.ticks - bot.last_attack >= difficulty.cooldown()
            {
                // Standard server PvP path checks actual reach, cooldown,
                // armor, shield, damage and knockback. No custom damage.
                self.guest_attack(host, bot.conn, target_id);
                bot.last_attack = bot.ticks;
            }
        }
        if bot.ticks.is_multiple_of(10) {
            let hp = host.hub.guests[bi].health;
            let yours = host.hub.guests[oi].health;
            let line = if bot.ticks <= 60 {
                format!("Practice starts in {}", (60 - bot.ticks) / 20 + 1)
            } else {
                format!(
                    "Practice | {} | Bot {:.0}/20 | You {:.0}/20",
                    difficulty.name(),
                    hp,
                    yours
                )
            };
            host.hub.overlay_message(literal(&line));
        }
        self.practice.bot = Some(bot);
    }
    fn practice_kit(&mut self, host: &mut LanHost, i: usize) {
        let conn = host.hub.guests[i].conn;
        let g = &mut host.hub.guests[i];
        g.inventory = vec![None; 36];
        g.inventory[0] = Some(ItemStack::new("minecraft:iron_sword", 1));
        g.offhand = Some(ItemStack::new("minecraft:shield", 1));
        // Inventory armor order: helmet, chestplate, leggings, boots.
        g.armor = [
            "iron_helmet",
            "iron_chestplate",
            "iron_leggings",
            "iron_boots",
        ]
        .map(|s| Some(ItemStack::new(&format!("minecraft:{s}"), 1)));
        g.dead = false;
        g.death_time = 0;
        g.selected = 0;
        g.health = 20.0;
        g.food = 20;
        g.saturation = 5.0;
        self.as_guest(host, conn, |g| {
            if let Some(p) = g.player.as_mut() {
                let _ = p.set_health(20.0, false);
                p.velocity = Vec3::ZERO;
            }
        });
        host.hub.send_inventory(i);
    }
    fn practice_arena(&mut self, [cx, y, cz]: [i32; 3]) -> Result<()> {
        let floor = super::render_control::parse_state(
            self.pack.assets.registry,
            "minecraft:polished_andesite",
        )?;
        let edge = super::render_control::parse_state(
            self.pack.assets.registry,
            "minecraft:blue_concrete",
        )?;
        let glass =
            super::render_control::parse_state(self.pack.assets.registry, "minecraft:glass")?;
        // Check residency before the first write, so a not-yet-loaded edge
        // cannot leave an incomplete arena and keep retrying it each tick.
        for x in [cx - 12, cx + 12] {
            for z in [cz - 12, cz + 12] {
                if !self.world.chunks.contains_key(&(x >> 4, z >> 4)) {
                    return Err("arena chunks not loaded".into());
                }
            }
        }
        // Original 25x25 arena in its own fresh flat world. Never modify a
        // normal save, copy a map, or generate work every rendered frame.
        for x in -12i32..=12 {
            for z in -12i32..=12 {
                let rim = x.abs() == 12 || z.abs() == 12;
                self.world
                    .set(cx + x, y - 1, cz + z, if rim { edge } else { floor })?;
                for dy in 0..=2 {
                    self.world.set(
                        cx + x,
                        y + dy,
                        cz + z,
                        if rim { glass } else { self.world.air },
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiplayer_and_unmarked_saves_are_denied() {
        assert!(permitted(true, true, false, false, 1));
        assert!(!permitted(false, true, false, false, 1));
        assert!(!permitted(true, false, false, false, 1));
        assert!(!permitted(true, true, true, false, 1));
        assert!(!permitted(true, true, false, true, 1));
        assert!(!permitted(true, true, false, false, 2));
    }
    #[test]
    fn approach_retreat_and_strafe_stay_within_normal_input() {
        for d in [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard] {
            let far = decision([0.0; 3], [0.0, 0.0, 10.0], 0, d);
            assert!(far[1] > 0.0);
            assert_eq!(far[2], 0.0);
            let near = decision([0.0; 3], [0.0, 0.0, 1.0], 0, d);
            assert!(near[1] < 0.0);
            let other = decision([0.0; 3], [0.0, 0.0, 1.0], 34, d);
            assert!(near[0] * other[0] < 0.0);
            assert!(near[0].hypot(near[1]) <= 1.0);
            assert!(d.cooldown() >= 13);
        }
    }
}
