-- Original client-only waypoint mod. Menu: key N (or a touch button on N), /wp, or Options > Mods.
MOD = { commands = {"wp"}, tick = true }
local points = {}
local loaded = false
local current_world = nil
local beam_enabled = true
local palette = {cyan=0x55FFFF, red=0xFF5555, green=0x55FF55, blue=0x5599FF,
  purple=0xCC77FF, pink=0xFF77BB, yellow=0xFFFF55, white=0xFFFFFF, orange=0xFFAA44}
local COLOR_ORDER = {"cyan","red","green","blue","purple","pink","yellow","white","orange"}
local MENU_LIST, MENU_ADD, MENU_COLOR, MENU_BEAM, MENU_HIDE, MENU_RENAME, MENU_REMOVE, MENU_BEAMS = 1, 2, 3, 4, 5, 6, 7, 8
local selected = 1
local function fail(message) error(message, 0) end
local function valid_name(s)
  return s and #s >= 1 and #s <= 24 and s:match("^[%w_-]+$")
end
local function number(s)
  local v = tonumber(s)
  if not v or v ~= v or math.abs(v) > 30000000 then fail("Coordinates must be numbers within +/-30000000") end
  return v
end
local function color(s)
  if not s then return 0x55FFFF end
  s = s:lower()
  if palette[s] then return palette[s] end
  s = s:gsub("^#", ""):gsub("^0x", "")
  if #s ~= 6 or not s:match("^%x+$") then fail("Use a color name or six-digit hex, e.g. cyan or FF77BB") end
  return tonumber(s, 16)
end
local function load()
  if loaded then return end
  if not game.world_id or not hud.waypoints then fail("Update Risiya to the waypoint API build first") end
  loaded = true
  beam_enabled = storage.get("beam") ~= "off"
  local text = storage.get("points_v1") or ""
  for line in text:gmatch("[^\n]+") do
    local world, dim, name, x, y, z, c, hidden, beam = line:match("^([^\t]+)\t([^\t]+)\t([^\t]+)\t([^\t]+)\t([^\t]+)\t([^\t]+)\t(%d+)\t([01])\t([01])$")
    x,y,z,c=tonumber(x),tonumber(y),tonumber(z),tonumber(c)
    if world and dim and valid_name(name) and x and y and z and c and #points < 4096 then
      points[#points+1]={world_id=world, dimension=dim, name=name, x=x,y=y,z=z,
        color=c, hidden=hidden=="1", beam=beam=="1"}
    end
  end
end
local function save()
  local lines={}
  for _,p in ipairs(points) do
    lines[#lines+1]=table.concat({p.world_id,p.dimension,p.name,string.format("%.6f",p.x),
      string.format("%.6f",p.y),string.format("%.6f",p.z),tostring(p.color),p.hidden and "1" or "0",p.beam and "1" or "0"},"\t")
  end
  storage.set("points_v1",table.concat(lines,"\n"))
end
local function context()
  local player=game.player()
  local world=game.world_id()
  if not player or not world then fail("Join a world or server first") end
  return world,player
end
local function publish()
  local player=game.player()
  local world=game.world_id()
  local visible={}
  if player and world then
    for _,p in ipairs(points) do
      if p.world_id==world and p.dimension==player.dimension and not p.hidden then
        visible[#visible+1]={name=p.name,world_id=world,dimension=p.dimension,x=p.x,y=p.y,z=p.z,
          color=p.color,beam=beam_enabled and p.beam}
      end
    end
  end
  hud.waypoints(visible)
  current_world=world
end
local function find(name,world,dim)
  for i,p in ipairs(points) do
    if p.name:lower()==(name or ""):lower() and p.world_id==world and p.dimension==dim then return p,i end
  end
  fail("No waypoint with that name in this world/dimension. Use /wp list")
end
local function world_points(world, dim)
  local out = {}
  for i, p in ipairs(points) do
    if p.world_id == world and p.dimension == dim then out[#out+1] = i end
  end
  return out
end
local function color_name(c)
  for _, n in ipairs(COLOR_ORDER) do if palette[n] == c then return n end end
  return string.format("#%06X", c or 0)
end
local function menu_widgets()
  local world, player = context()
  local dim = player.dimension
  local here = world_points(world, dim)
  if #here == 0 then selected = 1 end
  if selected > #here then selected = #here end
  local rows = {}
  for _, i in ipairs(here) do
    local p = points[i]
    local d = math.sqrt((p.x-player.x)^2 + (p.y-player.y)^2 + (p.z-player.z)^2)
    rows[#rows+1] = p.name .. string.format(" - %.0f m", d) .. (p.hidden and " [hidden]" or "")
  end
  if #rows == 0 then rows = {"(no waypoints here)"} end
  local w = {
    {type="list", id=MENU_LIST, rows=rows, selected=(#here>0) and selected or nil},
    {type="button", id=MENU_ADD, label="Add point here"},
  }
  if #here > 0 then
    local p = points[here[selected]]
    w[#w+1] = {type="button", id=MENU_COLOR, label="Color: " .. color_name(p.color)}
    w[#w+1] = {type="button", id=MENU_BEAM, label="Beam: " .. (p.beam and "on" or "off")}
    w[#w+1] = {type="button", id=MENU_HIDE, label=p.hidden and "Show" or "Hide"}
    w[#w+1] = {type="button", id=MENU_RENAME, label="Rename (chat: /wp rename "..p.name.." new)"}
    w[#w+1] = {type="button", id=MENU_REMOVE, label="Remove '"..p.name.."'"}
  end
  w[#w+1] = {type="button", id=MENU_BEAMS, label="All beams: " .. (beam_enabled and "on" or "off")}
  return w
end
local function open_menu()
  ui.open_screen({title="Waypoints", widgets=menu_widgets()})
end
local function next_color(c)
  for i, n in ipairs(COLOR_ORDER) do
    if palette[n] == c then return palette[COLOR_ORDER[(i % #COLOR_ORDER) + 1]] end
  end
  return palette[COLOR_ORDER[1]]
end
function on_ui_select(id, row)
  load()
  local world, player = context()
  local dim = player.dimension
  local here = world_points(world, dim)
  if id == MENU_LIST then
    if #here > 0 and row >= 1 and row <= #here then selected = row end
  elseif id == MENU_ADD then
    if #here >= 32 then chat.show("Waypoint menu: this dimension is full (32)") return end
    local n = 1
    local taken = {}
    for _, i in ipairs(here) do taken[points[i].name:lower()] = true end
    while taken["point"..n] do n = n + 1 end
    local name = "point"..n
    points[#points+1] = {name=name, world_id=world, dimension=dim, x=player.x, y=player.y, z=player.z,
      color=palette.cyan, hidden=false, beam=true}
    save(); publish()
    chat.show("Saved '"..name.."' - rename with /wp rename "..name.." <new>")
  elseif #here == 0 then
    return
  else
    local p = points[here[selected]]
    if not p then return end
    if id == MENU_COLOR then p.color = next_color(p.color)
    elseif id == MENU_BEAM then p.beam = not p.beam
    elseif id == MENU_HIDE then p.hidden = not p.hidden
    elseif id == MENU_RENAME then
      chat.show("Type: /wp rename "..p.name.." <new name>")
    elseif id == MENU_REMOVE then
      table.remove(points, here[selected])
      save(); publish(); open_menu(); return
    end
    save(); publish()
  end
  open_menu()
end
function on_ui_settings()
  local ok, w = pcall(menu_widgets)
  if ok then ui.set_settings(w) end
end
input.register_key("open", "Waypoints", "key.keyboard.n")
function on_key(name, down)
  if name == "open" and down then
    load()
    local ok = pcall(open_menu)
    if not ok then chat.show("Waypoint menu needs a world (join first)") end
  end
end
function on_tick(t)
  load()
  -- Retained primitives need no Lua per rendered frame. No storage I/O on ticks.
  -- Republish on dimension/world change too; engine also filters stale markers.
  local p=game.player()
  local world=game.world_id()
  local dim=p and p.dimension or nil
  if world~=current_world or dim~=_G.wp_dimension then
    _G.wp_dimension=dim
    publish()
  end
end
function on_command(command,args)
  load()
  local a={}
  for word in args:gmatch("%S+") do a[#a+1]=word end
  local sub=(a[1] or "menu"):lower()
  if sub=="menu" and ui then open_menu() return end
  if sub=="help" then
    return "/wp add name [color] OR /wp add name x y z [color]\n/wp list | /wp remove name | /wp hide name | /wp show name\n/wp edit name color value | coords x y z | rename newname | beam on/off\n/wp beam on/off (all beams)"
  end
  if sub=="beam" then
    if a[2]~="on" and a[2]~="off" then fail("Use /wp beam on or /wp beam off") end
    beam_enabled=a[2]=="on"; storage.set("beam",a[2]); publish()
    return "Waypoint beams "..a[2]
  end
  local world,player=context()
  local dim=player.dimension
  if sub=="add" then
    local name=a[2]
    if not valid_name(name) then fail("Name: 1-24 letters, numbers, _ or -") end
    local count=0
    for _,p in ipairs(points) do
      if p.world_id==world and p.dimension==dim then
        count=count+1
        if p.name:lower()==name:lower() then fail("That name exists here. Use /wp edit or /wp remove") end
      end
    end
    if count>=32 then fail("Limit: 32 points per world/dimension") end
    if #points>=4096 then fail("Waypoint storage is full. Remove old points first") end
    local x,y,z,c=player.x,player.y,player.z,a[3]
    if #a>=5 then
      if #a~=5 and #a~=6 then fail("Use /wp add name x y z [color]") end
      x,y,z=number(a[3]),number(a[4]),number(a[5]);c=a[6]
    elseif #a>3 then fail("Use /wp add name [color] OR /wp add name x y z [color]") end
    points[#points+1]={name=name,world_id=world,dimension=dim,x=x,y=y,z=z,color=color(c),hidden=false,beam=true}
    save();publish()
    return "Saved waypoint '"..name.."' at "..string.format("%.1f %.1f %.1f",x,y,z)
  elseif sub=="list" then
    local lines={"Waypoints in this dimension:"}
    for _,p in ipairs(points) do
      if p.world_id==world and p.dimension==dim then
        local d=math.sqrt((p.x-player.x)^2+(p.y-player.y)^2+(p.z-player.z)^2)
        lines[#lines+1]=p.name..string.format(" - %.0f m",d)..(p.hidden and " [hidden]" or "")
      end
    end
    if #lines==1 then return "No waypoints here. /wp add home cyan" end
    return table.concat(lines,"\n")
  elseif sub=="remove" or sub=="delete" then
    local p,i=find(a[2],world,dim); table.remove(points,i); save();publish();return "Removed '"..p.name.."'"
  elseif sub=="hide" or sub=="show" then
    local p=find(a[2],world,dim)
    p.hidden=sub=="hide"
    save();publish();return p.name..(p.hidden and " hidden" or " shown")
  elseif sub=="edit" then
    local p=find(a[2],world,dim)
    if a[3]=="color" then p.color=color(a[4])
    elseif a[3]=="coords" then
      local x,y,z=number(a[4]),number(a[5]),number(a[6]);p.x,p.y,p.z=x,y,z
    elseif a[3]=="beam" then
      if a[4]~="on" and a[4]~="off" then fail("Use /wp edit name beam on/off") end
      p.beam=a[4]=="on"
    elseif a[3]=="rename" then
      if not valid_name(a[4]) then fail("New name: 1-24 letters, numbers, _ or -") end
      for _,q in ipairs(points) do
        if q~=p and q.world_id==world and q.dimension==dim and q.name:lower()==a[4]:lower() then fail("That name exists") end
      end
      p.name=a[4]
    else fail("Use /wp edit name color value | coords x y z | rename newname | beam on/off") end
    save();publish();return "Updated '"..p.name.."'"
  end
  fail("Unknown waypoint command. /wp help")
end
