# A stopped run's twin

`twin` turns the selected tasks' transcripts from a cleanly stopped deployment
into a `temper fake llm --document` document (domain/testing.md, 4.1).
It is a developer tool, never shipped or published.

```sh
heavy env TEMPER_BUNDLE=none cargo run -p temper-twin --bin twin -- \
  --data /absolute/data/deployment --base /absolute/base.json \
  --out /absolute/twin.json \
  --cue '1=Add CSV export to widget' \
  --cue '2=ask the chat for a green light first' \
  --replace 'OLD=NEW'
```

`--data` names Jig's deployment directory, the resolved
`engine.deployment.directory` under Temper's data directory. It is the directory
holding `transcripts/` and `active`, not a generation's `store/` or `secrets/`.

Each `--cue TASK=PHRASE` takes a task's completions in file order, across
resumptions. A cue found in the opening user text becomes an `asked` script:
Smith puts a fresh Open chat's brief there. An opening with Smith's instruction
to begin the brief, as Finite and delegate runs have, stays `cued`, as does a
cue absent from the opening request. Pick distinct phrases of the person's
first request for Open chats and of the system brief for Finite runs and
delegates; replay the result to check selection. Both `asked` and `cued` are
replaced; other base members are preserved. The base and result are checked by
`temper_fake::llm::Document::decode` before any output is opened.

An `asked` script advances on tool results, including the messages returned
by a pending `wait`. A new user text selects from its own cues and restarts
the count. A recorded Open script therefore replays its opening request's
sequence; a later person's request needs its own script.

A completion's ordered text and calls become one turn. Calls finish
`tool_calls`, other completions `stop`. Output tokens are the recorded count,
or 1 when absent. Arguments are embedded as recorded JSON, preserving their
spacing and member order in the document; the fake subsequently applies its
normal compact JSON encoding when it serves them. `--replace OLD=NEW` applies
literal, non-overlapping byte substitutions to arguments and assistant text, in option
order, counting occurrences per option. An empty OLD or a replacement that
produces invalid object JSON is refused. Reports list turns per task and
occurrences per replacement and opaque blocks dropped per task. A refusal
prints one line and exits 2; no output is opened on a conversion refusal.

The bounds are the fake's: sixteen scripts, sixteen turns per script, sixteen lines
per turn, 256-byte nonempty cues, 64 KiB answers, 256 KiB total scripts and
a 1 MiB document. The fake's decoder checks its detailed JSON and byte bounds.
Opaque blocks are dropped, keeping text and calls in order. Cut calls and
compactions are refused with task and turn; bad or partial frames include
task and byte offset. Versions must be those
of this build's Jig and Smith codecs. There are no upgrades and the store is
never opened. Jig's `protocol/store.md`, 8.1–8.4, owns frame layout and the
clean-stop guarantee: a complete checksummed frame does not independently
prove its commit. Supply only data from a successful drained stop, never a
crash or failed commit. The reader detects incomplete or corrupt tails but
does not inspect the map's committed-length records.

The small codec fixtures are actual Jig frames recorded from the original
hand-written green-light story before B0 replaced it. They contain synthetic
test data only, no credential files.

The converter's round-trip tests reuse the checked green-light frames. They
play each converted task's cued script through the fake LLM domain, encode and
decode OpenAI response events, then encode Smith turns into Jig frames and
convert again. Normalized text, calls, finish modes, token counts and task/cue
mapping stay equal; opaque drops and argument replacements are counted across
both conversions. These tests run without a service, ring or world. The B0
real-loop smoke checks the story's landing path. Normal tests never rewrite
fixtures. Regenerate B0's document from its cleanly stopped
passing walk using the command named in `tests/real-loop/src/green_light.rs`.
Task, question and pull numbers and fake-git identities recur. The scripts'
forge reads infer their repository; no grant carries a listener address.

```sh
heavy env TEMPER_BUNDLE=none NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper-twin
heavy env TEMPER_BUNDLE=none NEXTEST_DOUBLE_SPAWN=0 cargo nextest run \
  -p temper-real-loop-world --profile fuzzy \
  --test suite -E 'test(fuzzy::green_light::)'
```
