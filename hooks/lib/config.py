"""
config.py - Load and validate profile configuration.

Responsible for:
  - Reading a JSON profile file.
  - Resolving `extends` references to a base profile, with a single
    level of inheritance.
  - Deep-merging so partial overrides do not silently drop the base's
    nested keys.
  - Refusing unsafe `extends` values (no path traversal, no shell
    metacharacters — only simple identifiers).
  - Containing base-profile lookups within the config root directory.
"""
from __future__ import annotations

import json
import re
from pathlib import Path

# Whitelist of filename characters allowed in `extends` values.
# Blocks path traversal and shell metacharacters.
_EXTENDS_PATTERN = re.compile(r"^[A-Za-z0-9_\-]+$")


def _deep_merge(base: dict, override: dict) -> dict:
    """Recursively merge `override` into `base`.

    Nested dicts have their keys preserved unless explicitly overridden.
    Lists and scalars are replaced wholesale. Returns a new dict; does
    not mutate inputs.
    """
    result = dict(base)
    for k, v in override.items():
        if k in result and isinstance(result[k], dict) and isinstance(v, dict):
            result[k] = _deep_merge(result[k], v)
        else:
            result[k] = v
    return result


def load_config(config_path: str) -> dict:
    """Load a JSON config, merging profiles via single-level `extends`.

    Security:
      - `extends` must be a simple identifier (no path separators, no
        traversal). Rejected with ValueError otherwise.
      - Base-profile resolution is confined to config_path's parent
        directory and its parent (for the profiles/ subdirectory
        pattern). No upward traversal beyond that.
    """
    path = Path(config_path).resolve()
    cfg = json.loads(path.read_text(encoding="utf-8"))

    if "extends" in cfg:
        base_name = cfg.pop("extends")
        if not isinstance(base_name, str) or not _EXTENDS_PATTERN.match(base_name):
            raise ValueError(
                f"invalid `extends` value: must match ^[A-Za-z0-9_-]+$, got {base_name!r}"
            )
        root = path.parent.parent.resolve()
        candidates = [
            (path.parent / f"{base_name}.json").resolve(),
            (path.parent.parent / f"{base_name}.json").resolve(),
        ]
        base_path = None
        for c in candidates:
            try:
                c.relative_to(root)
            except ValueError:
                continue
            if c.exists() and c.is_file():
                base_path = c
                break
        if base_path is None:
            raise FileNotFoundError(f"extends base not found: {base_name}")
        base = json.loads(base_path.read_text(encoding="utf-8"))
        cfg = _deep_merge(base, cfg)
    return cfg
