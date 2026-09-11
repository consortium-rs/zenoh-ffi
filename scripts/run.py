#!/usr/bin/env python3
"""Compatibility entry point; Rust owns process orchestration and validation."""
import os
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]
os.execvp("cargo", ["cargo", "run", "--locked", "--manifest-path",
                     str(root / "rust/Cargo.toml"), "--", *sys.argv[1:]])
