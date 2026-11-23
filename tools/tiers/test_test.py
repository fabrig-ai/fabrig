#!/usr/bin/env python3
"""Exercise tools/tiers/test, the implementer's counted nextest command, in a scratch repository."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from test_counts import CountsRepository


FAKE_CARGO = '''#!/usr/bin/env python3
import json
import os
from pathlib import Path
import sys

path = os.environ.get('TIERS_COUNTS_OUT')
runner = os.environ.get(os.environ['FIXTURE_RUNNER_KEY'])
Path('invocation.json').write_text(json.dumps(dict(args=sys.argv[1:], path=path, runner=runner)))
if path and runner:
    assert Path(runner).is_file()
    assert Path(path).is_file()
    with open(path, 'a') as output:
        for binary, name, instructions in json.loads(os.environ['FIXTURE_COUNTS']):
            print(json.dumps(dict(binary_id=binary, test=name, instructions=instructions)),
                  file=output)
print('nextest stdout')
print('nextest stderr', file=sys.stderr)
sys.exit(int(os.environ['FIXTURE_EXIT']))
'''

FAKE_CC = '''#!/bin/sh
printf 'built\\n' >> compiler.log
printf '#!/bin/sh\\nexit 91\\n' > "$4"
'''

class CountedCommandTests(CountsRepository):
    def setUp(self):
        if sys.platform != 'linux' or os.uname().machine not in ('x86_64', 'aarch64'):
            self.skipTest('requires a supported Linux host triple')
        super().setUp()
        glibc = 'CS_GNU_LIBC_VERSION' in os.confstr_names and os.confstr('CS_GNU_LIBC_VERSION')
        libc = 'gnu' if glibc else 'musl'
        self.triple = f'{os.uname().machine}-unknown-linux-{libc}'
        self.key = f"CARGO_TARGET_{self.triple.upper().replace('-', '_')}_RUNNER"
        tools = self.root / 'tools' / 'tiers'
        tools.mkdir(parents=True)
        self.command = tools / 'test'
        for name in ('test', 'budget-lines.awk', 'count-runner.c'):
            shutil.copy2(HERE / name, tools / name)
        (tools / 'layers.toml').write_text('[jig]\n"new-world::only" = "domain"\n')
        bin_dir = self.root / 'bin'
        bin_dir.mkdir()
        for name, source in (('cargo', FAKE_CARGO), ('cc', FAKE_CC)):
            program = bin_dir / name
            program.write_text(source)
            program.chmod(0o755)
        self.paranoid = self.root / 'perf_event_paranoid'
        self.paranoid.write_text('2\n')
        self.command.write_text(self.command.read_text().replace(
            '/proc/sys/kernel/perf_event_paranoid', str(self.paranoid)))
        self.env = {key: value for key, value in self.env.items()
                    if not key.startswith(('CARGO_', 'NEXTEST_', 'TIERS_'))}
        self.env.update(CARGO_HOME=str(self.root / 'cargo-home'),
                        PATH=str(bin_dir) + os.pathsep + self.env['PATH'],
                        FIXTURE_RUNNER_KEY=self.key, FIXTURE_EXIT='7',
                        FIXTURE_COUNTS=json.dumps([
                            ('jig-suite::domain', 'one', 2_000_000_000),
                            ('new-world::only', 'two', 2_900_000_000),
                            ('jig-lib', 'three', 400_000_000),
                        ]))
        profile = self.data['profiles']['default']
        profile['binaries']['jig-suite::domain']['counts'] = {
            self.first: 239_099_999_995, self.third: 400_000_000_000}

    def run_command(self, *args):
        result = subprocess.run((str(self.command), *args),
                                cwd=self.root, env=self.env, capture_output=True, text=True)
        self.assertEqual(result.returncode, 7, result.stderr)
        self.assertEqual(result.stdout, 'nextest stdout\n')
        self.assertTrue(result.stderr.startswith('nextest stderr\n'), result.stderr)
        invocation = json.loads((self.root / 'invocation.json').read_text())
        self.assertEqual(invocation['args'], ['nextest', 'run', *args])
        if invocation['path'] is not None:
            self.assertFalse(Path(invocation['path']).exists())
        return result.stderr.splitlines()[1:], invocation

    def publish_table(self, lines):
        table = '\n'.join(['version\t1', f'main\t{self.second}', *lines]) + '\n'
        blob = self.git('hash-object', '-w', '--stdin', content=table)
        tree = self.git('mktree', content=f'100644 blob {blob}\tallowances.tsv\n')
        commit = self.git('commit-tree', tree, '-m', 'publish allowances')
        self.git('update-ref', 'refs/temper/counts', commit)

    def test_lines_from_main_and_reuse_runner(self):
        self.publish_table([
            'binary\tdefault\tjig-suite::domain\tjig\tdomain',
            'binary\tdefault\tjig-lib\tjig\tstep',
            'layer\tdefault\tjig\tdomain\t241400000000\t4828000000',
            'layer\tdefault\tjig\tstep\t10000000000\t1000000000',
        ])
        args = ('-p', 'jig-suite', '--test', 'domain', '-E', 'test(=one)')
        lines, first = self.run_command(*args)
        self.assertEqual(lines, [
            f"budgets: jig default domain: this run's tests count 2.0 G; "
            f"a change may grow the layer by 4.8 G at main {self.second[:10]} (2% of 241.4 G, at least 1 G)",
            f"budgets: jig default step: this run's tests count 0.4 G; "
            f"a change may grow the layer by 1.0 G at main {self.second[:10]} (2% of 10.0 G, at least 1 G)",
            "budgets: new-world::only is not in main's counts; "
            "the gate places a new binary by tools/tiers/layers.toml",
        ])
        self.assertEqual(first['runner'], str(self.root / 'target' / 'tiers-count-runner'))
        self.assertEqual(Path(first['path']).parent, self.root / 'target')
        self.assertEqual((self.root / 'compiler.log').read_text(), 'built\n')
        lines_again, second = self.run_command(*args)
        self.assertEqual(lines_again, lines)
        self.assertNotEqual(first['path'], second['path'])
        self.assertEqual((self.root / 'compiler.log').read_text(), 'built\n')
        os.utime(Path(second['runner']), ns=(0, 0))
        self.run_command(*args)
        self.assertEqual((self.root / 'compiler.log').read_text(), 'built\nbuilt\n')

    def test_absent_table_is_silent(self):
        lines, first = self.run_command('--target-dir=scratch-target')
        self.assertEqual(lines, [])
        # The runner and the counts file live under the repository's target directory, whatever cargo's is.
        self.assertEqual(Path(first['runner']).parent, self.root / 'target')
        self.publish()
        lines, _ = self.run_command('--lib')
        self.assertEqual(lines, [])

    def test_profile_from_the_arguments(self):
        self.publish_table([
            'binary\tfuzzy\tjig-suite::domain\tjig\tdomain',
            'layer\tfuzzy\tjig\tdomain\t250000000000\t5000000000',
        ])
        for args in (('-P', 'fuzzy'), ('--profile=fuzzy',), ('-Pfuzzy',), ('-P=fuzzy',), ('--profile', 'fuzzy')):
            with self.subTest(args=args):
                lines, _ = self.run_command(*args)
                self.assertTrue(lines[0].startswith('budgets: jig fuzzy domain:'), lines)
                self.assertIn('by 5.0 G', lines[0])

    def test_paranoid_or_an_existing_runner_use_plain_nextest(self):
        self.paranoid.write_text('3\n')
        self.assertEqual(self.run_command('--lib'), ([], {
            'args': ['nextest', 'run', '--lib'], 'runner': None, 'path': None}))
        self.paranoid.write_text('2\n')
        self.env[self.key] = '/fixture-runner'
        self.assertEqual(self.run_command('--lib'), ([], {
            'args': ['nextest', 'run', '--lib'], 'runner': '/fixture-runner', 'path': None}))
        del self.env[self.key]
        config = self.root / '.cargo'
        config.mkdir()
        (config / 'config.toml').write_text(f'[target.{self.triple}]\nrunner = "fixture-runner"\n')
        self.assertEqual(self.run_command('--lib'), ([], {
            'args': ['nextest', 'run', '--lib'], 'runner': None, 'path': None}))
        self.assertFalse((self.root / 'compiler.log').exists())

    def test_missing_compiler_uses_plain_nextest(self):
        bin_dir = self.root / 'bin'
        (bin_dir / 'cc').unlink()
        (bin_dir / 'python3').symlink_to(sys.executable)
        self.env['PATH'] = str(bin_dir)
        self.assertEqual(self.run_command('--lib'), ([], {
            'args': ['nextest', 'run', '--lib'], 'runner': None, 'path': None}))


if __name__ == '__main__':
    unittest.main()
