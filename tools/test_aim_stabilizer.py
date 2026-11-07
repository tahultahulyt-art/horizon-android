#!/usr/bin/env python3
"""Exercise real touch look filtering without an APK or any player files."""
from pathlib import Path
import subprocess,tempfile
root=Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix='aim-stabilizer-') as d:
    target=Path(d)/'tests'
    subprocess.run(['rustc','--edition','2024','--test',str(root/'crates/horizon-client-render/src/touch/stabilizer.rs'),'-o',str(target)],check=True)
    subprocess.run([str(target)],check=True)
    subprocess.run(['rustc','--edition','2024','--test',str(root/'crates/horizon-client-render/src/touch/assist.rs'),'-o',str(target)],check=True)
    subprocess.run([str(target)],check=True)

    subprocess.run(['rustc','--edition','2024','--test',str(root/'tests/touch_assist_policy.rs'),'-o',str(target)],check=True)
    subprocess.run([str(target)],check=True)
