# Shield Mode

Shield Mode is a default-off touch camera filter. A separate **Target Stick**
toggle is also default off, requires Shield Mode and crosshair controls, and
has a 0-30% strength slider (15% initial value). Both are in the launcher's
Settings > Quick controls and the in-game gear > Quick controls/full touch
settings. The preferences live in the existing per-profile `horizon-touch.json`.
Old files default both toggles to off. No world/mod migration is involved.

## Smoothing

Off follows the previous immediate camera input path. On queues the existing
sensitivity-scaled camera delta and drains it with an exponential response on
each game frame. The time constant is 8ms for small pending deltas, 3ms over
3 degrees, making fast swipes less delayed. This adds a small input delay;
it does not increase sensitivity or guarantee hits.

Each output is deducted from the queue. With normal frame gaps, net rotation
is conserved before the existing pitch clamp. Finger release drains the remaining
real motion in the next frame, then no further movement is emitted. Clearing
touch state, menus, focus loss or a frame gap over 100ms discards pending motion
rather than replaying it later. Mouse, controller and gyro paths are unchanged.

## Optional Target Stick (aim assistance)

This is real target-aware assistance, not sensitivity slowdown, a cheat-detection
bypass or a promise of server permission. Follow server rules: aim assist may be
prohibited even at low strength. It never attacks or changes combat packets.
It cannot distinguish friends from enemies and considers other visible players,
not passive mobs, armor stands, dropped items or invisible/spectator/dead players.
It does not change shield item behavior.

Only frames with fresh real look-finger movement can receive a correction.
A stationary finger, release, menu, pause or zero strength produces no assist.
Players must be within normal melee reach, capped at 3 blocks, and within
2.5 degrees of the crosshair ray near their body. A block ray to the candidate
body point must be clear. No target identity, history or lock is retained.

The correction aims only toward the nearest body point to the view ray. If the
crosshair is already inside the body, there is no pull to the centre/head.
The correction is capped by all of: selected fraction of angular error, 8 degrees
per second and half the finger's turn this frame. Deliberate swipes over
150 degrees/second or input away from the corrective direction disengage it.
Each axis follows only existing finger movement on that same axis. Horizontal
dragging cannot add vertical correction; moving away disengages that axis.
This is a mild edge correction, not a full target lock. It cannot guarantee
tracking, hits, PC-like feel or server acceptance.

## Checks

`python3 tools/test_aim_stabilizer.py` compiles and runs the actual Rust filter
and assistance math tests without network or player files. Covers off pass-through,
reset, pause/invalid data, zero-input stillness, slow/fast response, net distance
at 30/60/90/120/240Hz, uneven frames/reversal, fresh-input gating, near-edge radius,
behind/out-of-range rejection, no centre snap, strength/speed limits, turn-away
release and yaw wrap. Shared settings tests cover default-off, roundtrip and limits.

Synthetic response images are math checks, not phone gameplay. Full Android
compile, UI layout and real device feel remain separate checks. No GPU pass or
Vulkan/GLES-specific feature is added.

## Attack tap stability (dev20)

The attack button still presses attack immediately. With Look while attacking
on, camera drag now waits for 0.8% of surface height of finger travel (equal
physical threshold on both axes). Small tap jitter is discarded. Crossing the
threshold applies only the travel beyond it, never a catch-up jump; subsequent
intentional dragging works normally. Release and clearing touch state reset it
per finger. This changes neither attack timing nor separate look-finger input.

This fixes a demonstrated code path for upward movement from an attack finger,
not proof of the cause of a particular gameplay recording. Gyroscope and
render-only hurt tilt/bobbing are independent settings.
