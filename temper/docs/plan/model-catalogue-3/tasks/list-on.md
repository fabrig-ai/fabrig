+++
id = "list-on"
part = "on"
kind = "feature"
needs = ["temper-worker", "core-reads", "offer-narrows", "client-facts", "list-fake"]
files = ["temper/crates/temper-init/template/temper.json", "temper/crates/temper-init/template/bare.json", "temper/crates/temper-init/src/tests/documents.rs", "temper/docs/examples/owned.json", "temper/crates/temper-fake/src/llm.rs", "temper/crates/temper-fake/src/wire.rs", "temper/crates/temper-fake/src/tests/**", "temper/crates/temper-fake/src/backends/**", "temper/docs/help/reference/configuration-engine.md", "temper/docs/design/client/journeys/*/recording/**", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 110
+++

# A box reads its lists

## Goal

`temper init`'s patterns turn reading on, and temper's fake answers it, so that a fresh box reads a fake provider's list as an account becomes usable, with the same stock offered.

Acceptance:

- `temper init`, box and bare, gives OpenAI's and Anthropic's `keep` and `drop` as `temper/docs/design/protocol/configuration.md`, 13.1 says, and none on the scripted provider. `temper-init/src/tests/documents.rs:65,169,590,640` hold them; `temper/docs/examples/owned.json:25` gains them; `configuration-engine.md` regenerates.
- temper's fake provider answers its list: `temper-fake/src/wire.rs:79` gives `Config.list` from `temper-fake/src/llm.rs:20`'s advertised models, in the form of the route's credential: a plan's `models` with `slug`, `display_name` and `visibility: list`. A fake's advertised names hold the provider's stock, so that a box's offer doesn't move; a row of `temper-fake`'s `tests/demo.rs` holds its body, parsed by `skein_llm::list::parse`.
- A focused story in `temper-fake` or `temper/tests/engine/domain`: the stock's names survive `kept` for every stock row with the patterns the templates give, each provider's: no row of the stock is dropped by a pattern. It reads the real template.
- Journeys: `temper-client-journeys again` for `welcome-playground`, `welcome-set-up`, `project-landing` and `chat-draft`, since a fresh box now reads a list and the `accounts` document moves. Their replay checks pass. Recording flakes as `common.md` says.
- A read's `Read from OpenAI` line shows in a recording only where the recording's pages show Models; if it shows elsewhere, a mark is wrong: it's yours to find, by `contracts.md`, 9, or to report.
- `configuration.status.md`'s 13.1 line goes.
- `heavy cargo check -p temper --tests` compiles the process tier's fixtures; no tier runs.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 1 and 3.
- `temper/docs/design/protocol/configuration.md`, 13.1; `jig/docs/design/protocol/configuration.md`, section 3; `skein/docs/design/fake-llm.md`, 3.2.

## Owns and may change

Owns the header's files.

- **Fixtures with their own documents need nothing,** `common.md` says so: `temper/crates/temper/tests/fixtures/cluster/engine.json:75`, `tests/support/github/engine.json:74`, `temper-playground/stories/a/deployment.json:177`, `temper-service/src/tests/fixtures.rs:13`, `temper/tests/real-loop/src/fixtures.rs:78`. A process-tier test that copies a template's document may meet a read after landing; after-landing repairs it.
- `temper/tools/journeys/src/again.rs:49` launches `temper box --fake-backends`: read it, change nothing unless a mark needs widening, as `common.md` says.

Options, each pre-approved:

- A second provider's fake on a fake box, if it has one, answers its route's form: Anthropic's `data` with `id`.
- If the fake's document names no advertised model for a stock row of its provider, add it to the fake's list: the stock's the fake's to serve.

## Steps

This changes what a fresh box does: run its readers' tests. Then `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-init -p temper-fake --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-init -p temper-fake -p temper-client-journeys`; `heavy cargo nextest run -p temper --lib -E 'test(/configuration|help/)'`; `heavy cargo check -p temper --tests`. Commit and push.

## Report

As the implementer's text says.
