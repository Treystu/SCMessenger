#!/usr/bin/env python3
"""Resolve the JEV gate key from every location this lane's doctrine names.

Why this module exists (#622, key half). The admitted Harness resolves a JEV
key from two env files and three environment variables only
(``harness/config.py:resolve_jev_key``):

  * ``~/.config/harness/jev.env``  (Harness-configured key)
  * ``~/.config/scmorc/jev.env``   (SCM ORC folder keys)
  * ``HARNESS_JEV_KEY`` / ``TYPESAFE_API_KEY`` / ``JEV_API_KEY``

The lane's key doctrine names a third location, ``~/.typesafe_key`` -- a bare
key file -- and operators keep shell-sourced key files, whose lines read
``export TYPESAFE_API_KEY=...``. Neither shape is readable by the Harness: the
bare file is never opened, and the ``export`` prefix makes the variable name
miss the exact-match list, so the key line is skipped and the gate reports
``typesafe_key=False``. That is an absence claim about a key that is present,
which is the failure this module removes.

Contract:

  * ``probe()`` returns one row per location in doctrine order, with a status
    and never any key material -- safe to print in a stand-up.
  * ``resolve_key()`` returns ``(key, location, probe_rows)``. ``location`` names
    the winning location only (path, plus the variable name for env-file hits);
    it never contains the key, and it is the only thing callers may print.
  * ``key`` is handed back for the API call only. Nothing here logs, prints or
    stores it, and no file or environment variable is created or mutated.
"""
from __future__ import annotations

import os
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# The variable names the Harness accepts, in its own order.
JEV_ENV_VARS: Tuple[str, ...] = (
    "HARNESS_JEV_KEY",
    "TYPESAFE_API_KEY",
    "JEV_API_KEY",
)

# Doctrine order: Harness-configured key, SCM ORC folder keys, ~/.typesafe_key,
# then the process environment (which the Harness also consults last).
KEY_FILE_LOCATIONS: Tuple[Tuple[str, str], ...] = (
    ("harness-config", ".config/harness/jev.env"),
    ("scm-orc", ".config/scmorc/jev.env"),
    ("typesafe-key-file", ".typesafe_key"),
)

ABSENT = "absent"
NO_KEY_LINE = "present, no usable key line"
RESOLVED = "resolved"


def _accept(value: str) -> str:
    """Trim whitespace and one layer of matching quotes from a candidate value."""
    value = value.strip()
    for quote in ('"', "'"):
        if len(value) >= 2 and value.startswith(quote) and value.endswith(quote):
            value = value[1:-1].strip()
            break
    return value


def _read_env_style(path: Path) -> Tuple[Optional[str], Optional[str], bool]:
    """Read ``KEY=value`` or ``export KEY=value`` from an env-style key file.

    Returns ``(value, variable_name, present)``. ``present`` distinguishes "the
    file does not exist" from "the file exists but carries nothing usable", so a
    present-but-unparsed key file is never reported as a simple absence.
    """
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None, None, False
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("export "):
            line = line[len("export "):].strip()
        if "=" not in line:
            continue
        name, value = line.split("=", 1)
        name = name.strip()
        if name not in JEV_ENV_VARS:
            continue
        value = _accept(value)
        if value:
            return value, name, True
    return None, None, True


def _read_bare_key(path: Path) -> Tuple[Optional[str], Optional[str], bool]:
    """Read a bare key file, also accepting an env-style line inside it."""
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None, None, False
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("export "):
            line = line[len("export "):].strip()
        if "=" in line:
            name, value = line.split("=", 1)
            name = name.strip()
            if name not in JEV_ENV_VARS:
                continue
            value = _accept(value)
            if value:
                return value, name, True
            continue
        value = _accept(line)
        if value:
            return value, None, True
    return None, None, True


def probe(
    home: Optional[Path] = None,
    env: Optional[Dict[str, str]] = None,
) -> List[Tuple[str, str]]:
    """One row per location, in doctrine order: ``(location, status)``."""
    home_dir = Path(home) if home is not None else Path(os.path.expanduser("~"))
    environ = env if env is not None else os.environ

    rows: List[Tuple[str, str]] = []
    for label, relative in KEY_FILE_LOCATIONS:
        path = home_dir / relative
        if relative.endswith(".typesafe_key"):
            value, name, present = _read_bare_key(path)
        else:
            value, name, present = _read_env_style(path)
        if value:
            shown = "~/{} ({})".format(relative, name) if name else "~/{}".format(relative)
            rows.append((shown, RESOLVED))
        elif present:
            rows.append(("~/{}".format(relative), NO_KEY_LINE))
        else:
            rows.append(("~/{}".format(relative), ABSENT))
    for variable in JEV_ENV_VARS:
        rows.append((variable, RESOLVED if environ.get(variable) else "unset"))
    return rows


def resolve_key(
    home: Optional[Path] = None,
    env: Optional[Dict[str, str]] = None,
) -> Tuple[Optional[str], Optional[str], List[Tuple[str, str]]]:
    """Resolve ``(key, location, probe_rows)``; ``location`` never holds the key."""
    home_dir = Path(home) if home is not None else Path(os.path.expanduser("~"))
    environ = env if env is not None else os.environ

    for label, relative in KEY_FILE_LOCATIONS:
        path = home_dir / relative
        if relative.endswith(".typesafe_key"):
            value, name, _present = _read_bare_key(path)
        else:
            value, name, _present = _read_env_style(path)
        if value:
            shown = "~/{} ({})".format(relative, name) if name else "~/{}".format(relative)
            return value, shown, probe(home_dir, environ)
    for variable in JEV_ENV_VARS:
        value = environ.get(variable)
        if value and _accept(value):
            return _accept(value), variable, probe(home_dir, environ)
    return None, None, probe(home_dir, environ)


def describe_rows(rows: List[Tuple[str, str]]) -> str:
    """Render probe rows for a log line: locations and statuses, never values."""
    return "; ".join("{}: {}".format(location, status) for location, status in rows)


if __name__ == "__main__":  # pragma: no cover - manual probe, prints no key
    _key, _location, _rows = resolve_key()
    print("[INFO] key resolution: {}".format(_location or "none"))
    print("[INFO] locations: {}".format(describe_rows(_rows)))
