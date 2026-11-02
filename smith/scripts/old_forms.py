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
    'max_out': (
        # The provider's wire vocabulary stays on the wire.
        'max_output_tokens',
        # The benchmark grader bounds subprocess output at its capture edge.
        'max_output_bytes',
    ),
}
# A frozen benchmark repository's own crate, outside smith's workspace,
# whose names smith does not own.
EXEMPT_DIRECTORIES = ('benchmarks/tasks/repository/shell-cwd/grader/',)
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
# Syntax forms ignore strings as well as comments; banned words keep strings.
FORMS.update({
    'actual': ENGINE['banned_word']('actual'),
    'genuine': ENGINE['banned_word']('genuine'),
    'allow': re.compile(r'#\s*!?\s*\[\s*allow\s*\('),
    'expect_manual_let_else': re.compile(r'\bexpect\s*\(\s*clippy\s*::\s*manual_let_else\b'),
    'expect_single_match': re.compile(r'\bexpect\s*\(\s*clippy\s*::\s*single_match\b'),
    'full_limits': ENGINE['full_limits'],
    'temp_dir': re.compile(r'\btemp_dir\s*\(\s*\)'),
    # One-byte hex literals in kind fields or first arguments of wire writers.
    'bare_wire_kind': re.compile(
        r'(?:\bkind\s*:\s*|\b(?:frame_writer|write_record|push_record|encode_record)\s*\(\s*)'
        r'0x[0-9a-fA-F]{2}\b'),
    'io_resume': re.compile(r'\b(?:skein_)?io\s*::\s*resume\b'),
    'test_windows': ENGINE['test_windows'],
    'path_attribute': re.compile(r'#\s*!?\s*\[\s*path\b'),
    'domain_while': ENGINE['domain_keyword']('while'),
    'domain_loop': ENGINE['domain_keyword']('loop'),
})
SYNTAX_FORMS = tuple(form for form in FORMS if form not in (*WORDS, 'takes', 'actual', 'genuine'))
# Seven production domain crates; test harnesses use ordinary Rust (Q7).
DOMAIN_PATH = re.compile(
    r'^crates/(?:smith-domain(?:-[^/]+)?|smith-host-domain|smith-local-domain|smith-inline-agent)/')
INCLUDE = {'domain_while': DOMAIN_PATH, 'domain_loop': DOMAIN_PATH}
COMMENT_FORMS = {
    'actual': ENGINE['banned_word']('actual'),
    'genuine': ENGINE['banned_word']('genuine'),
    'originally': re.compile(r'\boriginally\b', re.I),
    'previously': re.compile(r'\bpreviously\b', re.I),
    'no_longer': re.compile(r'\bno\s+longer\b', re.I),
    'used_to': re.compile(r'\bused\s+to\b', re.I),
    'formerly': re.compile(r'\bformerly\b', re.I),
    # Standalone outer doc-comment stamps; excludes //! module citations.
    'contract_stamp': re.compile(r'^\s*///\s*contract\s*:', re.I | re.M),
    'lower_test_of': re.compile(r'\bLower\s+test\s+of\b'),
    'dead_citation': ENGINE['dead_citations'],
}

without_comments = ENGINE['without_comments']


def counts():
    return ENGINE['counts'](globals())


def main():
    return ENGINE['main'](globals())


if __name__ == '__main__':
    sys.exit(main())
