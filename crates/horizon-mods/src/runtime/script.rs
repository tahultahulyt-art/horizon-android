//! Lua script mods (docs/script-mods.md): a `.hmod`/`.ksmod` holding
//! `main.lua` instead of `mod.wasm`, for mods written as small text scripts
//! on any phone — no compiler, no computer. The script runs in Luau
//! (Roblox's Lua): it has no file, network or package library of its own,
//! only the API below, the capabilities its manifest declares, an
//! instruction budget per call and a heap cap.
//!
//! The contract mirrors the WebAssembly one (docs/mods.md): the script sets
//! a global `MOD` table naming its commands and the events it wants, and
//! defines `on_tick`, `on_frame`, `on_chat` and `on_command` functions for
//! those events. Its API tables mirror the WIT interfaces:
//!
//! - `log(line)`: one line in the game's log under the mod's id. Always on.
//! - `game` (capability `game-read`): `player()` → a table with `x`, `y`,
//!   `z`, `yaw`, `pitch`, `health`, `food`, `dimension` or nil on the title
//!   screen; `clock()` → `game_time`, `day_time` or nil; `cps()` → attack
//!   and use clicks in the last second.
//! - `hud` (capability `hud`): `panel(corner, lines)` — `corner` is
//!   `"top-left"`, `"top-right"`, `"bottom-left"` or `"bottom-right"`,
//!   `lines` up to 16 strings or `{text=..., color=0xRRGGBB}` tables — and
//!   `clear()`. The game draws the panel every frame until it changes.
//! - `chat` (capability `chat`): `show(text)`, a line after the mod's name.
//! - `storage` (capability `storage`): `get(key)` → string or nil,
//!   `set(key, value)`, `delete(key)`; keys `[a-z0-9_.-]{1,64}`, the same
//!   per-mod folder, quota and budgets WebAssembly mods get.
//!
//! - `inventory` (capability `inventory`): `read()` → `{ {slot=, item=}, .. }`
//!   (item is the stack's SNBT; slots 0-35, 100-103 armor feet to head, 150
//!   offhand; filled only inside `on_command`), `creative()` → true in creative
//!   mode, and `replace(entries)` (creative mode only; unlisted slots become
//!   empty).
//!
//! A script mod is client-side only in this build and may declare only
//! `game-read`, `hud`, `chat`, `storage` and `inventory`; more of the contract opens as
//! the API grows (docs/script-mods.md). A script that errors in `on_tick`,
//! `on_frame` or `on_chat`, or overruns its budget, is turned off like a
//! WebAssembly mod that traps; the game goes on. A script error inside
//! `on_command` answers the command in red instead.

use super::{
    Corner, Line, Manifest, Output, Registration, Side, Snapshot, State, WorldSlot,
};
use crate::Capability;
use mlua::{Lua, Table, Value, VmState};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
};

/// A script call's instruction budget, in Luau interrupts (about one VM
/// instruction each): init may plan, tick and frame must stay cheap, a
/// command or chat line sits between. Mirrors the wasm fuel ladder.
const OPS_INIT: i64 = 1_000_000;
const OPS_TICK: i64 = 50_000;
const OPS_FRAME: i64 = 10_000;
const OPS_EVENT: i64 = 100_000;
/// A script mod's Lua heap cap, far under a WebAssembly mod's 64 MiB.
const LUA_MEMORY: usize = 16 << 20;

/// The capabilities a script mod may declare in this build.
const SCRIPT_CAPS: &[Capability] = &[
    Capability::GameRead,
    Capability::Hud,
    Capability::Chat,
    Capability::Storage,
    Capability::Inventory,
];

/// A running Lua script mod. Its [`State`] lives in the Lua state's app
/// data, where the API functions the script calls can reach it.
pub struct LuaRun {
    pub lua: Lua,
    /// What the script's `MOD` table asked for, in the WebAssembly
    /// contract's shape, so the dispatch in `Mods` treats both kinds alike.
    pub registration: Registration,
    budget: Arc<AtomicI64>,
}

impl LuaRun {
    /// Loads `script` and reads its `MOD` table. `path` is the mod's own
    /// file, for capability `storage`'s per-mod directory.
    pub fn start(
        path: &Path,
        m: &Manifest,
        script: &str,
        snapshot: Arc<Snapshot>,
        world: WorldSlot,
    ) -> Result<Self, String> {
        if m.side != Side::Client {
            return Err("a Lua script mod is client-side in this build (side = \"client\")".into());
        }
        if let Some(cap) = m
            .capabilities
            .iter()
            .find(|c| !SCRIPT_CAPS.contains(c))
        {
            return Err(format!(
                "a Lua script mod may use only game-read, hud, chat, storage and inventory in this build, not {}",
                cap.describe()
            ));
        }
        let lua = Lua::new();
        let budget = Arc::new(AtomicI64::new(0));
        let shared = budget.clone();
        lua.set_interrupt(move |_| {
            if shared.fetch_sub(1, Ordering::Relaxed) <= 0 {
                Err(mlua::Error::external(
                    "it took too long (an endless loop?)",
                ))
            } else {
                Ok(VmState::Continue)
            }
        });
        if let Err(e) = lua.set_memory_limit(LUA_MEMORY) {
            log::warn!("Horizon: Lua memory limit unavailable for mod {}: {e}", m.id);
        }
        let data_dir = path
            .parent()
            .unwrap_or(Path::new("."))
            .join(crate::manifest::STORAGE_DIR)
            .join(&m.id);
        lua.set_app_data(State {
            name: m.name.clone(),
            id: m.id.clone(),
            snapshot,
            out: Vec::new(),
            panel: None,
            ui: Default::default(),
            shapes: Vec::new(),
            corner: None,
            zoom: None,
            minimap: None,
            far_terrain: None,
            input: Default::default(),
            browser: Default::default(),
            data_dir,
            storage_budget: super::STORAGE_WRITE_BUDGET,
            limits: Default::default(),
            world,
        });
        install_api(&lua, m).map_err(|e| format!("setting up the mod's API: {e}"))?;
        budget.store(OPS_INIT, Ordering::Relaxed);
        lua.load(script)
            .set_name("@main.lua")
            .exec()
            .map_err(|e| lua_reason("main.lua", e))?;
        let registration = read_registration(&lua)?;
        Ok(Self {
            lua,
            registration,
            budget,
        })
    }

    /// The outputs the script's start queued (`chat.show` while loading).
    pub fn take_out(&self) -> Vec<Output> {
        let mut data = self
            .lua
            .try_app_data_mut::<State>()
            .ok()
            .flatten()
            .expect("mod state is present");
        std::mem::take(&mut data.out)
    }

    /// The instruction budget for the call that follows.
    pub fn set_budget(&self, fuel: u64) {
        let ops = if fuel >= super::FUEL_INIT {
            OPS_INIT
        } else if fuel >= super::FUEL_EVENT {
            OPS_EVENT
        } else if fuel >= super::FUEL_TICK {
            OPS_TICK
        } else {
            OPS_FRAME
        };
        self.budget.store(ops, Ordering::Relaxed);
    }

    /// Runs the script's global function `name` when it defines one.
    fn call(&self, name: &str, args: impl mlua::IntoLuaMulti) -> Result<Value, String> {
        let f: Option<mlua::Function> = self
            .lua
            .globals()
            .get(name)
            .map_err(|e| format!("reading {name}: {e}"))?;
        let Some(f) = f else {
            return Ok(Value::Nil);
        };
        f.call(args).map_err(|e| lua_reason(name, e))
    }

    pub fn call_tick(&self, tick: u64) -> Result<(), String> {
        self.call("on_tick", (tick,)).map(|_| ())
    }
    pub fn call_frame(&self, seconds: f32) -> Result<(), String> {
        self.call("on_frame", (seconds,)).map(|_| ())
    }
    pub fn call_chat(&self, text: &str) -> Result<(), String> {
        self.call("on_chat", (text,)).map(|_| ())
    }
    /// `on_command(name, args)`: a string is feedback, nil nothing, a Lua
    /// error red feedback — a command's error never stops the mod.
    pub fn call_command(&self, name: &str, args: &str) -> Result<Option<String>, String> {
        match self.call("on_command", (name, args)) {
            Ok(Value::String(s)) => Ok(Some(s.to_string_lossy())),
            Ok(_) => Ok(None),
            Err(e) => Err(super::clip(e)),
        }
    }
}

/// The script's `MOD` table: `commands` it owns and the events it wants.
fn read_registration(lua: &Lua) -> Result<Registration, String> {
    let g = lua.globals();
    let table: Option<Table> = g.get("MOD").map_err(|e| e.to_string())?;
    let mut r = Registration {
        commands: Vec::new(),
        tick: false,
        frame: false,
        chat: false,
    };
    let Some(table) = table else {
        return Ok(r);
    };
    let want = |key: &str| -> Result<bool, String> {
        Ok(table
            .get::<Option<bool>>(key)
            .map_err(|e| format!("MOD.{key}: {e}"))?
            .unwrap_or(false))
    };
    r.tick = want("tick")?;
    r.frame = want("frame")?;
    r.chat = want("chat")?;
    let commands: Vec<String> = table
        .get::<Option<Vec<String>>>("commands")
        .map_err(|e| format!("MOD.commands: a list of names expected ({e})"))?
        .unwrap_or_default();
    for name in commands {
        let ok = (1..=32).contains(&name.len())
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-".contains(&b));
        if !ok {
            return Err(format!(
                "MOD.commands: {name:?} is not a command name ([a-z0-9_-]{{1,32}})"
            ));
        }
        r.commands.push(name);
    }
    Ok(r)
}

/// An error as the player reads it, like the WebAssembly sandbox's.
fn lua_reason(where_: &str, e: mlua::Error) -> String {
    let text = e.to_string();
    if text.contains("it took too long") {
        "it took too long (an endless loop?)".into()
    } else {
        super::clip(format!("{where_}: {text}"))
    }
}

/// The mod's API tables, one per declared capability (plus `log`).
fn install_api(lua: &Lua, m: &Manifest) -> Result<(), String> {
    let toml_err = |e: mlua::Error| e.to_string();
    let g = lua.globals();
    g.set(
        "log",
        lua.create_function(|lua, message: String| {
            with_data(lua, |d| {
                use super::bindings::horizon::mods::log::Host;
                d.write(super::bindings::horizon::mods::log::Level::Info, message);
            });
            Ok(())
        })
        .map_err(toml_err)?,
    )
    .map_err(toml_err)?;
    if m.capabilities.contains(&Capability::GameRead) {
        let game = lua.create_table().map_err(toml_err)?;
        game.set(
            "player",
            lua.create_function(|lua, ()| {
                let player = with_data(lua, |d| d.snapshot.player.clone());
                player.map(|p| player_table(lua, p)).transpose()
            })
            .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        game.set(
            "clock",
            lua.create_function(|lua, ()| {
                let clock = with_data(lua, |d| d.snapshot.clock);
                match clock {
                    Some(c) => {
                        let t = lua.create_table()?;
                        t.set("game_time", c.game_time)?;
                        t.set("day_time", c.day_time)?;
                        Ok(Some(t))
                    }
                    None => Ok(None),
                }
            })
            .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        game.set(
            "cps",
            lua.create_function(|lua, ()| Ok(with_data(lua, |d| d.snapshot.cps)))
                .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        g.set("game", game).map_err(toml_err)?;
    }
    if m.capabilities.contains(&Capability::Hud) {
        let hud = lua.create_table().map_err(toml_err)?;
        hud.set(
            "panel",
            lua.create_function(|lua, (corner, lines): (String, Table)| {
                let corner = parse_corner(&corner)?;
                let mut out = Vec::new();
                for line in lines.sequence_values::<Value>() {
                    let line = match line? {
                        Value::String(s) => Line {
                            text: s.to_string_lossy(),
                            color: 0xFF_FFFF,
                        },
                        Value::Table(t) => Line {
                            text: t.get("text")?,
                            color: t.get::<Option<u32>>("color")?.unwrap_or(0xFF_FFFF),
                        },
                        _ => {
                            return Err(mlua::Error::external(
                                "hud.panel: a line is a string or {text=..., color=...}",
                            ));
                        }
                    };
                    out.push(line);
                }
                with_data(lua, |d| {
                    use super::bindings::horizon::mods::hud::Host;
                    d.set_panel(corner, out);
                });
                Ok(())
            })
            .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        hud.set(
            "clear",
            lua.create_function(|lua, ()| {
                with_data(lua, |d| {
                    use super::bindings::horizon::mods::hud::Host;
                    d.clear();
                });
                Ok(())
            })
            .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        g.set("hud", hud).map_err(toml_err)?;
    }
    if m.capabilities.contains(&Capability::Chat) {
        let chat = lua.create_table().map_err(toml_err)?;
        chat.set(
            "show",
            lua.create_function(|lua, text: String| {
                with_data(lua, |d| {
                    use super::bindings::horizon::mods::chat::Host;
                    d.show(text);
                });
                Ok(())
            })
            .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        g.set("chat", chat).map_err(toml_err)?;
    }
    if m.capabilities.contains(&Capability::Inventory) {
        let inv = lua.create_table().map_err(toml_err)?;
        inv.set(
            "read",
            lua.create_function(|lua, ()| {
                let items = with_data(lua, |d| d.snapshot.inventory.clone());
                let out = lua.create_table()?;
                for (i, (slot, snbt)) in items.iter().enumerate() {
                    let e = lua.create_table()?;
                    e.set("slot", *slot)?;
                    e.set("item", snbt.as_str())?;
                    out.set(i + 1, e)?;
                }
                Ok(out)
            })
            .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        inv.set(
            "creative",
            lua.create_function(|lua, ()| Ok(with_data(lua, |d| d.snapshot.creative)))
                .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        inv.set(
            "replace",
            lua.create_function(|lua, entries: Table| {
                let mut slots: Vec<(u8, String)> = Vec::new();
                for e in entries.sequence_values::<Table>() {
                    let e = e?;
                    let slot: u8 = e.get("slot")?;
                    let item: String = e.get("item")?;
                    if !matches!(slot, 0..=35 | 100..=103 | 150) {
                        return Err(mlua::Error::external(format!(
                            "inventory.replace: no slot {slot}"
                        )));
                    }
                    if item.len() > 16384 {
                        return Err(mlua::Error::external("inventory.replace: an item is too big"));
                    }
                    if slots.iter().any(|(s, _)| *s == slot) {
                        return Err(mlua::Error::external(format!(
                            "inventory.replace: slot {slot} twice"
                        )));
                    }
                    slots.push((slot, item));
                    if slots.len() > 41 {
                        return Err(mlua::Error::external("inventory.replace: too many slots"));
                    }
                }
                with_data(lua, |d| {
                    if !d.snapshot.creative {
                        return Err(mlua::Error::external(
                            "only in creative mode",
                        ));
                    }
                    d.out.retain(|o| !matches!(o, Output::SetInventory { .. }));
                    d.out.push(Output::SetInventory { slots });
                    Ok(())
                })
            })
            .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        g.set("inventory", inv).map_err(toml_err)?;
    }
    if m.capabilities.contains(&Capability::Storage) {
        let storage = lua.create_table().map_err(toml_err)?;
        storage.set(
            "get",
            lua.create_function(|lua, key: String| {
                let value = with_data(lua, |d| {
                    use super::bindings::horizon::mods::storage::Host;
                    d.get(key)
                });
                value.map(|v| lua.create_string(v)).transpose()
            })
            .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        storage.set(
            "set",
            lua.create_function(|lua, (key, value): (String, mlua::String)| {
                let result = with_data(lua, |d| {
                    use super::bindings::horizon::mods::storage::Host;
                    d.set(key, value.as_bytes().to_vec())
                });
                result.map_err(mlua::Error::external)
            })
            .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        storage.set(
            "delete",
            lua.create_function(|lua, key: String| {
                with_data(lua, |d| {
                    use super::bindings::horizon::mods::storage::Host;
                    d.delete(key);
                });
                Ok(())
            })
            .map_err(toml_err)?,
        )
        .map_err(toml_err)?;
        g.set("storage", storage).map_err(toml_err)?;
    }
    Ok(())
}

/// The mod's [`State`], for one API call. A borrow conflict would mean the
/// API re-entered itself; it never does, so a poisoned borrow is a bug,
/// not a script error — turn it into one anyway.
fn with_data<R>(lua: &Lua, f: impl FnOnce(&mut State) -> R) -> R {
    let mut data = lua
        .try_app_data_mut::<State>()
        .ok()
        .flatten()
        .expect("mod state is not re-entered");
    f(&mut data)
}

fn player_table(lua: &Lua, p: super::Player) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.set("x", p.x)?;
    t.set("y", p.y)?;
    t.set("z", p.z)?;
    t.set("yaw", p.yaw)?;
    t.set("pitch", p.pitch)?;
    t.set("health", p.health)?;
    t.set("food", p.food)?;
    t.set("dimension", p.dimension.as_str())?;
    Ok(t)
}

fn parse_corner(corner: &str) -> mlua::Result<Corner> {
    Ok(match corner {
        "top-left" => Corner::TopLeft,
        "top-right" => Corner::TopRight,
        "bottom-left" => Corner::BottomLeft,
        "bottom-right" => Corner::BottomRight,
        _ => {
            return Err(mlua::Error::external(format!(
                "hud.panel: {corner:?} is not top-left, top-right, bottom-left or bottom-right"
            )));
        }
    })
}
