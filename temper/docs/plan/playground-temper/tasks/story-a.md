+++
id = "story-a"
part = "story"
kind = "feature"
needs = ["playground-processes", "fake-pace"]
files = [
  "temper/crates/temper-playground/stories/**",
  "temper/crates/temper-playground/src/stories.rs",
  "temper/crates/temper-playground/src/stories/**",
  "temper/crates/temper-playground/src/story.rs",
  "temper/crates/temper-playground/src/lib.rs",
  "temper/crates/temper-playground/Cargo.toml",
  "temper/crates/temper-playground/src/tests/**",
  "temper/docs/design/playground.status.md",
  "Cargo.lock",
]
effort = "high"
minutes = 90
+++

# Story A as data, checked as data

## Goal

A story is data, no code: the deployment's document, the fake LLM's and the fake forge's
documents in `temper-fake`'s form of a document of its own with their `pace`, and a description
(`temper/docs/design/playground.md`, 5.1). The first is A, the box's playground: its scripts from
one source, `temper-init`'s template, its `otherwise` worded for the browser (5.2, 5.4). This task
writes story A, `stories::{Story, A, all}` (C4), `Story::start` and the checks as data.

Acceptance:

- `temper/crates/temper-playground/stories/a/` holds `deployment.json`, `llm.json`, `forge.json`
  and `description.txt`, compiled in by `include_bytes!`/`include_str!` (C4).
- `llm.json`'s `asked` and `cued` scripts are `temper/crates/temper-init/template/temper.json`'s
  `application.demo.llm.asked` and `.cued`, word for word; its `otherwise` is one turn, `finish`
  `stop`, whose text is: "I am the scripted model of temper's playground, running in your browser:
  I answer only its three suggested requests: Add CSV export to widget, Add a dark theme to widget
  and Fix widget's empty search state. A real model comes with temper itself: download it to
  connect yours." (its `tokens` counted as the template's are); its `pace` is
  `{"reply_ms": 1200}`. Its `listen`, `key` and `models` are the composition's fake LLM's places
  (`fakes`).
- `forge.json`: owner `playground`, one repository `widget` landing on `main` with `ci` null, the
  box's seed (as `temper_fake::forge::Document::fake` seeds it), `pace` `{"ci_ms": 4000}`; its
  `listen` and `admin_token` the composition's fake forge's places.
- `deployment.json`: the composition's deployment document (`documents`) as one file, naming the
  scripted provider at the fake LLM's endpoint and the fakes' places, and no forge and no project
  (`StartPlayground` makes them: `temper/docs/design/protocol/setup.md`, 8.4).
- `description.txt`: "Try the playground, then ask the three suggested requests: the CSV export
  is delegated, pushed, passed by CI and landed on main; the dark theme and the empty search are
  answered in text."
- `Story::start(seed, wall)` decodes the fakes' documents by `temper_fake::llm::Document::decode`
  and `temper_fake::forge::Document::decode` and builds `Fakes` as `temper fake` builds its peers,
  with `Links::Print`; `stories::all()` is `[A]`.
- The tests below pass; the status entry below goes.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md` and `notes/contracts.md`, C3 and C4.
- `temper/docs/design/playground.md`, 5.1, 5.2 and 5.4.
- `temper/docs/design/protocol/setup.md`, 8.4: the documents of their own (their members and
  `pace`) and "Try the playground".
- The reports of `playground-processes` (`Start`, `Fakes`) and `fake-pace` (the paced names).

## Owns and may change

- `stories/a/*`, `src/stories.rs` (and `src/stories/`), tests under `src/tests/`; `src/lib.rs`
  its `pub mod stories;` and re-exports; `Cargo.toml` gains `temper-fake` and, as
  dev-dependencies, what the cue check needs (as `temper/crates/temper-fake/src/tests/demo.rs`
  uses: `smith-domain-run`, `smith-agent-world`, `serde_json`; test kits are allowed there).
- `src/story.rs`: a constructor of `Fakes` from `temper-fake`'s peers, if `Fakes`' fields cannot
  be built from them as they are; `Start`, `Fakes` and `Links` keep their shape (C3). If `Fakes`'
  forge (`temper_fake_git::Typed`) cannot be made from `temper_fake::forge`'s seeded forge at all,
  stop: that is a question.
- Status, as `notes/common.md` says: `temper/docs/design/playground.status.md`, **Not yet**:
  remove `5: stories as data, with the fakes' latencies as their pace.` and nothing else.

## Steps

1. The four files; `stories.rs`; `Story::start`.
2. Tests, in `src/tests/stories.rs`, checked as data as `temper-init`'s scripts are
   (`playground.md`, 5.1):
   - `story_a_decodes_within_the_fakes_bounds`: both fakes' documents decode, and the LLM's
     scripts fit its limits as `temper-fake/src/tests/demo.rs`,
     `the_templates_demo_fits_the_fakes_bounds` (~399), checks the template's.
   - `story_as_scripts_are_the_templates`: `asked` and `cued` equal the template's, read from
     `temper-init/template/temper.json` by `include_bytes!`; `otherwise` is the one above.
   - `story_as_cues_are_matched_apart`: each cue selects only its own agent's brief, as
     `the_csv_cues_select_only_their_own_agents_briefs` (~495) checks the template's.
   - `story_as_pace_is_paced`: the decoded LLM's latency is 1200 ms, the forge's CI 4000 ms.
   - `story_a_starts_and_prints_its_link`: `world(A.start(7, wall))` stepped until it waits idle
     prints one sign-in line on the service's standard output; no process failed.
   `heavy cargo nextest run -p temper-playground --lib`.
3. The status edit; the citations check.
4. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-playground --all-targets -- -D
   warnings`. Commit; push.

## Report

As the implementer's text says, and: `Story::start`'s path from documents to `Fakes`, and any
member of the template's scripts the fake's document of its own could not hold as it is.

## Rebase (2026-11-03, step merge, foreman playground-temper/story-a/foreman-1)

resolve the playground manifest conflict by retaining both tasks’ dependencies.
The merge conflicts in `temper/crates/temper-playground/Cargo.toml`.
Integration commit `7b9513785fa6d60c3d5e003c6577cc1758cdf03b` adds `temper-worker-checkout-world.workspace = true` under `[dev-dependencies]`; its fixture checks require it.
Keep that entry alongside story A’s `smith-domain-run.workspace = true` and `smith-agent-world.workspace = true`.
Keep story A’s `temper-fake.workspace = true` under `[dependencies]`.
Preserve the automatically merged `Cargo.lock`, checking that the `temper-playground` dependency list contains all four additions.
No source or story-data changes are needed for this conflict.
Validate with `heavy cargo nextest run -p temper-playground --lib` and `heavy cargo clippy -p temper-playground --all-targets -- -D warnings`.
