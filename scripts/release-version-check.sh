#!/usr/bin/env bash
# Verify all release-version surfaces agree.
set -euo pipefail

ROOT="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
ROOT="$(cd "$ROOT" && pwd)"

ROOT="$ROOT" python3 <<'PY'
import json
import os
import tomllib
from pathlib import Path

root = Path(os.environ["ROOT"])
version_path = root / "VERSION"
if not version_path.is_file():
    raise SystemExit("missing VERSION")

expected = version_path.read_text(encoding="utf-8").strip()
if not expected:
    raise SystemExit("VERSION must not be empty")

checks = {
    "plugin.json": json.loads((root / "plugin.json").read_text(encoding="utf-8"))["version"],
    ".codex-plugin/plugin.json": json.loads((root / ".codex-plugin/plugin.json").read_text(encoding="utf-8"))["version"],
    "MANIFEST.toml": tomllib.loads((root / "MANIFEST.toml").read_text(encoding="utf-8"))["plugin_version"],
    "rust/task-registry-flow-cli/Cargo.toml": tomllib.loads((root / "rust/task-registry-flow-cli/Cargo.toml").read_text(encoding="utf-8"))["package"]["version"],
}

for path, value in checks.items():
    if value != expected:
        raise SystemExit(f"version mismatch: {path}={value}, VERSION={expected}")

print(f"release version ok: {expected}")
PY
