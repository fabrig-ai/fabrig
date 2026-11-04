"""Tree forms for the shared repository ratchet; run from this tree's root."""
from pathlib import Path
import re
import runpy
import sys

ENGINE = runpy.run_path(str(Path(__file__).resolve().parents[2] / 'tools/old_forms/old_forms.py'))

FORMS = {
    'max_out': re.compile(r'\b\w*(?:max_out|MAX_OUT)\w*\b'),
    'worst_case': re.compile(r'\b\w*worst_case\w*\b'),
}
# Text without spaces that every match of a form holds, which stripping
# comments never makes: a file holding none of a form's markers is not
# searched for it.
MARKERS = {
    'max_out': ('max_out', 'MAX_OUT'),
    'worst_case': ('worst_case',),
}
BASELINE = Path('scripts/old-forms.txt')
HEADER = (
    '# Uses of the old forms (skein/docs/foundation/programming-model.md, 6.3): each count may only fall.',
    '# Lower them with `python3 -I -B scripts/old_forms.py --write`; a count rises only with a `# Raised:` line saying who and why.',
)
RULE = (
    "Uses of the old forms may only fall (skein's docs/foundation/programming-model.md, 6.3): "
    'new code takes the new forms, or removes as many old uses as it adds.'
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
