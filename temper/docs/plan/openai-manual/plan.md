# openai-manual: the plan

From `inputs/request.md` (the director, 11-03, on after-landing's red-10): skein's OpenAI live
stories call the public Responses route, and the Codex login lent to `live-skein` is refused
there with `401 Missing scopes: api.responses.write`; no lent login serves the route in a form
skein reads. Like the three Anthropic stories before them, the two positive OpenAI stories
become manual. One direct part, one task; shared rules in `notes/common.md`.

## Parts and tasks

- **openai-live-manual** (`skein/docs/design/llm.md, 5`), slice 1.
  - `tasks/openai-stories-manual.md`, docs, medium, ~25 minutes, needs nothing:
    `openai_streamed_text_and_history` and `openai_tool_round_trip` in
    `skein/tests/llm/tests/live.rs` get the request's `#[ignore = "live: needs a login that
    holds api.responses.write; ..."]`; `openai_unauthorized` keeps running. The task places
    the lead's texts in `skein/docs/design/llm.status.md` (section 5's **Departs** entry, now
    five stories with both reasons) and `skein/docs/design/STATUS.md` (its summary no longer
    counts `llm.md` among the documents with no gap), and rewrites the README's live
    section. Its check is the request's: `live-skein` lists four tests (`openai_unauthorized`
    and the three offline login tests), five with `--run-ignored only`, and the three offline
    tests run by name; no provider story runs.

## Critical path

`openai-stories-manual` (~25 minutes, up to ~50 with the buffer); then fab's end check on
the trunk. The next `live-skein` run on a main holding it is the request's proof: it calls a
provider only in `openai_unauthorized`.

## Overlaps

None found. No neighbouring draft (`reference-m10`, `configuring-p2-build`, and the queued
`v0.1.0-ladder`, `sign-in-wait`, `reference-walks`) names `skein/tests/llm/`,
`skein/crates/skein-llm/`, `llm.status.md` or skein's `STATUS.md`, against the trunk at
8c124bc391. No test reads the README or either status file (`.config/gate/read-by-tests.txt`),
and nothing else names the two tests.

## Calls made

- **The status texts are the lead's, placed by the task.** The status records the code as it
  stands, so the entry lands with the attributes it describes; the spec gives the texts whole,
  a stated departure from the implementer's text, as `live-temper-stories-2` did.
- **STATUS.md too.** skein's index still said `llm.md` has no gap, stale since the Anthropic
  stories went manual (75155a68dd); the same task corrects it, one sentence.
- **The README says what the automatic suite still needs.** `openai_unauthorized` builds its
  call through `input`, which reads the lent login's token field before replacing it, so the
  automatic run still names a Codex login; the README says its token is sent replaced, and
  that the manual OpenAI stories need a login of the same form holding
  `api.responses.write`.
- **No focused or fuzzy run.** Both profiles exclude the `live` binary, the only file of
  `skein-llm-world` the task changes; clippy compiles it, and its offline tests run by name
  under `live-skein`.
- **`--list` is `cargo nextest list`:** nextest's `run` takes no `--list`; the spec names the
  subcommand, and a fallback if heavy refuses it.
- **No contracts note:** one part, no shape crosses it. No question: the request settles it.
