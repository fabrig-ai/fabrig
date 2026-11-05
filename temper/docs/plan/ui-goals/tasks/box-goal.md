+++
id = "box-goal"
part = "box-goal"
kind = "feature"
needs = ["goals-rail", "report-stock"]
files = [
  "temper/crates/temper-fake/src/backends.rs",
  "temper/crates/temper/tests/fake_backends/walk/goal.rs",
  "temper/crates/temper/tests/fake_backends/walk.rs",
]
effort = "high"
minutes = 75
+++

# A goal on the fake box shows its report

## Goal

The meeting point: `temper box --fake-backends`' scripted model starts a goal from a chat, which writes its report, and the client shows it in the rail and the list.

Acceptance:

- The fake model's document (`backends.rs`, `model_document`) gains: an `asked` script on a person's words that the walk names, whose chat calls `goal` and says so in words; a `cued` script on the root's line
  `You are a goal: the work your person asked for`, whose turns are `note` with the report (`temper/docs/plan/ui-goals/notes/contracts.md`, The report note), then `finish` with its report. Its other words stay "Hello from the local fake model.", as `walk.rs` asserts.
- A walk, `goal.rs`: on a fresh box, a chat in the first project with those words; `has_goals` turns true; the rail shows Goals; `Address::Goals` lists the goal with the report's headline, its state's word and its count.
- Its body's `Left:` and `Risk:` lines read by `report::parse`.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/domain/agent.md`, 8.3; `temper/docs/design/protocol/configuration.md`, 12.1.
- `temper/crates/temper/tests/fake_backends/walk/open_default.rs`; `temper/tests/real-loop/tests/fixtures/b0/llm.json` (a goal's turns); `skein/docs/design/fake-llm.md`, 2.4.

## Owns and may change

Owns, as `files` lists them. **`backends.rs` is beside `claude-always/always-offered`:** take its `model_document` line as it stands on your base and change only `"cued"` and `"asked"`.

Options, each pre-approved:

- **Scripts are static:** a report's scope needs literal numbers. Use those the walk's order makes true (a fresh box, its first chat and first goal), with a comment. A later chat's goal gets a refused note and ends without a report, a normal state of a fake.
- If the goal's `note` is refused in the walk, read the refusal before anything else; it is the story this task is for. A fault in `report-stock`'s authority is fixed here, named in your report.
- `finish`'s form as `b0`'s.
- If a `goal` call on the fake box's chat needs what the box lacks (a forge grant, a repository), adopt `widget` in the walk, as `open_default.rs` and its neighbours do.

## Steps

1. **The scripts,** the walk beside `open_default.rs`.
2. **A change of meaning:** the model's document grows, so run `heavy cargo nextest run -p temper-fake`, and the walk, in the process tier's binary: `heavy cargo nextest run -p temper --test process -E 'test(goal)'`; check the count, since the filter must select it.
3. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, with the walk's seconds.
