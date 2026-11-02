"""Tree forms for the shared repository ratchet; run from this tree's root."""
from pathlib import Path
import re
import runpy
import sys

ENGINE = runpy.run_path(str(Path(__file__).resolve().parents[2] / 'tools/old_forms/old_forms.py'))

FORMS = {
    'max_out': re.compile(r'\b\w*(?:max_out|MAX_OUT)\w*\b'),
    'JournalRoom': re.compile(r'\bJournalRoom\b'),
    'takes': re.compile(r'\.takes\(&'),
    'worst_case': re.compile(r'\b\w*worst_case\w*\b'),
}
EXEMPT = {
    'max_out': {
        # The kernel's submission bounds stay at its edges.
        name: ('io', 'skein_io')
        for name in ('MAX_OUT_DOWN', 'MAX_OUT_UP', 'MAX_OUT_RESUME', 'MAX_OUT_FIRE')
    },
}
BASELINE = Path('scripts/old-forms.txt')
HEADER = (
    "# Uses of the old forms (skein's docs/foundation/programming-model.md, 6.3): each count may only fall.\n"
    '# Lower them with `python3 -I -B scripts/old_forms.py --write`; '
    'only the director raises one, recorded in a `# Raised:` line.\n'
)
RULE = (
    "Uses of the old forms may only fall (skein's docs/foundation/programming-model.md, 6.3): "
    'new code takes the new forms (Queue::uncapped, Journal::uncapped, decide, backlog), '
    'or removes as many old uses as it adds.'
)
MARKERS = {
    'max_out': ('max_out', 'MAX_OUT'), 'JournalRoom': ('JournalRoom',),
    'takes': ('takes',), 'worst_case': ('worst_case',),
}
WORDS = ('max_out', 'JournalRoom', 'worst_case')
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
