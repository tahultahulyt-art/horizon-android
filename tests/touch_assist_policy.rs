//! Runs the real game-side policy against deterministic scene stubs.
#![allow(dead_code)]
extern crate self as horizon_client;
extern crate self as horizon_core;
pub mod geometry {
    #[derive(Clone, Copy)]
    pub struct Vec3 {
        pub x: f64,
        pub y: f64,
        pub z: f64,
    }
    #[derive(Clone, Copy)]
    pub struct Aabb {
        pub lo: Vec3,
        pub hi: Vec3,
    }
    impl Aabb {
        pub fn min_position(&self) -> Vec3 {
            self.lo
        }
        pub fn max_position(&self) -> Vec3 {
            self.hi
        }
    }
}
#[path = "../crates/horizon-client-render/src/touch/assist.rs"]
pub mod assist;
pub mod touch {
    pub use crate::assist;
}
use geometry::{Aabb, Vec3};
struct Settings {
    aim_assist_strength: f32,
}
struct Touch {
    settings: Settings,
    crosshair: bool,
}
impl Touch {
    fn crosshair(&self) -> bool {
        self.crosshair
    }
}
struct Pad {
    touch: Touch,
}
struct Kind {
    id: &'static str,
}
struct Extra {
    spectator: bool,
}
struct Mob {
    kind: Kind,
    extra: Extra,
    removed: bool,
    id: u32,
    alive: bool,
    invisible: bool,
    bounds: Aabb,
}
impl Mob {
    fn alive(&self) -> bool {
        self.alive
    }
    fn is_invisible(&self) -> bool {
        self.invisible
    }
    fn bounds(&self) -> Aabb {
        self.bounds
    }
}
struct Effects {
    invisible: bool,
}
impl Effects {
    fn get(&self, _: &str) -> Option<()> {
        self.invisible.then_some(())
    }
}
struct Guest {
    loaded: bool,
    dead: bool,
    spectator: bool,
    effects: Effects,
    bounds: Aabb,
}
impl Guest {
    fn spectator(&self) -> bool {
        self.spectator
    }
    fn bounds(&self) -> Aabb {
        self.bounds
    }
}
struct Hub {
    guests: Vec<Guest>,
}
impl Hub {
    fn here(&self, _: &Guest) -> bool {
        true
    }
}
struct Lan {
    hub: Hub,
}
struct Game<'a> {
    pad: Pad,
    mobs: Vec<Mob>,
    lan: Option<Lan>,
    yaw: f32,
    pitch: f32,
    entity_reach: f64,
    riding: Option<u32>,
    spectator: bool,
    camera: bool,
    wall: bool,
    marker: std::marker::PhantomData<&'a ()>,
}
impl Game<'_> {
    fn spectator(&self) -> bool {
        self.spectator
    }
    fn remote_camera_id(&self) -> Option<u32> {
        self.camera.then_some(99)
    }
    fn eye_position(&self) -> Option<[f64; 3]> {
        Some([0.0, 1.6, 0.0])
    }
    fn clip_ray(&self, _: [f64; 3], _: [f64; 3], _: f64) -> Option<()> {
        self.wall.then_some(())
    }
}
fn bounds() -> Aabb {
    Aabb {
        lo: Vec3 {
            x: -0.3,
            y: 0.0,
            z: 2.0,
        },
        hi: Vec3 {
            x: 0.3,
            y: 1.8,
            z: 2.6,
        },
    }
}
impl Default for Game<'_> {
    fn default() -> Self {
        Self {
            pad: Pad {
                touch: Touch {
                    settings: Settings {
                        aim_assist_strength: 0.15,
                    },
                    crosshair: true,
                },
            },
            mobs: vec![Mob {
                kind: Kind {
                    id: "minecraft:player",
                },
                extra: Extra { spectator: false },
                removed: false,
                id: 1,
                alive: true,
                invisible: false,
                bounds: bounds(),
            }],
            lan: None,
            yaw: 8.0,
            pitch: 0.0,
            entity_reach: 3.0,
            riding: None,
            spectator: false,
            camera: false,
            wall: false,
            marker: std::marker::PhantomData,
        }
    }
}
#[path = "../crates/horizon-client/src/game/touch_assist.rs"]
mod policy;
fn turns(g: &Game<'_>) -> bool {
    g.touch_assist_correction([-0.1, 0.0], 0.016)[0] < 0.0
}
#[test]
fn only_visible_living_players_are_candidates() {
    let mut g = Game::default();
    assert!(turns(&g));
    g.mobs[0].kind.id = "minecraft:cow";
    assert!(!turns(&g));
    g.mobs[0].kind.id = "minecraft:player";
    g.mobs[0].invisible = true;
    assert!(!turns(&g));
    g.mobs[0].invisible = false;
    g.mobs[0].alive = false;
    assert!(!turns(&g));
    g.mobs[0].alive = true;
    g.mobs[0].extra.spectator = true;
    assert!(!turns(&g));
    g.mobs[0].extra.spectator = false;
    g.mobs[0].removed = true;
    assert!(!turns(&g));
}
#[test]
fn walls_and_camera_modes_disable_assist() {
    let mut g = Game::default();
    g.wall = true;
    assert!(!turns(&g));
    g.wall = false;
    g.spectator = true;
    assert!(!turns(&g));
    g.spectator = false;
    g.camera = true;
    assert!(!turns(&g));
    g.camera = false;
    g.pad.touch.crosshair = false;
    assert!(!turns(&g));
}
#[test]
fn living_visible_lan_guest_can_be_used_but_not_invisible() {
    let mut g = Game::default();
    g.mobs.clear();
    g.lan = Some(Lan {
        hub: Hub {
            guests: vec![Guest {
                loaded: true,
                dead: false,
                spectator: false,
                effects: Effects { invisible: false },
                bounds: bounds(),
            }],
        },
    });
    assert!(turns(&g));
    g.lan.as_mut().unwrap().hub.guests[0].effects.invisible = true;
    assert!(!turns(&g));
}
