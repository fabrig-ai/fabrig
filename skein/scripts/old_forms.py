"""Old forms may only fall (skein's docs/design/draft/dynamic-memory.md, 7,
and docs/design/draft/component-kit.md, 4).

Count matches in tracked Rust files, excluding generated/ components and
comments, while keeping string literals. Compare each form independently
with scripts/old-forms.txt; --write can initialize or lower that baseline,
and adds a row at its count for each form the baseline does not record yet.
Escapes separate words in ordinary strings and chars; raw strings stay literal.
A file is searched for a form only when it holds one of the form's MARKERS,
which every match holds. The forms in WORDS match whole identifiers, among
the words holding a marker; the others match the source, across tokens.
A form in EXCLUDE skips the files under its path prefixes: the crate that
owns the shared item, which keeps it.
The max_out form exempts only the exact identifiers in EXEMPT: the four
kernel submission bounds and the provider wire field max_output_tokens.
The worst_case form exempts the four reserved old bound names in EXEMPT.
Identifiers containing an exempt name still count.
Recorded raises and other comments stay in place when counts are rewritten.
Run from skein/, where paths are relative, using only the Python standard library:

    python3 -I -B scripts/old_forms.py [--write]
"""
import argparse
from pathlib import Path
import re
import subprocess
import sys

# One pop of a container, a path's or a method's result's, perhaps expected.
POP = r'[\w.]+(?:\(\))?\.pop(?:_front|_back|_first|_last)?\(\)(?:\.expect\("[^"]*"\))?'
FORMS = {
    'max_out': re.compile(r'\b\w*(?:max_out|MAX_OUT)\w*\b'),
    'JournalRoom': re.compile(r'\bJournalRoom\b'),
    'takes': re.compile(r'\.takes\(&'),
    'worst_case': re.compile(r'\b\w*worst_case\w*\b'),
    # The component kit's counts (component-kit.md, 4).
    'io_resume': re.compile(r'\b(?:skein_)?io::resume\b'),
    'earliest_helper': re.compile(r'\bfn\s+(?:earliest|earlier)\b'),
    'lifecycle_enum': re.compile(r'\benum\s+Lifecycle\b'),
    'growth_floor': re.compile(r'\bconst\s+GROWTH_FLOOR\b'),
    # A `for _ in 0..` loop whose whole body is one pop, its value dropped:
    # a container's clear.
    'pop_loop': re.compile(
        r'\bfor _ in 0(?:_?[iu](?:8|16|32|64|128|size))?\.\.[^{};]*\{\s*(?:'
        r'let\s+_\w*(?:\s*:[^=;]*)?\s*=\s*' + POP + r'|drop\(\s*' + POP + r'\s*\)|' + POP
        + r')\s*;?\s*\}'
    ),
    'hex_decimal_fn': re.compile(r'\bfn\s+(?:hex|decimal)\b'),
    'host_forwarder': re.compile(r'\bimpl\s+(?:\w+::)*Host\s+for\s+Process\b'),
}
# Text without spaces that every match of a form holds, which stripping
# comments never makes: a file holding none of a form's markers is not
# searched for it.
MARKERS = {
    'max_out': ('max_out', 'MAX_OUT'),
    'JournalRoom': ('JournalRoom',),
    'takes': ('takes',),
    'worst_case': ('worst_case',),
    'io_resume': ('io::resume',),
    'earliest_helper': ('earliest', 'earlier'),
    'lifecycle_enum': ('Lifecycle',),
    'growth_floor': ('GROWTH_FLOOR',),
    'pop_loop': ('.pop',),
    'hex_decimal_fn': ('hex', 'decimal'),
    'host_forwarder': ('Process',),
}
# The forms that match one identifier, searched among the words holding one
# of their markers: over the whole source they would backtrack through every
# unrelated word in the tree.
WORDS = ('max_out', 'JournalRoom', 'worst_case')
# Path prefixes, relative to skein/, whose files a form does not count: the
# crate that owns the shared item.
EXCLUDE = {
    'io_resume': ('crates/skein-io/',),
    'earliest_helper': ('crates/skein-lib/',),
    'lifecycle_enum': ('crates/skein-lib/',),
    'hex_decimal_fn': ('crates/skein-lib/',),
}
EXEMPT = {
    'worst_case': (
        # The generator keeps these reserved as field names (codec.md, 4).
        'worst_case_bytes', 'worst_case_heap',
        'worst_case_bytes_within', 'worst_case_heap_within',
    ),
    'max_out': (
        # The kernel's submission bounds stay at its edges.
        'MAX_OUT_DOWN', 'MAX_OUT_UP', 'MAX_OUT_RESUME', 'MAX_OUT_FIRE',
        # The provider's wire vocabulary stays on the wire.
        'max_output_tokens',
    ),
}
BASELINE = Path('scripts/old-forms.txt')
HEADER = (
    "# Uses of the old forms (skein's docs/design/draft/dynamic-memory.md, 7, and component-kit.md, 4): "
    'each count may only fall.\n'
    '# Lower them with `python3 -I -B scripts/old_forms.py --write`; '
    'only the director raises one, recorded in a `# Raised:` line.\n'
)
RULE = (
    "Uses of the old forms may only fall (skein's docs/design/draft/dynamic-memory.md, 7, "
    'and component-kit.md, 4): new code takes the new forms (Queue::uncapped, Journal::uncapped, '
    "decide, backlog, and the component kit's shared items in lib and io), "
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
        name = path.decode('utf-8')
        source = Path(name).read_text(encoding='utf-8')
        forms = [
            form for form in FORMS
            if not name.startswith(EXCLUDE.get(form, ()))
            and any(marker in source for marker in MARKERS[form])
        ]
        if not forms:
            continue
        source = without_comments(source)
        # Split the words once, for the forms in WORDS.
        words = IDENTIFIER.findall(source) if any(form in WORDS for form in forms) else ()
        for form in forms:
            text = source
            if form in WORDS:
                text = ' '.join(word for word in words if any(marker in word for marker in MARKERS[form]))
            totals[form] += sum(
                1 for match in FORMS[form].finditer(text)
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
            if (list(baseline) != list(FORMS)[:len(entries)] or len(entries) != len(baseline)
                    or any(count < 0 for count in baseline.values())):
                raise ValueError('expected each form once, in order, with nonnegative counts')
        except ValueError as error:
            parser.error(f'invalid {BASELINE}: {error}')
        # Forms added since the baseline was written: their rows start at
        # today's counts.
        missing = list(FORMS)[len(baseline):]
        if missing and not args.write:
            parser.error(f'{BASELINE} records no {", ".join(missing)}; initialize with --write')
        if missing and lines and not lines[-1].endswith('\n'):
            lines[-1] += '\n'
        for form in missing:
            baseline[form] = current[form]
            lines.append(f'{form} {current[form]}\n')
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
