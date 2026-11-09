# Script mods (Lua)

A script mod is a `.ksmod` (or `.hmod`): a zip holding `horizon-mod.toml`
and `main.lua`. It is the same sandbox shape as a WebAssembly mod
(docs/mods.md) — capabilities the player sees, off until turned on, a time
budget per call, turned off alone when it misbehaves — but the code is a
plain text script: anyone can write one on a phone, no compiler, no
computer. The engine is Luau (Roblox's Lua), vendored; scripts have no
file, network or package access of their own.

## The manifest

Same `horizon-mod.toml` as any mod. In this build a script mod is
client-side (`side = "client"`, the default) and may declare only
`game-read`, `hud`, `chat`, `storage` and `inventory`; more of the contract opens as
the script API grows.

```toml
id = "my_mod"
name = "My Mod"
version = "1.0.0"
description = "What it does."
authors = ["you"]
api = "0.1"
capabilities = ["game-read", "hud", "storage"]
```

## The script

`main.lua` declares its hooks in a global `MOD` table and defines the
event functions it asked for:

```lua
MOD = {
  commands = { "mymod" },  -- /mymod ... runs on_command
  tick = true,             -- on_tick(t) every game tick (20 per second)
  frame = false,           -- on_frame(seconds) every rendered frame
  chat = false,            -- on_chat(text) for each received chat line
}

function on_tick(t)
  local attack, use = game.cps()
  hud.panel("top-left", { "CPS: " .. attack })
end

function on_command(name, args)
  return "you said: " .. args   -- string: feedback; error("why"): red
end
```

## API v1

- `log(text)` — one line in the game's log under the mod's id. Always on.
- `game` (capability `game-read`):
  - `game.player()` → `{x, y, z, yaw, pitch, health, food, dimension}` or
    nil on the title screen
  - `game.clock()` → `{game_time, day_time}` or nil
  - `game.cps()` → attack and use clicks in the last second (two numbers)
- `hud` (capability `hud`): one text panel per mod, drawn by the game
  every frame until changed (no call per frame).
  - `hud.panel(corner, lines)` — `corner` is `"top-left"`, `"top-right"`,
    `"bottom-left"` or `"bottom-right"`; up to 16 lines, each a string or
    `{text = "...", color = 0xRRGGBB}`
  - `hud.clear()`
- `chat` (capability `chat`): `chat.show(text)`, a line after the mod's
  name that only the local player sees (8 per call).
- `storage` (capability `storage`): `storage.get(key)` → string or nil,
  `storage.set(key, value)`, `storage.delete(key)`. Keys
  `[a-z0-9_.-]{1,64}`; the same per-mod folder, quota (4 MiB, 64 keys) and
  write budget as WebAssembly mods.

- `inventory` (capability `inventory`, "see everything in your inventory
  and, in creative mode, replace it"):
  - `inventory.read()` → `{ {slot = n, item = "<SNBT>"}, ... }`, one entry
    per non-empty slot: 0-35 inventory (0-8 hotbar), 100-103 armor feet to
    head, 150 offhand. Filled only inside `on_command`; elsewhere empty.
  - `inventory.creative()` → true in creative mode.
  - `inventory.operator()` → true when the player has operator rights (command level 2 or more).
  - `inventory.replace(entries)` — entries in the shape `read()` returns;
    every slot not listed becomes empty. Creative mode only; an unreadable
    item changes nothing. A server still has the last word: it takes the
    change only from a creative player.

## Limits and safety

- A call gets an instruction budget (about 1M at load, 100k per command or
  chat line, 50k per tick, 10k per frame); a script that overruns — an
  endless loop — is turned off with "it took too long", like a WebAssembly
  mod out of fuel. The game goes on.
- The Lua heap is capped at 16 MiB.
- A script error in `on_tick`, `on_frame` or `on_chat` stops the mod;
  Options > Mods says why. An error inside `on_command` answers the
  command in red instead, and the mod stays on.
- Scripts cannot touch gameplay: no reach, hitbox, damage or movement
  changes are possible through this API, and none are planned — those
  would be cheats on public servers.

## For players

Kusobo Loader > "Lua sample" installs the CPS counter (off; enable in
Installed). "Lua template" exports `kusobo-script-template.zip` to the
Configuration folder with a ready manifest, script and `GUIDE.txt`, a
phone-friendly how-to. Import accepts `.ksmod` and `.hmod` alike.
