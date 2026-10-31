"""Tree forms for the shared repository ratchet; run from this tree's root."""
from pathlib import Path
import re
import runpy
import sys

ENGINE = runpy.run_path(str(Path(__file__).resolve().parents[2] / 'tools/old_forms/old_forms.py'))

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
