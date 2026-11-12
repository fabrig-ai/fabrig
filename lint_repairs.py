import json
import re
from pathlib import Path
from collections import defaultdict, Counter
allowed = set('assert_is_empty manual_assert_eq byte_char_slices double_must_use unneeded_wildcard_pattern extend_with_drain drain_collect manual_is_variant_and useless_conversion explicit_iter_loop redundant_else needless_late_init question_mark'.split())
edits = defaultdict(set)
counts = Counter()
for line in Path('nightly-clippy-third.jsonl').read_text().splitlines():
    row = json.loads(line)
    if row.get('reason') != 'compiler-message':
        continue
    msg = row['message']
    code = (msg.get('code') or {}).get('code', '').removeprefix('clippy::')
    if msg['level'] != 'error' or code not in allowed:
        continue
    for child in msg['children']:
        spans = [s for s in child['spans'] if s.get('suggested_replacement') is not None]
        if not spans or any(s['suggestion_applicability'] != 'MachineApplicable' for s in spans):
            continue
        for span in spans:
            replacement = span['suggested_replacement']
            if code == 'assert_is_empty':
                replacement = re.sub(r'\[\] as (\[[^\n]*?; 0\])', r'<\1>::default()', replacement)
            edit = (span['byte_start'], span['byte_end'], replacement)
            if edit not in edits[span['file_name']]:
                counts[code] += 1
            edits[span['file_name']].add(edit)
for name, spans in edits.items():
    path = Path(name)
    text = path.read_bytes()
    end = len(text)
    for start, stop, replacement in sorted(spans, reverse=True):
        assert stop <= end, (name, start, stop, end)
        text = text[:start] + replacement.encode() + text[stop:]
        end = start
    path.write_bytes(text)
print('Edited', len(edits), 'files; suggestion spans by lint:', dict(counts))
