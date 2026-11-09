-- CK Kits: /ck save <name>, /ck load <name>, /ck list, /ck delete <name>.
-- Kits are global (same in every world and on your own server) and work
-- only in creative mode. A kit holds the whole inventory: hotbar, main
-- slots, armor and offhand, with enchants and names.

MOD = { commands = { "ck" } }

local MAX_KITS = 40

local function clean(name)
  name = string.lower(name or "")
  name = string.gsub(name, "%s+", "_")
  if name == "" or #name > 24 or string.find(name, "[^a-z0-9_%-]") then
    return nil
  end
  return name
end

local function names()
  local out = {}
  for n in string.gmatch(storage.get("index") or "", "[^\n]+") do
    out[#out + 1] = n
  end
  return out
end

local function save_names(list)
  table.sort(list)
  storage.set("index", table.concat(list, "\n"))
end

local function has(list, name)
  for _, n in ipairs(list) do
    if n == name then return true end
  end
  return false
end

local USAGE = "Use: /ck save <name>, /ck load <name>, /ck list, /ck delete <name>"

function on_command(cmd, args)
  local sub, rest = string.match(args or "", "^%s*(%S+)%s*(.-)%s*$")
  if not sub then return USAGE end
  sub = string.lower(sub)

  if sub == "list" then
    local list = names()
    if #list == 0 then return "No kits saved yet. /ck save <name>" end
    return "Kits: " .. table.concat(list, ", ")
  end

  if sub ~= "save" and sub ~= "load" and sub ~= "delete" then
    error(USAGE, 0)
  end
  if not inventory.creative() then
    error("Kits work only in creative mode", 0)
  end
  local name = clean(rest)
  if not name then
    error("Give a kit name: letters, numbers, - or _ (max 24)", 0)
  end
  local list = names()

  if sub == "save" then
    if not has(list, name) and #list >= MAX_KITS then
      error("Too many kits (max " .. MAX_KITS .. "). /ck delete <name> first", 0)
    end
    local lines = {}
    for _, e in ipairs(inventory.read()) do
      lines[#lines + 1] = e.slot .. " " .. e.item
    end
    storage.set("kit." .. name, table.concat(lines, "\n"))
    if not has(list, name) then
      list[#list + 1] = name
      save_names(list)
    end
    return "Saved kit '" .. name .. "' (" .. #lines .. " stacks)"
  end

  if sub == "load" then
    local data = storage.get("kit." .. name)
    if not data then error("No kit named '" .. name .. "'. /ck list") end
    local entries = {}
    for line in string.gmatch(data, "[^\n]+") do
      local slot, item = string.match(line, "^(%d+) (.+)$")
      if slot then
        entries[#entries + 1] = { slot = tonumber(slot), item = item }
      end
    end
    inventory.replace(entries)
    return "Loaded kit '" .. name .. "' (" .. #entries .. " stacks)"
  end

  -- delete
  if not has(list, name) then error("No kit named '" .. name .. "'") end
  storage.delete("kit." .. name)
  local kept = {}
  for _, n in ipairs(list) do
    if n ~= name then kept[#kept + 1] = n end
  end
  save_names(kept)
  return "Deleted kit '" .. name .. "'"
end
