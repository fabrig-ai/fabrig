"""Select and run the small Gate checks in the checked-out landing candidate."""
from pathlib import Path
import os
import runpy
import subprocess
import sys
import xml.etree.ElementTree as ET

HOOKS = {'.temper/pre-pr', '.temper/goal-child-pre-pr', '.temper/goal-pre-pr'}
PYTHON = ['python3', '-I', '-B']


def route(paths):
    if not paths:
        return 'full'
    if paths & {'.forgejo/scripts/gate_scope.py', '.forgejo/workflows/gate.yml'}:
        return 'mixed'
    checker = {path for path in paths if path in HOOKS or
               path.startswith('tools/pre-pr/') and path.endswith(('.py', '.md'))}
    if paths == checker:
        return 'checker'
    return 'mixed' if checker else 'classify'


def checks(work, base, head, reports, execute=subprocess.run):
    """Return 0 for a narrow pass, 2 for refusal, 1 to continue with the full Gate."""
    changed = subprocess.check_output(['git', 'diff', '--name-only', '--no-renames', '-z', base, head, '--'], cwd=work)
    paths = {os.fsdecode(path) for path in changed.split(b'\0') if path}
    scope = route(paths)
    commands = []
    if scope in ('checker', 'mixed'):
        commands.append(('checker', [*PYTHON, 'tools/pre-pr/test_pre_pr.py']))
    if scope == 'classify':
        policy = runpy.run_path(str(work / 'tools/pre-pr/pre_pr.py'))
        if policy['code_inputs'](paths, policy['metadata'](work, os.environ), work):
            return 1
    if scope == 'classify' or scope == 'checker' and any(path.endswith('.md') for path in paths):
        commands.append(('citations', [*PYTHON, 'tools/citations/citations.py', '--all']))
    report = ET.Element('testsuite', name='preflight')
    failed = []
    for name, command in commands:
        print(f'gate: selection: {name}', flush=True)
        case = ET.SubElement(report, 'testcase', classname='gate', name=name)
        try:
            status = execute(command, cwd=work, timeout=600).returncode
            reason = f'exit {status}' if status else None
        except (OSError, subprocess.TimeoutExpired) as error:
            reason = str(error)
        if reason:
            failed.append('gate::' + name)
            ET.SubElement(case, 'failure', message=reason)
    if not failed and scope in ('full', 'mixed'):
        return 1
    report.set('tests', str(len(commands)))
    report.set('failures', str(len(failed)))
    ET.ElementTree(report).write(reports / 'junit.xml', encoding='utf-8', xml_declaration=True)
    (reports / 'failed-tests.txt').write_text(''.join(name + '\n' for name in failed))
    return 2 if failed else 0


if __name__ == '__main__':
    root, base, head = sys.argv[1:]
    sys.exit(checks(Path(root), base, head, Path.cwd()))
