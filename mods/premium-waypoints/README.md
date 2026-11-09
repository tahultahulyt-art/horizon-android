# Premium Waypoints

Original client-only mod for Risiya's waypoint API build. No Lunar code or assets.
Install the Risiya update over your current app. Import premium-waypoints.hmod in
Kusobo Loader, enable it, and enter a world. Points are local, not shared with other players.

- `/wp add home cyan` saves your position.
- `/wp add mine 100 64 -200 FF77BB` saves typed coordinates and hex color.
- `/wp list` lists this world/server and dimension's points and distances.
- `/wp hide home` and `/wp show home` hide/show it.
- `/wp edit home rename base`
- `/wp edit base color purple`
- `/wp edit base coords 10 70 20`
- `/wp edit base beam off`
- `/wp remove base`
- `/wp beam off` disables all beams for lower rendering cost; `/wp beam on` restores them.

Colors: cyan, red, green, blue, purple, pink, yellow, white, orange, or six-digit RGB hex.
Names: 1-24 letters/numbers/_/-. Up to 32 points per world/dimension.
World/server and dimension separation is automatic; points survive game restarts.
A changed server address or renamed/moved local save gets a different identity.
A server offering multiple separate worlds under the same address and dimension ID
cannot be distinguished automatically: these share a waypoint set.

Beams are 256 blocks high, respect terrain depth and are culled beyond 1024 blocks.
Labels stay readable at distance; markers behind the camera are not drawn.
Labels are overlaid and can be seen through terrain when looking toward the point.
F1 hides markers. Labels hide while a menu/chat is open.
No teleporting, minimap dots, death markers or server sharing.

Performance: no waypoint geometry/labels when the mod is off. Retained drawing,
no per-frame Lua and no storage I/O on ticks. Active beams cost GPU work.
Phone appearance/FPS, OpenGL ES and server reconnects require device testing.
