"""Validate proposed Gate changes with contracts read from the landing base."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def workflow(text):
    import yaml

    class UniqueKeys(yaml.BaseLoader):
        def construct_mapping(self, node, deep=False):
            mapping = super().construct_mapping(node, deep)
            if len(mapping) != len(node.value):
                raise ValueError('duplicate workflow key')
            return mapping

    document = yaml.load(text, Loader=UniqueKeys)
    if not isinstance(document.get('jobs'), dict) or not document['jobs']:
        raise ValueError('missing workflow jobs')
    # Names used for display are the only YAML fields outside the execution contract.
    document.pop('name', None)
    for job in document['jobs'].values():
        for step in job.get('steps', []):
            step.pop('name', None)
            if 'run' in step:
                subprocess.run(['bash', '-n'], input=step['run'], text=True, check=True)
    return document


def workflow_changed(work, base):
    path = '.forgejo/workflows/gate.yml'
    landed = subprocess.check_output(['git', 'show', f'{base}:{path}'], cwd=work)
    return workflow(landed) != workflow((work / path).read_bytes())


def replay(work, kind):
    scripts = work / '.forgejo/scripts'
    workflow((work / '.forgejo/workflows/gate.yml').read_bytes())
    if kind == 'routing':
        subprocess.run(['sh', '-n', str(scripts / 'gate_route.sh')], check=True)
    suite = 'test_gate_log.py' if kind == 'reporting' else 'test_gate_routes.py'
    with tempfile.TemporaryDirectory(prefix='gate-contract-') as temporary:
        scratch = Path(temporary)
        env = {'PATH': os.defpath, 'HOME': str(scratch), 'LC_ALL': 'C', 'GIT_CONFIG_NOSYSTEM': '1'}
        # Run the landed contract against candidate code, then exercise new candidate cases.
        contract = Path(__file__).with_name(suite)
        shutil.copyfile(contract, scratch / suite)
        if kind == 'reporting':
            shutil.copyfile(scripts / 'gate_log.py', scratch / 'gate_log.py')
        env['GATE_TEST_SCRIPTS'] = str(scripts)
        for test in (scratch / suite, scripts / suite):
            if test == scripts / suite and test.read_bytes() == contract.read_bytes():
                continue
            subprocess.run([sys.executable, '-I', '-B', str(test)], cwd=scratch, env=env,
                           timeout=60, check=True)


if __name__ == '__main__':
    replay(Path(sys.argv[1]).resolve(), sys.argv[2])
