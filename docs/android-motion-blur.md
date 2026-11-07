# Camera motion blur (Android dev build)

Options > Horizon Settings > Motion Blur: On / Off. Default Off.
The same option appears in Advanced > Display. It saves per game profile in
horizon-graphics.txt as motionBlur:true/false.

This is camera-turn blur, not frame generation, FPS improvement or full
per-object motion blur. A short, five-tap blur of the current world image follows
camera rotation. The hand and HUD are drawn after the pass and remain sharp.
It uses no depth texture reads or previous-image blending, for the OpenGL ES
path and to avoid accumulated trails. It resets on title screens, shader packs,
long frame gaps and large camera cuts. With a stationary camera it stays sharp.

Shader packs with their own post-processing suspend this built-in effect.
Turn the shader pack off to use it. TAA/FXAA and resolution scale are retained.
When Motion Blur is Off, it does not force a world target or a new render pass.

The effect adds texture sampling and GPU/bandwidth work while On; it can reduce
FPS, increase heat or make fast aiming harder. Turn it Off if uncomfortable.
Host tests cover shader validation, GLES310 translation, GPU pipelines, pixel
changes while turning, stationary sharpness and resetting. A synthetic scene
is not a test on the Infinix X6726. No phone FPS result is claimed.

On the phone, compare the same world and camera sweep with Off, On, Off in
OpenGL ES at unchanged resolution, render distance and FPS cap. Check stationary
sharpness, rapid turns, pause/resume, changing worlds and the hand/HUD. Existing
Android Performance recording can capture frame-time reports, but recording
itself adds overhead. Report actual frame times rather than a made-up FPS gain.
