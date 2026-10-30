"""Executable offline suite imports must leave a clean caller checkout."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


KIT = Path(__file__).resolve().parent


class BytecodeTests(unittest.TestCase):
    def test_executable_suite_imports_preserve_clean_checkout(self):
        with tempfile.TemporaryDirectory(prefix="heavy-bytecode-") as directory:
            repo = Path(directory)
            kit = repo / "tools/heavy"
            shutil.copytree(KIT, kit, ignore=shutil.ignore_patterns("__pycache__"))
            # No inherited global excludes or bytecode policy may mask a write.
            env = {"PATH": os.defpath, "HOME": str(repo), "GIT_CONFIG_GLOBAL": "/dev/null",
                   "GIT_CONFIG_NOSYSTEM": "1", "GIT_AUTHOR_NAME": "Fixture",
                   "GIT_AUTHOR_EMAIL": "fixture@example.invalid", "GIT_COMMITTER_NAME": "Fixture",
                   "GIT_COMMITTER_EMAIL": "fixture@example.invalid"}

            def run(argv):
                return subprocess.run(argv, cwd=repo, env=env, stdin=subprocess.DEVNULL,
                                      text=True, capture_output=True, check=True)

            run(["git", "init", "-q"])
            run(["git", "add", "."])
            run(["git", "commit", "-qm", "fixture"])
            for name, arguments in (("test-gate-policy", ["--help"]),
                                    ("test-heavy-plain", ["--help"]),
                                    ("test-heavy-byte-repairs", ["--help"]),
                                    ("test-heavy-slot-wait-cap", ["--help"]),
                                    ("test-receipt-inputs", [])):
                with self.subTest(suite=name):
                    run([str(kit / name), *arguments])
                    self.assertEqual(run(["git", "status", "--porcelain=v1",
                                          "--untracked-files=all"]).stdout, "")


if __name__ == "__main__":
    unittest.main()
