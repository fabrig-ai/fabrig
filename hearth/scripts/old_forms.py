"""Tree forms for the shared repository ratchet; run from this tree's root."""
from pathlib import Path
import re
import runpy
import sys

ENGINE = runpy.run_path(str(Path(__file__).resolve().parents[2] / 'tools/old_forms/old_forms.py'))

FORMS = {
    'max_out': re.compile(r'\b\w*(?:max_out|MAX_OUT)\w*\b'),
    'worst_case': re.compile(r'\b\w*worst_case\w*\b'),
    # The component kit's counts (skein/docs/design/draft/component-kit.md, 4).
    'io_resume': re.compile(r'\b(?:skein_)?io::resume\b'),
    'earliest_helper': re.compile(r'\bfn\s+(?:earliest|earlier)\b'),
    'hex_decimal_fn': re.compile(r'\bfn\s+(?:hex|decimal)\b'),
    'host_forwarder': re.compile(r'\bimpl\s+(?:\w+::)*Host\s+for\s+Process\b'),
    'path_attribute': re.compile(r'#\s*!?\s*\[\s*path\b'),
    'domain_while': ENGINE['domain_keyword']('while'),
}
# Text without spaces that every match of a form holds, which stripping
# comments never makes: a file holding none of a form's markers is not
# searched for it.
MARKERS = {
    'max_out': ('max_out', 'MAX_OUT'),
    'worst_case': ('worst_case',),
    'io_resume': ('io::resume',),
    'earliest_helper': ('earliest', 'earlier'),
    'hex_decimal_fn': ('hex', 'decimal'),
    'host_forwarder': ('Process',),
}
WORDS = ('max_out', 'worst_case')
EXEMPT = {
    'max_out': {
        # The kernel's submission bounds stay at its edges.
        name: ('io', 'skein_io')
        for name in ('MAX_OUT_DOWN', 'MAX_OUT_UP', 'MAX_OUT_RESUME', 'MAX_OUT_FIRE')
    },
}
# Syntax forms mask strings: a panic's message saying "while" is not a loop.
SYNTAX_FORMS = ('path_attribute', 'domain_while')
# Every crate under crates/ is step code but the binary's shell (hearth/AGENTS.md).
INCLUDE = {'domain_while': re.compile(r'^crates/hearth-[^/]+/')}
BASELINE = Path('scripts/old-forms.txt')
HEADER = (
    '# Uses of the old forms (skein/docs/foundation/programming-model.md, 6.3, and component-kit.md, 4): '
    'each count may only fall.\n'
    '# Lower them with `python3 -I -B scripts/old_forms.py --write`; '
    'a count rises only with a `# Raised:` line saying who and why.\n'
)
RULE = (
    'Uses of the old forms may only fall (skein/docs/foundation/programming-model.md, 6.3, '
    'and component-kit.md, 4): new code takes the new forms (Queue::uncapped, Journal::uncapped, '
    "decide, backlog, and the component kit's shared items in lib and io), "
    'or removes as many old uses as it adds.'
)
COMMENT_FORMS = {
    'actual': ENGINE['banned_word']('actual'),
    'genuine': ENGINE['banned_word']('genuine'),
}

without_comments = ENGINE['without_comments']


def counts():
    return ENGINE['counts'](globals())


def main():
    return ENGINE['main'](globals())


if __name__ == '__main__':
    sys.exit(main())
