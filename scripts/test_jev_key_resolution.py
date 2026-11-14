#!/usr/bin/env python3
"""Regression tests for #622's key half: a present key must never read absent.

The failure mode these tests pin: the Harness resolves a JEV key from two env
files and three environment variables only, requires a bare ``KEY=value`` line,
and never opens ``~/.typesafe_key``. A key at a documented location can
therefore be present and still be reported missing.

Run: ``python3 scripts/test_jev_key_resolution.py``
"""
from __future__ import annotations

import os
import sys
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

import jev_key_resolution as keys  # noqa: E402

KEY_VALUE = "fixture-placeholder-not-a-real-key"
OTHER_VALUE = "fixture-second-placeholder-not-a-real-key"


def scratch(name: str) -> Path:
    """Repo-local scratch under tmp/ (AGENTS.md rule 2: never the system tmp)."""
    base = SCRIPT_DIR.parent / "tmp"
    base.mkdir(parents=True, exist_ok=True)
    directory = base / "jev-keyresolution-{}-{}".format(name, os.getpid())
    if directory.exists():
        import shutil

        shutil.rmtree(directory)
    directory.mkdir(parents=True)
    return directory


def write(path: Path, text: str) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


class DocumentedLocationTests(unittest.TestCase):
    """Each documented location resolves on its own, with no other key present."""

    def tearDown(self) -> None:
        home = getattr(self, "home", None)
        if home is not None:
            import shutil

            shutil.rmtree(home, ignore_errors=True)

    def test_bare_typesafe_key_file_resolves(self):
        self.home = scratch("bare")
        write(self.home / ".typesafe_key", KEY_VALUE + "\n")
        key, location, rows = keys.resolve_key(home=self.home, env={})
        self.assertEqual(key, KEY_VALUE)
        self.assertIn(".typesafe_key", location)
        self.assertTrue(any(status == keys.RESOLVED for _loc, status in rows))

    def test_export_prefixed_scm_orc_key_resolves(self):
        self.home = scratch("export")
        write(
            self.home / ".config" / "scmorc" / "jev.env",
            "# shell-sourced key file\n"
            "export TYPESAFE_API_KEY=" + KEY_VALUE + "\n",
        )
        key, location, _rows = keys.resolve_key(home=self.home, env={})
        self.assertEqual(key, KEY_VALUE)
        self.assertIn("scmorc", location)
        self.assertIn("TYPESAFE_API_KEY", location)

    def test_harness_config_location_precedes_scm_orc(self):
        self.home = scratch("order")
        write(self.home / ".config" / "harness" / "jev.env", "HARNESS_JEV_KEY=" + KEY_VALUE + "\n")
        write(self.home / ".config" / "scmorc" / "jev.env", "TYPESAFE_API_KEY=" + OTHER_VALUE + "\n")
        key, location, _rows = keys.resolve_key(home=self.home, env={})
        self.assertEqual(key, KEY_VALUE, "the Harness-configured location is tried first")
        self.assertIn("harness", location)

    def test_environment_variable_is_the_last_resort(self):
        self.home = scratch("env")
        key, location, _rows = keys.resolve_key(
            home=self.home, env={"JEV_API_KEY": KEY_VALUE}
        )
        self.assertEqual(key, KEY_VALUE)
        self.assertEqual(location, "JEV_API_KEY")

    def test_quotes_and_trailing_space_are_stripped(self):
        self.home = scratch("quotes")
        write(self.home / ".config" / "harness" / "jev.env", 'TYPESAFE_API_KEY="' + KEY_VALUE + '"   \n')
        key, _location, _rows = keys.resolve_key(home=self.home, env={})
        self.assertEqual(key, KEY_VALUE)


class AbsenceIsExplicitTests(unittest.TestCase):
    def tearDown(self) -> None:
        home = getattr(self, "home", None)
        if home is not None:
            import shutil

            shutil.rmtree(home, ignore_errors=True)

    def test_nothing_present_reports_every_location(self):
        self.home = scratch("empty")
        key, location, rows = keys.resolve_key(home=self.home, env={})
        self.assertIsNone(key)
        self.assertIsNone(location)
        self.assertEqual(len(rows), len(keys.KEY_FILE_LOCATIONS) + len(keys.JEV_ENV_VARS))
        statuses = [status for _loc, status in rows]
        self.assertEqual(statuses.count(keys.ABSENT), len(keys.KEY_FILE_LOCATIONS))
        self.assertEqual(statuses.count("unset"), len(keys.JEV_ENV_VARS))

    def test_present_file_without_a_key_line_is_not_reported_absent(self):
        self.home = scratch("noline")
        write(self.home / ".config" / "harness" / "jev.env", "# only a comment\n\n")
        _key, _location, rows = keys.resolve_key(home=self.home, env={})
        status = dict(rows)["~/.config/harness/jev.env"]
        self.assertEqual(status, keys.NO_KEY_LINE,
                         "an existing key file with no usable line is a different fact from absence")

    def test_location_and_rows_never_carry_key_material(self):
        self.home = scratch("noleak")
        write(self.home / ".typesafe_key", KEY_VALUE + "\n")
        key, location, rows = keys.resolve_key(home=self.home, env={"JEV_API_KEY": OTHER_VALUE})
        self.assertEqual(key, KEY_VALUE)
        rendered = keys.describe_rows(rows)
        for secret in (KEY_VALUE, OTHER_VALUE):
            self.assertNotIn(secret, location)
            self.assertNotIn(secret, rendered)
            self.assertNotIn(secret, repr(rows))


class HarnessGapTests(unittest.TestCase):
    """The live pre-fix behaviour, measured rather than described."""

    def tearDown(self) -> None:
        home = getattr(self, "home", None)
        if home is not None:
            import shutil

            shutil.rmtree(home, ignore_errors=True)
        if getattr(self, "_saved_home", None) is not None:
            os.environ["HOME"] = self._saved_home

    def test_harness_env_file_parser_misses_the_export_shape(self):
        """The exact cause: `export KEY=...` is skipped, so a present key reads absent."""
        self.home = scratch("harnessgap")
        write(
            self.home / ".config" / "scmorc" / "jev.env",
            "export TYPESAFE_API_KEY=" + KEY_VALUE + "\n",
        )
        try:
            from harness_source import resolve_source  # type: ignore

            harness_root = resolve_source().root
            if str(harness_root) not in sys.path:
                sys.path.insert(0, str(harness_root))
            from harness.config import resolve_jev_key  # type: ignore
        except Exception:  # noqa: BLE001 - no admitted checkout in CI
            self.skipTest("Harness checkout not admitted; comparison skipped")
        self._saved_home = os.environ.get("HOME", "")
        os.environ["HOME"] = str(self.home)
        try:
            self.assertIsNone(
                resolve_jev_key(),
                "the Harness parser is expected to miss an `export`-prefixed line",
            )
        finally:
            os.environ["HOME"] = self._saved_home
        key, location, _rows = keys.resolve_key(home=self.home, env={})
        self.assertEqual(key, KEY_VALUE, "the consumer-side resolver must still find it")
        self.assertIn("scmorc", location)


class WiringTests(unittest.TestCase):
    """local_harness must prefer the Harness key and fall back, never report absent."""

    def tearDown(self) -> None:
        home = getattr(self, "home", None)
        if home is not None:
            import shutil

            shutil.rmtree(home, ignore_errors=True)

    def _stub_modules(self, harness_key):
        def resolve_jev_key():
            return harness_key

        return {
            "root": Path("."),
            "source": object(),
            "resolve_jev_key": resolve_jev_key,
            "resolve_api_key": lambda: None,
            "JevEvaluator": object,
            "JevPolicy": object,
            "JevEvaluationResult": object,
            "jev_cost": lambda *a, **k: 0.0,
            "validate_operator_pack": lambda *a, **k: None,
            "issue_sort_question_pack": lambda *a, **k: None,
            "match_keywords": lambda *a, **k: None,
            "_validate_questions": lambda *a, **k: None,
            "_parse_answer": lambda *a, **k: None,
        }

    def test_harness_key_wins_when_the_harness_resolves_one(self):
        import local_harness

        original = local_harness.import_harness_modules
        local_harness.import_harness_modules = lambda: self._stub_modules(KEY_VALUE)
        try:
            mod = local_harness.import_harness()
        finally:
            local_harness.import_harness_modules = original
        self.assertEqual(mod["key"], KEY_VALUE)
        self.assertTrue(str(mod["key_location"]).startswith("harness:"))

    def test_falls_back_to_a_documented_location_when_the_harness_finds_none(self):
        import local_harness

        self.home = scratch("wiring")
        write(self.home / ".typesafe_key", KEY_VALUE + "\n")
        original = local_harness.import_harness_modules
        original_home = os.environ.get("HOME", "")
        local_harness.import_harness_modules = lambda: self._stub_modules(None)
        os.environ["HOME"] = str(self.home)
        try:
            mod = local_harness.import_harness()
        finally:
            local_harness.import_harness_modules = original
            os.environ["HOME"] = original_home
        self.assertEqual(mod["key"], KEY_VALUE,
                         "a key present at a documented location must not read absent")
        self.assertIn(".typesafe_key", mod["key_location"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
