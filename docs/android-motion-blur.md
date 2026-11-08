# Camera motion blur (Android dev build)

Options > Horizon Settings > Motion Blur: On / Off. Default Off.
The same option appears in Advanced > Display. It saves per game profile in
horizon-graphics.txt as motionBlur:true/false.

This is camera-turn blur, not frame generation, FPS improvement or full
per-object motion blur. It is frame accumulation, the Lunar Client style
(`/motionbl 55`): each shown frame keeps 55% of the previous shown frame, so
turns smear smoothly in every direction at any turn speed. The history is an
Rgba8 render target; a fixed per-pixel noise dithers the stored frame so
repeated blending shows no banding. The hand and HUD are drawn after the pass
and remain sharp.
It uses no depth texture reads or object motion vectors, for the OpenGL ES
path. The blend resets on title screens, shader packs, frame gaps over 0.2s
and camera cuts (teleports, respawns, F5's flip), so old scenes never smear
into new ones. With a stationary camera the blend converges back to sharp
within a few frames.

Shader packs with their own post-processing suspend this built-in effect.
Turn the shader pack off to use it. TAA/FXAA and resolution scale are retained.
When Motion Blur is Off, it does not force a world target or a new render pass.

The effect adds one full-screen copy and blend per frame while On; it can
reduce FPS, increase heat or make fast aiming harder. Turn it Off if
uncomfortable. Trail length depends on frame rate: at low FPS the same
turn leaves a longer trail. Host tests cover shader validation, GLES310
translation, GPU pipelines, pixel changes while turning, stationary
convergence, and resets on title, camera flip and teleport. A synthetic scene
is not a test on the Infinix X6726. No phone FPS result is claimed.

On the phone, compare the same world and camera sweep with Off, On, Off in
OpenGL ES at unchanged resolution, render distance and FPS cap. Check
stationary sharpness, rapid turns, F5 toggling, pause/resume, changing worlds
and the hand/HUD. Existing Android Performance recording can capture
frame-time reports, but recording itself adds overhead. Report actual frame
times rather than a made-up FPS gain.
