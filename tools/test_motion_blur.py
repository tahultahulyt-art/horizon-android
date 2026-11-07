#!/usr/bin/env python3
"""Validate motion blur, GLES translation and actual GPU pixel output. No player files.

Use WGPU_BACKEND=gl for the host GL backend. Host/software GPU tests are not
measurements of any phone. HORIZON_BLUR_TEST_OUTPUT saves optional640x320 RGBA
captures. Without it the test creates no image files.
"""
from pathlib import Path
import os
import subprocess
MANIFEST = '[package]\nname = "horizon-blur-tests"\nversion = "0.1.0"\nedition = "2024"\n[workspace]\n[dependencies]\nwgpu = { version = "30.0.1", default-features = false, features = ["std", "parking_lot", "vulkan", "gles", "wgsl"] }\npollster = "0.4"\nbytemuck = { version = "1", features = ["derive"] }\nglam = "0.30"\nlog = "0.4"\nnaga = { version = "30.0.1", features = ["wgsl-in", "glsl-out"] }\n'
ROOT = Path(__file__).resolve().parents[1]
work = ROOT / '.local/android/motion-blur-tests'
(work / 'src').mkdir(parents=True, exist_ok=True)
(work / 'Cargo.toml').write_text(MANIFEST)
(work / 'src/lib.rs').write_text(f'#[path = "{ROOT / "tests/motion_blur_shader.rs"}"] pub mod checks;\npub use checks::world_post;\n')
(work / 'src/main.rs').write_text((ROOT / 'tests/motion_blur_pixels.rs').read_text())
env = dict(os.environ)
env.setdefault('XDG_RUNTIME_DIR', '/tmp')
subprocess.run(['cargo','test','--manifest-path',str(work / 'Cargo.toml')],check=True,env=env)
subprocess.run(['cargo','run','--manifest-path',str(work / 'Cargo.toml')],check=True,env=env)
