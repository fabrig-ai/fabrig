#!/bin/sh
# B0 rung-b0-4, after its clean stop; run from the workspace root.
# Replace the walk's reviewed forge head with the twin's deterministic produced head.
# Replace the walk's merge identity with the twin's deterministic merge identity.
set -eu
heavy env TEMPER_BUNDLE=none NEXTEST_DOUBLE_SPAWN=0 cargo run -p temper-twin -- \
  --data "${1:-/home/free/src/rust/tmp/qa-focused/rung-b0-4/data/deployment}" \
  --base temper/tests/real-loop/tests/fixtures/b0/llm-base.json \
  --out temper/tests/real-loop/tests/fixtures/b0/llm.json \
  --cue '1=## Replying' \
  --cue '2=You are a goal: the work your person asked for' \
  --cue '3=You are an advisor: you read what your brief asks and report what you found; you change nothing and ask no one.' \
  --cue '5=Add CSV export to the widget in src/main.rs.' \
  --cue '6=Review this change at the named head' \
  --replace '4dfdeaa4e98fffe1d544c2e27689347d0041fe6a=975835de1c9756ce000000000000000200000000' \
  --replace '8fa3728dbceeaa330e087dbe2812e112237aa9e4=1d0b14e4db018fed000000000000000300000000'
