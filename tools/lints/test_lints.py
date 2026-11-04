"""Regression checks for tools/lints/design.md, 5, on repositories built per test."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


class Lints(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="lints-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        shutil.copytree(HERE, self.root / "tools/lints")
        self.write("Cargo.toml", '[workspace]\nmembers = ["skein/a", "jig/b", "outside/c", "skein/a"]\n')
        for member in ("skein/a", "jig/b", "outside/c"):
            self.write(f"{member}/Cargo.toml", '[package]\nname = "fixture"\n\n' + self.profile("step"))
        self.git("init", "-q")
        self.git("add", ".")

    def write(self, path, text):
        path = self.root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def git(self, *args):
        return subprocess.run(("git", *args), cwd=self.root, capture_output=True, text=True, check=True)

    def profile(self, name):
        return (self.root / f"tools/lints/profiles/{name}.toml").read_text()

    def run_check(self, *args, status=0):
        result = subprocess.run((sys.executable, "-I", "-B", "tools/lints/lints.py", *args),
                                cwd=self.root, capture_output=True, text=True)
        self.assertEqual(result.returncode, status, result.stdout + result.stderr)
        return result.stdout

    def test_a_block_that_is_a_definition_passes(self):
        for name in ("step", "harness", "binary", "ring"):
            with self.subTest(name=name):
                self.write("skein/a/Cargo.toml", self.profile(name) + '\n# trailing comment\n\n')
                for variant in ("step", "ordinary", "ring"):
                    self.write("skein/clippy.toml", (self.root / f"tools/lints/clippy/{variant}.toml").read_text())
                    self.git("add", "skein/clippy.toml")
                    self.assertEqual(self.run_check(), "0 findings\n")

    def test_a_changed_level_is_a_finding(self):
        self.write("skein/a/Cargo.toml", self.profile("harness").replace('"forbid"', '"allow"'))
        output = self.run_check(status=1)
        self.assertIn("skein/a/Cargo.toml: matches no profile; harness line 2", output)
        self.assertIn('expected \'unsafe_code = "forbid"\'', output)

    def test_an_added_lint_is_a_finding(self):
        self.write("skein/a/Cargo.toml", self.profile("binary") + 'extra_lint = "allow"\n')
        self.assertIn("extra_lint", self.run_check(status=1))

    def test_a_package_with_no_block_is_a_finding(self):
        self.write("skein/a/Cargo.toml", "[package]\n")
        self.assertIn("skein/a/Cargo.toml: no block", self.run_check(status=1))

    def test_a_second_block_is_a_finding(self):
        for separator in ("", "\n[dependencies]\n"):
            with self.subTest(separator=separator):
                self.write("skein/a/Cargo.toml", self.profile("step") + separator + self.profile("step"))
                self.assertIn("skein/a/Cargo.toml: a second block", self.run_check(status=1))

    def test_a_package_outside_the_trees_is_not_read(self):
        self.write("outside/c/Cargo.toml", "not TOML")
        self.write("outside/clippy.toml", "not a variant")
        self.git("add", ".")
        self.assertEqual(self.run_check(), "0 findings\n")
        self.assertEqual(self.run_check("outside"), "0 findings\n")

    def test_paths_narrow_the_check(self):
        self.write("jig/b/Cargo.toml", "")
        self.write("jig/clippy.toml", "")
        self.git("add", ".")
        self.assertEqual(self.run_check("skein"), "0 findings\n")
        self.assertIn("jig/b/Cargo.toml: no block", self.run_check("jig/b/Cargo.toml", status=1))
        self.assertEqual(self.run_check("jig/bb"), "0 findings\n")

    def test_a_member_named_twice_is_one_finding(self):
        self.write("skein/a/Cargo.toml", "")
        self.assertEqual(self.run_check(status=1), "skein/a/Cargo.toml: no block\n1 findings\n")

    def test_a_clippy_toml_that_is_no_variant_is_a_finding(self):
        text = (self.root / "tools/lints/clippy/ordinary.toml").read_text()
        self.write("skein/clippy.toml", text.replace("allow-unwrap-in-tests = true", "allow-unwrap-in-tests = false"))
        self.git("add", ".")
        output = self.run_check(status=1)
        self.assertIn("skein/clippy.toml: matches no variant; ordinary line 8", output)
        self.assertIn("allow-unwrap-in-tests = false", output)

    def test_set_gives_a_profile_and_keeps_the_rest(self):
        before, after = '[package]\nname = "fixture"\n\n', '\n# dependencies\n[dependencies]\n'
        self.write("skein/a/Cargo.toml", before + "# old block\n# old reason\n" + self.profile("step") + after)
        self.run_check("--set", "harness", "skein/a")
        self.assertEqual((self.root / "skein/a/Cargo.toml").read_text(), before + self.profile("harness") + after)
        self.write("skein/a/Cargo.toml", '[package]\nname = "fixture"')
        self.run_check("--set", "ring", "skein/a/Cargo.toml")
        self.assertEqual((self.root / "skein/a/Cargo.toml").read_text(),
                         '[package]\nname = "fixture"\n' + self.profile("ring"))

    def test_write_follows_a_changed_definition(self):
        profile = self.profile("harness")
        variant = (self.root / "tools/lints/clippy/ordinary.toml").read_text()
        self.write("skein/a/Cargo.toml", "# package policy\n" + profile)
        self.write("skein/clippy.toml", variant)
        self.git("add", ".")
        self.write("tools/lints/profiles/harness.toml", profile.replace('"forbid"', '"deny"'))
        self.write("tools/lints/clippy/ordinary.toml", variant + "\n# definition change\n")
        self.run_check("--write", "skein")
        self.assertEqual((self.root / "skein/a/Cargo.toml").read_text(), "# package policy\n" + self.profile("harness"))
        self.assertEqual((self.root / "skein/clippy.toml").read_text(), variant + "\n# definition change\n")

    def test_no_definition_allows_a_disallowed_lint(self):
        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["lints"]
        for name in ("step", "harness", "binary", "ring", "workspace"):
            lints = workspace if name == "workspace" else tomllib.loads(self.profile(name))["lints"]
            lints = workspace if lints.get("workspace") else lints
            for lint in ("disallowed_types", "disallowed_macros", "disallowed_methods"):
                with self.subTest(profile=name, lint=lint):
                    level = lints["clippy"][lint]
                    self.assertNotEqual(level.get("level") if isinstance(level, dict) else level, "allow")

    def test_harness_alone_allows_wildcards(self):
        for name, expected in (("harness", "allow"), ("binary", "deny"), ("ring", "deny")):
            with self.subTest(name=name):
                lints = tomllib.loads(self.profile(name))["lints"]["clippy"]
                self.assertEqual(lints["wildcard_enum_match_arm"], expected)


if __name__ == "__main__":
    unittest.main()
