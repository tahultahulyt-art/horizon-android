#!/usr/bin/env python3
"""Run Kusobo installation tests without loading Android or touching player files."""
from pathlib import Path
import subprocess, tempfile
root=Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix="kusobo-tests-") as work:
    target=Path(work)/"tests"
    subprocess.run(["rustc", "--edition", "2024", "--test", str(root/"tests/kusobo_install.rs"), "-o", str(target)],check=True)
    subprocess.run([str(target), "--test-threads=1"],check=True)
