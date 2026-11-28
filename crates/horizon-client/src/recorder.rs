//! Stage A replay recorder: per-tick POV camera + world state logging to a
//! single deflated `.hrec` file. Capture only — no rendering work, no game
//! behaviour changes; the recorder never touches input or physics.
//!
//! Format (all little-endian):
//!   header: "HREC", u32 version, u32 tick rate, u64 unix start secs,
//!           u16 world-name byte length, name bytes
//!   then tagged records:
//!   1 TICK    i64 game_time, u8 weather bits (1 raining, 2 thundering),
//!             3xf64 camera eye, f32 yaw, f32 pitch, u8 selected hotbar slot,
//!             u16 entity count, entities (below),
//!             u32 block-change count, per change 3xi32 pos + u32 new state
//!   2 SPAWN   u32 entity id, u8 class (0 mob, 1 item, 2 xp orb),
//!             u32 aux (item count / orb value / 0), u16 name len, name utf8
//!   3 DESPAWN u32 entity id
//!   4 CHUNK   i32 cx, i32 cz, i32 min_y, u8 section count, then per section:
//!             u8 uniform flag; uniform => u32 value, else 4096 x u32
//!   5 END     u64 total ticks recorded
//!
//! Entities per TICK: u32 id, 3xf32 pos, f32 yaw, f32 head yaw, f32 head
//! pitch, u8 flags (1 on ground, 2 hurt, 4 dying), f32 health.

use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use flate2::{Compression, write::DeflateEncoder};

use crate::fast_hash::{FxHashMap, FxHashSet};

pub const FORMAT_VERSION: u32 = 1;
pub const TICK_RATE: u32 = 20;

const TAG_TICK: u8 = 1;
const TAG_SPAWN: u8 = 2;
const TAG_DESPAWN: u8 = 3;
const TAG_CHUNK: u8 = 4;
const TAG_END: u8 = 5;

pub const CLASS_MOB: u8 = 0;
pub const CLASS_ITEM: u8 = 1;
pub const CLASS_ORB: u8 = 2;

/// One entity's pose this tick.
pub struct EntityPose {
    pub id: u32,
    pub class: u8,
    /// Spawn-time data: type or item name, and item count / orb value.
    pub name: String,
    pub aux: u32,
    pub pos: [f32; 3],
    pub yaw: f32,
    pub head_yaw: f32,
    pub head_pitch: f32,
    pub flags: u8,
    pub health: f32,
}

/// A chunk's full block data for the recording's terrain baseline.
pub struct ChunkSnap {
    pub cx: i32,
    pub cz: i32,
    pub min_y: i32,
    /// One entry per resident section: either uniform or the full 4096 states.
    pub sections: Vec<Result<u32, Box<[u32; 4096]>>>,
}

pub struct TickFrame {
    pub game_time: i64,
    pub raining: bool,
    pub thundering: bool,
    pub eye: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub selected: u8,
    pub entities: Vec<EntityPose>,
    pub block_changes: Vec<((i32, i32, i32), u32)>,
    pub new_chunks: Vec<ChunkSnap>,
}

pub struct Recorder {
    out: DeflateEncoder<BufWriter<File>>,
    known: FxHashMap<u32, u8>,
    known_chunks: FxHashSet<(i32, i32)>,
    ticks: u64,
    path: PathBuf,
}

impl Recorder {
    pub fn start(path: PathBuf, world_name: &str) -> io::Result<Self> {
        let file = File::create(&path)?;
        let mut out = DeflateEncoder::new(BufWriter::new(file), Compression::fast());
        out.write_all(b"HREC")?;
        w32(&mut out, FORMAT_VERSION)?;
        w32(&mut out, TICK_RATE)?;
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        w64(&mut out, secs)?;
        let name = world_name.as_bytes();
        w16(&mut out, name.len().min(u16::MAX as usize) as u16)?;
        out.write_all(&name[..name.len().min(u16::MAX as usize)])?;
        Ok(Self {
            out,
            known: FxHashMap::default(),
            known_chunks: FxHashSet::default(),
            ticks: 0,
            path,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Whether this chunk column still needs a baseline snapshot written.
    pub fn needs_chunk(&self, cx: i32, cz: i32) -> bool {
        !self.known_chunks.contains(&(cx, cz))
    }

    pub fn tick(&mut self, frame: TickFrame) -> io::Result<()> {
        for snap in &frame.new_chunks {
            self.known_chunks.insert((snap.cx, snap.cz));
            self.out.write_all(&[TAG_CHUNK])?;
            w32(&mut self.out, snap.cx as u32)?;
            w32(&mut self.out, snap.cz as u32)?;
            w32(&mut self.out, snap.min_y as u32)?;
            self.out.write_all(&[snap.sections.len() as u8])?;
            for section in &snap.sections {
                match section {
                    Ok(uniform) => {
                        self.out.write_all(&[1])?;
                        w32(&mut self.out, *uniform)?;
                    }
                    Err(full) => {
                        self.out.write_all(&[0])?;
                        for v in full.iter() {
                            w32(&mut self.out, *v)?;
                        }
                    }
                }
            }
        }
        // Spawns and despawns, diffed against the last tick's entity set.
        let mut seen: FxHashSet<u32> = FxHashSet::default();
        for e in &frame.entities {
            seen.insert(e.id);
            if self.known.get(&e.id) != Some(&e.class) {
                if self.known.contains_key(&e.id) {
                    self.out.write_all(&[TAG_DESPAWN])?;
                    w32(&mut self.out, e.id)?;
                }
                self.out.write_all(&[TAG_SPAWN])?;
                w32(&mut self.out, e.id)?;
                self.out.write_all(&[e.class])?;
                w32(&mut self.out, e.aux)?;
                let name = e.name.as_bytes();
                w16(&mut self.out, name.len().min(u16::MAX as usize) as u16)?;
                self.out.write_all(&name[..name.len().min(u16::MAX as usize)])?;
                self.known.insert(e.id, e.class);
            }
        }
        let gone: Vec<u32> = self
            .known
            .keys()
            .filter(|id| !seen.contains(id))
            .copied()
            .collect();
        for id in gone {
            self.out.write_all(&[TAG_DESPAWN])?;
            w32(&mut self.out, id)?;
            self.known.remove(&id);
        }
        self.out.write_all(&[TAG_TICK])?;
        w64(&mut self.out, frame.game_time as u64)?;
        self.out.write_all(&[u8::from(frame.raining) | u8::from(frame.thundering) << 1])?;
        for v in frame.eye {
            self.out.write_all(&v.to_le_bytes())?;
        }
        self.out.write_all(&frame.yaw.to_le_bytes())?;
        self.out.write_all(&frame.pitch.to_le_bytes())?;
        self.out.write_all(&[frame.selected])?;
        w16(&mut self.out, frame.entities.len() as u16)?;
        for e in &frame.entities {
            w32(&mut self.out, e.id)?;
            for v in e.pos {
                self.out.write_all(&v.to_le_bytes())?;
            }
            self.out.write_all(&e.yaw.to_le_bytes())?;
            self.out.write_all(&e.head_yaw.to_le_bytes())?;
            self.out.write_all(&e.head_pitch.to_le_bytes())?;
            self.out.write_all(&[e.flags])?;
            self.out.write_all(&e.health.to_le_bytes())?;
        }
        w32(&mut self.out, frame.block_changes.len() as u32)?;
        for ((x, y, z), state) in &frame.block_changes {
            w32(&mut self.out, *x as u32)?;
            w32(&mut self.out, *y as u32)?;
            w32(&mut self.out, *z as u32)?;
            w32(&mut self.out, *state)?;
        }
        self.ticks += 1;
        Ok(())
    }

    /// Flush and close; returns the file path and recorded tick count.
    pub fn finish(self) -> io::Result<(PathBuf, u64)> {
        let mut this = self;
        this.out.write_all(&[TAG_END])?;
        w64(&mut this.out, this.ticks)?;
        this.out.flush()?;
        this.out.finish()?;
        Ok((this.path, this.ticks))
    }
}

fn w16(w: &mut impl Write, v: u16) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}
fn w32(w: &mut impl Write, v: u32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}
fn w64(w: &mut impl Write, v: u64) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

/// `recording-YYYYMMDD-HHMMSS.hrec` from the wall clock (no chrono here).
pub fn file_name(now_secs: u64) -> String {
    let days = (now_secs / 86_400) as i64;
    let rem = now_secs % 86_400;
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Howard Hinnant's civil-from-days algorithm.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("recording-{y:04}{m:02}{d:02}-{h:02}{mi:02}{s:02}.hrec")
}
