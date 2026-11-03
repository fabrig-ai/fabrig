+++
id = "openai-stories-manual"
part = "openai-live-manual"
kind = "docs"
needs = []
files = [
  "skein/tests/llm/tests/live.rs",
  "skein/crates/skein-llm/README.md",
  "skein/docs/design/llm.status.md",
  "skein/docs/design/STATUS.md",
]
effort = "medium"
minutes = 25
+++

# The two positive OpenAI live stories are ignored, owner-run like the Anthropic ones

## Goal

The live suite's OpenAI stories call the public Responses route, and the Codex login lent to
`live-skein` is refused there with `401 Missing scopes: api.responses.write`; no lent login
serves the route in a form the suite reads. So `openai_streamed_text_and_history` and
`openai_tool_round_trip` become manual stories, ignored like the three Anthropic ones beside
them; `openai_unauthorized` keeps running, since it sends an invalid token on purpose. The
status and the README say so.

Acceptance:

- In `skein/tests/llm/tests/live.rs`, `openai_streamed_text_and_history` (~505) and
  `openai_tool_round_trip` (~514) each carry, between `#[test]` and `fn`, exactly
  `#[ignore = "live: needs a login that holds api.responses.write; run by the owner, how is decided later (skein/docs/design/llm.status.md, 5)"]`.
  Nothing else of the file changes: `openai_unauthorized` stays un-ignored; `input`,
  `text_and_history`, `tool_round_trip`, `unauthorized` and the offline tests are as they
  are.
- `heavy cargo nextest list --profile live-skein -p skein-llm-world --test live` lists exactly
  four tests: `openai_unauthorized`, `a_login_is_read_from_its_file_or_the_directory_holding_it`,
  `a_login_lacking_a_field_is_refused_naming_only_the_field` and
  `an_anthropic_key_file_requires_private_nonempty_plaintext`. With `--run-ignored only` it
  lists exactly five: the two OpenAI stories above and the three `anthropic_*` ones.
- The three offline login tests pass under `live-skein` (step 4); no provider story runs
  (`notes/common.md`).
- `heavy cargo clippy -p skein-llm-world --all-targets -- -D warnings` is clean.
- `skein/docs/design/llm.status.md`, `skein/docs/design/STATUS.md` and
  `skein/crates/skein-llm/README.md` hold the texts of Steps 2 and 3.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing broken.

## Inputs

- `temper/docs/plan/openai-manual/notes/common.md`.
- `temper/docs/plan/openai-manual/inputs/request.md`.
- `skein/docs/design/llm.md, 5`: the sentence beginning "Live admission, with a key or a
  plan's token, is not implied by offline tests" (read only; the design is unchanged).
- `skein/docs/design/llm.status.md`: its one **Departs** entry, section 5, which this task
  widens.
- `skein/crates/skein-llm/README.md`: the live suite's paragraphs (~89-136).

## Owns and may change

Owns:

- `skein/tests/llm/tests/live.rs`: the two `#[ignore = ...]` lines above, nothing else.
- `skein/crates/skein-llm/README.md`: the live section's hunk of Step 3, nothing else.
- `skein/docs/design/llm.status.md` and `skein/docs/design/STATUS.md`: the texts of Step 2,
  nothing else. These two are the task's one departure from the implementer's text ("Not
  yours: any `docs/design/`"): the lead wrote the texts; the task places them with the code
  they describe.

Read only: `.config/nextest.toml` (`[profile.live-skein]`: selects only the `live` binary),
the rest of `live.rs`.

No focused or fuzzy run: the only file of the package this task changes is the `live`
binary, which both profiles exclude (a stated departure from the implementer's text); the
clippy run compiles it and step 4 runs its offline tests.

Pre-approved options:

- If heavy refuses `cargo nextest list`, report it under `## Left` with heavy's message; in
  place of the two listings, check that `live.rs` holds exactly five `#[ignore = "live:`
  lines (`rg -c`), each directly above one of the five ignored tests' `fn`, and Done stands
  on the other acceptance. Never run the binary without the step-4 filter to see what it
  selects.
- If an offline login test fails, it is not this change's (attributes and prose only): read
  its log once, run it once alone; passing, report it as a flake with both logs and the
  machine; failing, report it under `## Left` with the log's path.
- If the citations check flags a line this task did not write, report it under `## Left`;
  one it wrote, fix it in the task's own text and say how in the report.

## Steps

1. `live.rs`: add the attribute to the two tests, in the form of the Anthropic ones beside
   them:

   ```rust
   #[test]
   #[ignore = "live: needs a login that holds api.responses.write; run by the owner, how is decided later (skein/docs/design/llm.status.md, 5)"]
   fn openai_streamed_text_and_history() {
   ```

   and the same above `fn openai_tool_round_trip()`.

2. The status texts.

   `skein/docs/design/llm.status.md` becomes, whole (the date stays 2026-11-03):

   ```markdown
   # llm: status

   2026-11-03. Where skein's code lags or departs from [llm.md](llm.md).

   - **Departs:** 5: five of the live suite's provider stories are ignored, manual
     stories run by the owner; how the owner runs them is to be decided later.
     `anthropic_streamed_text_and_history`, `anthropic_tool_round_trip` and
     `anthropic_unauthorized` need an Anthropic API key: none is provided or wanted
     for the automatic suite, by the user's decision of 2026-11-03.
     `openai_streamed_text_and_history` and `openai_tool_round_trip` need a login
     whose access token holds the public route's scope, `api.responses.write`: the
     Codex login lent to the suite lacks it and is refused (`401 Missing scopes:
     api.responses.write`), and no lent login serves the route in a form the suite
     reads, by the director's decision of 2026-11-03, which the user takes.
     `openai_unauthorized` stays in the automatic suite: it sends the lent token
     replaced by an invalid one.
   ```

   `skein/docs/design/STATUS.md`'s summary still counts `llm.md` among the documents with no
   gap. In its first paragraph, replace

   ```markdown
   in `lib.md`, `examples.md` and `ui.md`. The code departs in three
   crates' `tests/` directories (`testing.md`). `README.md`, `channel.md`,
   `codec.md`, `kv.md`, `world.md`, `fake-checkout.md`, `fake-llm.md`,
   `forge-client.md`, `forge-connection.md`, `forge-setup.md`, `llm.md`,
   `llm-connection.md`, `oauth.md`, `oauth-issuer.md`, `ui-text.md` and
   `browser.md` have no gap.
   ```

   with

   ```markdown
   in `lib.md`, `examples.md` and `ui.md`. The code departs in three
   crates' `tests/` directories (`testing.md`), and five of the LLM live
   suite's provider stories are manual, run by the owner (`llm.md`).
   `README.md`, `channel.md`, `codec.md`, `kv.md`, `world.md`,
   `fake-checkout.md`, `fake-llm.md`, `forge-client.md`,
   `forge-connection.md`, `forge-setup.md`, `llm-connection.md`,
   `oauth.md`, `oauth-issuer.md`, `ui-text.md` and `browser.md` have no
   gap.
   ```

   Nothing else of either file.

3. `skein/crates/skein-llm/README.md`. Replace the text from `There are no retries or
   credential refreshes.` (~93) through the closing fence of the Anthropic `sh` block (~111)
   with:

   ````markdown
   There are no retries or credential refreshes. OpenAI's text and tool stories use
   subscription quota and require a valid, unexpired OAuth access token that holds
   the public route's scope, `api.responses.write`. Anthropic tests use an API key.

   The automatic live suite calls a provider only in `openai_unauthorized`, which
   sends the lent login's token replaced by an invalid one. Name a Codex login, then
   run it:

   ```sh
   export SKEIN_TEST_LIVE_OPENAI_AUTH="$HOME/.codex/auth.json"
   heavy cargo nextest run -p skein-llm-world --profile live-skein
   ```

   The other five provider stories are ignored and manual
   (skein/docs/design/llm.status.md, 5). The two OpenAI stories,
   `openai_streamed_text_and_history` and `openai_tool_round_trip`, need a login of
   the same form whose access token holds `api.responses.write`, which the Codex
   login lacks; the owner names it in `SKEIN_TEST_LIVE_OPENAI_AUTH`. The three
   Anthropic stories, `anthropic_streamed_text_and_history`,
   `anthropic_tool_round_trip` and `anthropic_unauthorized`, need a private plaintext
   key file, which the owner names in `SKEIN_TEST_LIVE_ANTHROPIC_KEY`. The owner sets
   the variable a story reads and runs it alone, replacing `NAME` with its exact test
   name:

   ```sh
   export SKEIN_TEST_LIVE_OPENAI_AUTH="/path/to/login/auth.json"         # an OpenAI story
   export SKEIN_TEST_LIVE_ANTHROPIC_KEY="/path/to/private/anthropic-key" # an Anthropic story
   heavy cargo nextest run -p skein-llm-world --test live --profile live-skein --run-ignored only -E 'test(=NAME)'
   ```
   ````

   The paragraphs after it (from `SKEIN_TEST_LIVE_OPENAI_AUTH` names a Codex `auth.json`)
   stay as they are.

4. Checks, one at a time, from the repository root:
   - `heavy cargo clippy -p skein-llm-world --all-targets -- -D warnings`;
   - `heavy cargo nextest list --profile live-skein -p skein-llm-world --test live`: the four
     of the acceptance, and no other;
   - `heavy cargo nextest list --profile live-skein -p skein-llm-world --test live
     --run-ignored only`: the five, and no other;
   - `heavy cargo nextest run --profile live-skein -p skein-llm-world --test live -E
     'test(/^an?_/)'`: the three offline login tests, passed (check it selected three and
     that `openai_unauthorized` was not among them).

5. `cargo fmt`; from `skein/`, `python3 -I -B scripts/old_forms.py` (from the root it finds no
   `scripts/old-forms.txt`): no count rises.

6. One commit, `llm: keep the OpenAI text and tool live stories owner-run`, saying why (the
   lent Codex login lacks the public route's `api.responses.write`); then, from the root,
   `python3 -I -B tools/citations/citations.py --base origin/main`. Push your branch.

## Report

As the implementer's text says, and also: both listings as nextest printed them, the
offline run's count, and the citations check's output.
