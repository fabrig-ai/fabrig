# A stopped run's twin

`temper-twin` turns the selected tasks' transcripts from a cleanly stopped deployment
into a `temper fake llm --document` document (temper/docs/design/testing/fakes.md, 1).
The binary is in `temper/bin/temper-twin`; its crate is `temper-twin-tool`.
It is a developer tool, never shipped or published.

```sh
env TEMPER_INTERNAL_BUNDLE=none cargo run -p temper-twin --bin temper-twin -- \
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
or 1 when absent. Untouched arguments are embedded as recorded JSON, preserving
their spacing and member order; a subtree changed by a take is JSON-encoded.
The fake applies its normal compact JSON encoding when it serves arguments. `--replace OLD=NEW` applies
literal, non-overlapping byte substitutions to arguments and assistant text, in option
order, counting occurrences per option. An empty OLD or a replacement that
produces invalid object JSON is refused. Reports list turns per task, occurrences per replacement, uses per take and
opaque blocks dropped per task. An argument refusal
prints its reason and usage and exits 2. A conversion refusal prints one line
and exits 2; no output is opened on a conversion refusal.

`--takes FILE` applies after replacements. FILE is a JSON list of at most eight
`{"name","value","before","after"}` objects: `value` is the recorded text after
`--replace`; `before` and `after` delimit its value in the replay's conversation.
Names are distinct, 1–32 ASCII letters, digits or `_`; values are 1–256 bytes
without a line feed, and delimiters 1–256 bytes each. A whole JSON number token
becomes `"{{#NAME}}"`; occurrences inside decoded argument strings and assistant
text become `{{NAME}}`. Every turn using a name gains its take line before its
other lines, in FILE order. An unused take is refused by name. Counts are
occurrences, not turns.

When the recorded value also means something else, add `"only"`: one to eight
places, each with optional `call`, `argument`, `prefix` and `suffix`. A `call`
selects that call's arguments; `argument` narrows them to the value at an RFC 6901
pointer (and needs `call`). Without `call`, the place selects assistant text.
`prefix` and `suffix`, each at most 256 decoded bytes, constrain string/text
occurrences and remain unchanged. Overlapping places count an occurrence once.
For example, `{"call":"message","argument":"/target"}` leaves depth constants
alone; `{"prefix":"PR #"}` takes just that number in assistant text.

An existing twin can gain replacements and takes without its transcripts:

```sh
cargo run -p temper-twin -- --from twin.json --takes takes.json --out portable.json
```

`--from` cannot be combined with `--data`, `--base` or `--cue`. It preserves
non-script members and transforms `cued`, `asked` and `otherwise`, including
short text turns. Transcripts still replace only the two selected script lists,
leaving the base's fallback unchanged. Both forms validate the output through
temper-fake before opening `--out`.

The bounds are the fake's: sixteen scripts, sixteen turns per script, sixteen lines
per turn, 256-byte nonempty cues, 64 KiB answers, 256 KiB total scripts and
a 1 MiB document. The fake's decoder checks its detailed JSON and byte bounds.
Opaque blocks are dropped, keeping text and calls in order. Cut calls and
compactions are refused with task and turn; bad or partial frames include
task and byte offset. Versions must be those
of this build's Jig and Smith codecs. There are no upgrades and the store is
never opened. Jig's `protocol/store/transcripts.md`, 1–4, owns frame layout and the
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
The fixtures' takes bind message targets, pull numbers, heads and merges from
the replay's conversation, rather than requiring the walk's numbers and git
identities to recur. The scripts' forge reads infer their repository; no grant
carries a listener address.

```sh
env TEMPER_INTERNAL_BUNDLE=none NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper-twin-tool -p temper-twin
env TEMPER_INTERNAL_BUNDLE=none NEXTEST_DOUBLE_SPAWN=0 cargo nextest run \
  -p temper-real-loop-world --profile fuzzy \
  --test suite -E 'test(fuzzy::green_light::)'
```
