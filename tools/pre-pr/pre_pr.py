"""Batch delivery formatting (temper/docs/design/protocol/configuration.md, 13.9).

Cargo's target roots retain module traversal, target editions and the nearest
rustfmt configuration. Batching avoids a formatter startup for every package.
"""
import json
from pathlib import Path
import subprocess


def _configuration(source, root):
    """Find the target's rustfmt configuration without leaving the repository."""
    source.relative_to(root)
    directory = source.parent
    while True:
        for name in ('.rustfmt.toml', 'rustfmt.toml'):
            candidate = directory / name
            if candidate.is_file():
                return candidate
        if directory == root:
            return None
        directory = directory.parent


def _groups(metadata, root):
    """Deduplicate target roots sharing an edition and rustfmt configuration."""
    groups = {}
    for package in metadata['packages']:
        for target in package['targets']:
            source = Path(target['src_path'])
            edition = target.get('edition', package['edition'])
            key = (edition, _configuration(source, root))
            groups.setdefault(key, set()).add(str(source))
    if not groups:
        raise ValueError('cargo metadata has no formatting targets')
    return groups


def _check(arguments, sources):
    """Bound each argv while retaining every target and propagating failures."""
    batch = []
    size = sum(len(argument.encode()) + 1 for argument in arguments)
    for source in sorted(sources):
        length = len(source.encode()) + 1
        if batch and size + length > 65536:
            subprocess.run(arguments + batch, check=True, stdin=subprocess.DEVNULL)
            batch = []
            size = sum(len(argument.encode()) + 1 for argument in arguments)
        batch.append(source)
        size += length
    if batch:
        subprocess.run(arguments + batch, check=True, stdin=subprocess.DEVNULL)


def main():
    """Check all workspace targets in batches; run no build or network request."""
    root = Path.cwd()
    result = subprocess.run(
        ['cargo', 'metadata', '--offline', '--no-deps', '--format-version', '1'],
        check=True, stdout=subprocess.PIPE, stdin=subprocess.DEVNULL,
    )
    for (edition, configuration), sources in _groups(json.loads(result.stdout), root).items():
        arguments = ['rustfmt', '--check', '--edition', edition]
        if configuration is not None:
            arguments.extend(['--config-path', str(configuration)])
        _check(arguments, sources)


if __name__ == '__main__':
    main()
