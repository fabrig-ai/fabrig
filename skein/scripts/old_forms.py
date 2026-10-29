"""Old forms may only fall (skein's docs/design/draft/dynamic-memory.md, 7).

Count matches in tracked Rust files, excluding generated/ components and
comments, while keeping string literals. Compare each form independently
with scripts/old-forms.txt; --write can initialize or lower that baseline.
Escapes separate words in ordinary strings and chars; raw strings stay literal.
The max_out form exempts only the exact identifiers in EXEMPT: the four
kernel submission bounds and the provider wire field max_output_tokens.
Identifiers containing an exempt name still count.
Recorded raises and other comments stay in place when counts are rewritten.
Run from the repository root, using only the Python standard library:

    python3 -I -B scripts/old_forms.py [--write]
"""
import argparse
from pathlib import Path
import re
import subprocess
import sys

FORMS = {
    'max_out': re.compile(r'\b\w*(?:max_out|MAX_OUT)\w*\b'),
    'JournalRoom': re.compile(r'\bJournalRoom\b'),
    'takes': re.compile(r'\.takes\(&'),
    'worst_case': re.compile(r'\b\w*worst_case\w*\b'),
}
EXEMPT = {
    'max_out': (
        # The kernel's submission bounds stay at its edges.
        'MAX_OUT_DOWN', 'MAX_OUT_UP', 'MAX_OUT_RESUME', 'MAX_OUT_FIRE',
        # The provider's wire vocabulary stays on the wire.
        'max_output_tokens',
    ),
}
BASELINE = Path('scripts/old-forms.txt')
HEADER = (
    "# Uses of the old forms (skein's docs/design/draft/dynamic-memory.md, 7): each count may only fall.\n"
    '# Lower them with `python3 -I -B scripts/old_forms.py --write`; '
    'only the director raises one, recorded in a `# Raised:` line.\n'
)
RULE = (
    "Uses of the old forms may only fall (skein's docs/design/draft/dynamic-memory.md, 7): "
    'new code takes the new forms (Queue::uncapped, Journal::uncapped, decide, backlog), '
    'or removes as many old uses as it adds.'
)
# Match whole quoted literals before looking for comment markers inside them.
# A closing quote distinguishes a character from a lifetime such as 'a.
TOKEN = re.compile(
    r'//[^\n]*|/\*|(?:br|cr|r)(?P<hashes>#{0,255})"'
    r'|"(?:\\[\s\S]|[^"\\])*"'
    r"|'(?:[^'\\\n]|\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|[^\n]))'"
)
BLOCK = re.compile(r'/\*|\*/')
ESCAPE = re.compile(r'\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|[\s\S])')
IDENTIFIER = re.compile(r'\b\w+\b')


def without_comments(source):
    """A small lexical scan: nested comments, raw/escaped strings and chars."""
    pieces = []
    position = 0
    while token := TOKEN.search(source, position):
        pieces.append(source[position:token.start()])
        end = token.end()
        if token.group() == '/*':
            depth = 1
            while depth and (marker := BLOCK.search(source, end)):
                depth += 1 if marker.group() == '/*' else -1
                end = marker.end()
            if depth:
                end = len(source)
            pieces.append(' ')
        elif token.group().startswith('//'):
            pieces.append(' ')
        else:
            if token.group('hashes') is not None:
                closing = '"' + token.group('hashes')
                close = source.find(closing, end)
                end = len(source) if close == -1 else close + len(closing)
                pieces.append(source[token.start():end])
            else:
                pieces.append(ESCAPE.sub(' ', token.group()))
        position = end
    pieces.append(source[position:])
    return ''.join(pieces)


def counts():
    paths = subprocess.run(
        ['git', 'ls-files', '-z', '*.rs'], check=True, capture_output=True,
    ).stdout.split(b'\0')
    totals = dict.fromkeys(FORMS, 0)
    for path in paths:
        if not path or b'generated' in path.split(b'/'):
            continue
        source = Path(path.decode('utf-8')).read_text(encoding='utf-8')
        if not any(marker in source for marker in (*FORMS, 'MAX_OUT')):
            continue
        source = without_comments(source)
        # Prefilter whole identifiers once: the three identifier patterns
        # otherwise backtrack through every unrelated word in the tree.
        candidates = ' '.join(
            word for word in IDENTIFIER.findall(source)
            if 'max_out' in word or 'MAX_OUT' in word
            or 'JournalRoom' in word or 'worst_case' in word
        )
        for form, pattern in FORMS.items():
            totals[form] += sum(
                1 for match in pattern.finditer(source if form == 'takes' else candidates)
                if match.group() not in EXEMPT.get(form, ())
            )
    return totals


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true', help='initialize or lower the baseline')
    args = parser.parse_args()
    current = counts()
    try:
        lines = BASELINE.read_text(encoding='utf-8').splitlines(keepends=True)
    except FileNotFoundError:
        if not args.write:
            parser.error(f'{BASELINE} is missing; initialize with --write')
        baseline = current
        lines = HEADER.splitlines(keepends=True) + [
            f'{form} {count}\n' for form, count in current.items()
        ]
    else:
        try:
            entries = [line.split() for line in lines if line.strip() and not line.startswith('#')]
            baseline = {form: int(count) for form, count in entries}
            if (list(baseline) != list(FORMS) or len(entries) != len(FORMS)
                    or any(count < 0 for count in baseline.values())):
                raise ValueError('expected each form once, in order, with nonnegative counts')
        except ValueError as error:
            parser.error(f'invalid {BASELINE}: {error}')
    raised = False
    for form, count in current.items():
        if count > baseline[form]:
            print(f'{form}: count {count}, baseline {baseline[form]}', file=sys.stderr)
            raised = True
    if raised:
        print(RULE, file=sys.stderr)
        return 1
    if args.write:
        for index, line in enumerate(lines):
            if line.strip() and not line.startswith('#'):
                form = line.split()[0]
                lines[index] = f'{form} {current[form]}\n'
        BASELINE.write_text(''.join(lines), encoding='utf-8')
    else:
        for form, count in current.items():
            if count < baseline[form]:
                print(f'{form}: count {count}, baseline {baseline[form]}; lower with '
                      '`python3 -I -B scripts/old_forms.py --write`', file=sys.stderr)
    return 0


if __name__ == '__main__':
    sys.exit(main())
