-- Kusobo CPS Counter: attack and use clicks per second, top-left.
-- This file is also the template for your own mods (see GUIDE.txt).

MOD = {
  commands = { "cps" },  -- /cps moves the counter
  tick = true,           -- on_tick runs every game tick (20 per second)
  chat = false,
}

local corners = { "top-left", "top-right", "bottom-left", "bottom-right" }

function on_tick(t)
  local corner = storage.get("corner") or "top-left"
  local attack, use = game.cps()
  hud.panel(corner, {
    { text = "CPS: " .. attack, color = 0x55FF55 },
    { text = "Use: " .. use, color = 0xAAAAAA },
  })
end

function on_command(name, args)
  local corner = storage.get("corner") or "top-left"
  local next = 1
  for i, c in ipairs(corners) do
    if c == corner then next = i % #corners + 1 end
  end
  storage.set("corner", corners[next])
  return "CPS counter moved to " .. corners[next]
end
