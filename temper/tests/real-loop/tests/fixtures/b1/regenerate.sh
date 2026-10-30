#!/bin/sh
# B1 rung-b1-6, box3 on main 44230b8fc, after its clean stop.
# Run from the workspace root; replace forge identities with the twin's.
# The address marker is bound to the kernel-assigned listener by repair::recipe.
set -eu
heavy env TEMPER_BUNDLE=none NEXTEST_DOUBLE_SPAWN=0 cargo run -p temper-twin -- \
  --data "${1:-/home/free/src/rust/tmp/qa-focused/rung-b1-6/data3/deployment}" \
  --base temper/tests/real-loop/tests/fixtures/b1/llm-base.json \
  --out temper/tests/real-loop/tests/fixtures/b1/llm.json \
  --cue '1=## Replying' \
  --cue '2=You are a goal: the work your person asked for' \
  --cue '4=Implement CSV export in src/main.rs after requester approved.' \
  --cue '5=Repair the failed CI on the change branch' \
  --cue '6=Review this change at the named head' \
  --replace '2804e16853d74af24043f5fb61a0997edc47406e=1d0b14e4db018fed000000000000000300000000' \
  --replace '836967f08039a68ff3cb644b454d8b68c0d8c934=6e73e372e2338aca000000000000000400000000' \
  --replace '836967f08039=6e73e372e233' \
  --replace 'http://127.0.0.1:8368=http://B1-FORGE'
