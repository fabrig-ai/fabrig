"""Shared-target freshness after the base judge (tools/pre-pr/design.md, 5, 9)."""
import contextlib
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest
from unittest import mock


SPEC = importlib.util.spec_from_file_location('pre_pr', Path(__file__).with_name('pre_pr.py'))
PRE_PR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PRE_PR)


class SharedTarget(unittest.TestCase):
    def test_base_and_head_builds_use_their_own_sources(self):
        self.check_builds(generated=False)

    def test_build_script_regenerates_each_sides_code(self):
        for missing in (False, True):
            with self.subTest(missing_input=missing):
                self.check_builds(generated=True, missing=missing)

    def check_builds(self, generated, missing=False):
        with tempfile.TemporaryDirectory(prefix='pre-pr-target-') as directory:
            root = Path(directory)
            env = os.environ.copy()
            for name in ('GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'CARGO_TARGET_DIR'):
                env.pop(name, None)
            env.update(TEMPER_BUNDLE='none', CARGO_TARGET_DIR=str(root / 'target'))

            def command(*args, cwd=root, command_env=env):
                return subprocess.run(args, cwd=cwd, env=command_env, check=True,
                                      capture_output=True, text=True).stdout

            def write(name, content):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content)

            command('git', 'init', '-q')
            command('git', 'config', 'user.name', 'Pre-pr test')
            command('git', 'config', 'user.email', 'pre-pr@example.invalid')
            command('git', 'config', 'commit.gpgsign', 'false')
            write('.gitignore', '/target\n')
            write('Cargo.toml', '[workspace]\nmembers = ["api", "consumer"]\nresolver = "3"\n')
            write('api/Cargo.toml', '[package]\nname = "api"\nversion = "0.1.0"\nedition = "2024"\n')
            write('consumer/Cargo.toml', '[package]\nname = "consumer"\nversion = "0.1.0"\n'
                  'edition = "2024"\n[dependencies]\napi = { path = "../api" }\n')
            source = 'api/value.txt' if generated else 'api/src/lib.rs'
            if generated:
                write('api/src/lib.rs', 'include!(concat!(env!("OUT_DIR"), "/value.rs"));\n')
                write('api/build.rs', '''fn main() {
    println!("cargo:rerun-if-changed=value.txt");
    let value = std::fs::read_to_string("value.txt").unwrap_or_else(|_| "head".to_owned());
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("value.rs");
    std::fs::write(output, format!("pub fn value() -> &'static str {{ {:?} }}", value.trim())).unwrap();
}
''')
            write(source, 'base\n' if generated else 'pub fn value() -> &\'static str { "base" }\n')
            write('consumer/src/main.rs', 'fn main() { println!("{}", api::value()); }\n')
            command('cargo', 'generate-lockfile', '--offline')
            command('git', 'add', '.')
            command('git', 'commit', '-qm', 'base')
            command('git', 'update-ref', 'refs/temper/landing', 'HEAD')
            # A kept base checkout can predate the head's shared artifacts too.
            worktree = root / '.git/temper-pre-pr/base'
            command('git', 'worktree', 'add', '--detach', str(worktree), 'HEAD')
            if missing:
                (root / source).unlink()
            else:
                write(source, 'head\n' if generated else 'pub fn value() -> &\'static str { "head" }\n')
            # Older head sources reproduce Cargo accepting the base's newer rlib.
            older = time.time_ns() - 10_000_000_000
            if not missing:
                os.utime(root / source, ns=(older, older))
            self.assertEqual(command('cargo', 'run', '--offline', '-q', '-p', 'consumer'), 'head\n')
            index = (root / '.git/index').read_bytes()
            unchanged = (root / 'consumer/src/main.rs').stat().st_mtime_ns
            real_run = PRE_PR.run
            base_builds = []

            def run(arguments, cwd, run_env, *args, **kwargs):
                if arguments[0] == 'python3':
                    return 0, ''
                return real_run(arguments, cwd, run_env, *args, **kwargs)

            def clippy(packages, data, cwd, build_env):
                if cwd == root:
                    return 101, [PRE_PR.failure('clippy api lint api/src/lib.rs', 'head lint',
                                                ('api', 'lint', 'api/src/lib.rs', 'head lint'), 'api')]
                self.assertEqual(build_env['CARGO_TARGET_DIR'], str(root / 'target'))
                command('cargo', 'build', '--offline', '-q', '-p', 'consumer',
                        cwd=cwd, command_env=build_env)
                self.assertEqual(command(str(root / 'target/debug/consumer')), 'base\n')
                base_builds.append(cwd)
                return 0, []

            with contextlib.chdir(root), mock.patch.dict(os.environ, env, clear=True), \
                    mock.patch.object(PRE_PR, 'formatting', return_value=[]), \
                    mock.patch.object(PRE_PR, 'run', side_effect=run), \
                    mock.patch.object(PRE_PR, 'clippy', side_effect=clippy), \
                    mock.patch.object(PRE_PR, 'nextest', return_value=(0, [])):
                self.assertEqual(PRE_PR.main(), 1)

            self.assertEqual(len(base_builds), 1)
            self.assertEqual(command('cargo', 'run', '--offline', '-q', '-p', 'consumer'), 'head\n')
            self.assertEqual((root / '.git/index').read_bytes(), index)
            self.assertEqual((root / 'consumer/src/main.rs').stat().st_mtime_ns, unchanged)


if __name__ == '__main__':
    unittest.main()
