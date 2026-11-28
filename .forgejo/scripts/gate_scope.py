"""Select and run the small Gate checks in the checked-out landing candidate."""
from pathlib import Path
import fnmatch
import os
import runpy
import subprocess
import sys
import xml.etree.ElementTree as ET

HOOKS = {'.temper/hooks/deliver', '.temper/hooks/finish', '.temper/hooks/close',
         '.temper/pre-pr', '.temper/close', '.temper/checked-close'}
PYTHON = ['python3', '-I', '-B']
SCRIPTS = Path(__file__).resolve().parent
WORKFLOW = '.forgejo/workflows/gate.yml'
REPORTING = {'.forgejo/scripts/gate_log.py', '.forgejo/scripts/test_gate_log.py'}
ROUTING = {'.forgejo/scripts/gate_route.sh', '.forgejo/scripts/gate_scope.py',
           '.forgejo/scripts/gate_validate.py', '.forgejo/scripts/test_gate_routes.py'}


# The repository's own tools in Python, shell and prose, which their sibling suites check (tools/*/test_*.py).
TOOLING_SUFFIXES = ('.py', '.sh', '.md', '')


def tooling(path):
    return path.startswith('tools/') and Path(path).suffix in TOOLING_SUFFIXES


def tool_suites(work, paths):
    """Every suite beside a changed tool, by repository-relative path."""
    return sorted({str(suite.relative_to(work)) for path in paths
                   for suite in (work / path).parent.glob('test_*.py')})


def route(paths):
    if not paths:
        return 'full'
    checker = {path for path in paths if path in HOOKS or
               path.startswith('tools/pre-pr/') and path.endswith(('.py', '.md'))}
    gate = paths & (REPORTING | ROUTING | {WORKFLOW})
    if gate:
        return 'gate' if paths <= gate | checker else 'gate-and-product'
    if any(path.startswith('.forgejo/') for path in paths):
        return 'full'
    if paths == checker:
        return 'checker'
    if all(tooling(path) for path in paths):
        return 'tooling'
    return 'mixed' if checker else 'classify'


def checks(work, base, head, reports, execute=subprocess.run):
    """Return 0 for a narrow pass, 2 for refusal, 1 to continue with the full Gate."""
    changed = subprocess.check_output(['git', 'diff', '--name-only', '--no-renames', '-z', base, head, '--'], cwd=work)
    paths = {os.fsdecode(path) for path in changed.split(b'\0') if path}
    scope = route(paths)
    commands = []
    regular = scope in ('full', 'mixed', 'gate-and-product')
    if scope in ('gate', 'gate-and-product'):
        workflow_exec = False
        if WORKFLOW in paths:
            # A valid change to triggers, jobs, environment or executable steps also exercises the regular Gate.
            validator = runpy.run_path(str(SCRIPTS / 'gate_validate.py'))
            workflow_exec = validator['workflow_changed'](work, base)
            regular |= workflow_exec
        if paths & (REPORTING | {WORKFLOW}):
            commands.append(('reporting', [*PYTHON, str(SCRIPTS / 'gate_validate.py'), str(work), 'reporting']))
        if paths & ROUTING or workflow_exec:
            commands.append(('routing', [*PYTHON, str(SCRIPTS / 'gate_validate.py'), str(work), 'routing']))
    if scope in ('checker', 'mixed') or (scope.startswith('gate') and
            any(p in HOOKS or p.startswith('tools/pre-pr/') for p in paths)):
        commands.append(('checker', [*PYTHON, 'tools/pre-pr/test_pre_pr.py']))
    if scope == 'tooling':
        # A tool that tests read, or one with no suite beside it, takes the regular route.
        policy = runpy.run_path(str(work / 'tools/pre-pr/pre_pr.py'))
        globs = policy['test_input_globs'](work)
        suites = tool_suites(work, paths)
        if not suites or any(fnmatch.fnmatchcase(path, glob) for path in paths for glob in globs):
            return 1
        commands.extend(('tooling', [*PYTHON, suite]) for suite in suites)
    if scope == 'classify':
        policy = runpy.run_path(str(work / 'tools/pre-pr/pre_pr.py'))
        if policy['code_inputs'](paths, policy['metadata'](work, os.environ), work):
            return 1
    if scope == 'classify' or scope in ('checker', 'gate', 'tooling') and any(path.endswith('.md') for path in paths):
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
    if not failed and regular:
        return 1
    report.set('tests', str(len(commands)))
    report.set('failures', str(len(failed)))
    ET.ElementTree(report).write(reports / 'junit.xml', encoding='utf-8', xml_declaration=True)
    (reports / 'failed-tests.txt').write_text(''.join(name + '\n' for name in failed))
    return 2 if failed else 0


if __name__ == '__main__':
    root, base, head = sys.argv[1:]
    try:
        result = checks(Path(root), base, head, Path.cwd())
    except Exception as error:
        print(f'Gate validation failed: {error}', file=sys.stderr)
        result = 2
    sys.exit(result)
