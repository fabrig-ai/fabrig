#!/bin/sh
# B0 walk 1, after its clean stop; run from the workspace root.
# Quote the valid head value to preserve the walk's failing, padded first lookup.
# The next replacements correct that head in its failure report and the goal's merge abbreviation.
set -eu
heavy env TEMPER_BUNDLE=none NEXTEST_DOUBLE_SPAWN=0 cargo run -p temper-twin -- \
  --data "${1:-/home/free/src/rust/tmp/qa-focused/rung-b0-1/data/deployment}" \
  --base tests/real-loop/tests/fixtures/b0/llm-base.json \
  --out tests/real-loop/tests/fixtures/b0/llm.json \
  --cue '1=## Replying' \
  --cue '2=You are a goal: the work your person asked for' \
  --cue '4=Implement the approved CSV export for widget.' \
  --cue '5=Review this change at the named head' \
  --cue '6=This review is asked again at the same head' \
  --replace '"86dad3c2ee1ccca46932d124ac00c07716f4fc42"="975835de1c9756ce000000000000000200000000"' \
  --replace '86dad3c2ee1ccca46932d124ac00c07716f4fc42.=975835de1c9756ce000000000000000200000000.' \
  --replace '83156a5=1d0b14e'
