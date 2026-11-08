# walks-lower: rules every task follows

- **Read first:** fabrig's `AGENTS.md`, `temper/AGENTS.md`, your spec, and
  `temper/docs/plan/walks-lower/inputs/request.md`, read whole (it is
  short): its rulings say what each walk's lower story must keep, and its
  `For every task of this plan` list binds every task. The design your
  story realises is `temper/docs/design/domain/testing.md`, 2.6, the
  bullet **The fake box keeps two walks**, and the world's own section
  your spec names.
- **What decides a test's place is what it checks, never its cost**
  (`skein/docs/foundation/testing-strategy.md`, 8.1). A lower story holds
  the walk's behaviour with the lower world's real parts: it never stubs
  what the walk proves. Where your world can't hold a keep without a stub,
  stop that keep, write the rest, and say so in the report under `##
  Left` with what the world lacks: that is no failure, and the walk stays
  for that keep.
- **A quiet wait doesn't move as a wait.** A walk's sitting, poll pace or
  hold is the lower world's clock, advanced by the story (`Time`,
  `Duration`, the world's `after`), never a sleep and never a real-time
  wait.
- **Failing first, where it can.** A story that guards a bug a walk found
  is shown red with the fix reverted by hand: revert the lines your spec
  names, run the story alone, see it fail for that reason, restore, see
  it pass. Every other new story bites once (the implementer's text): the
  report names the rule broken and the failure seen.
- **Only focused and fuzzy run.** The lower worlds here are in the
  focused suite: `temper-client-world` (`temper/tests/client`, the joint
  world), `temper-real-loop-world` (`temper/tests/real-loop`, test group
  `temper-real-loop`), `temper-one-process-world`
  (`temper/tests/one-process`), and the `temper` package's `suite`
  binary. Never `tools/tiers/run`, a `<tier>-<tree>`
  profile, a browser, process or live tier, or `scripts/browser.sh`; the
  `process` binary of `temper` is compiled
  (`heavy cargo check -p temper --tests`), never run. No walk, render
  check, screenshot or throwaway driver: the plan is complete when it
  lands, and after-landing reads the process tier's figures.
- **No walk leaves in a story task.** The walks under
  `temper/crates/temper/tests/fake_backends/` are `walks-go`'s alone,
  once every lower story has merged. A story task never edits them.
- **Tests stay cheap.** A new story extends a sibling's setup in the same
  file where one fits; helpers stay beside the story in its suite file. A
  harness change under `temper/tests/client/src/` or
  `temper/tests/real-loop/src/` is a constructor or helper the story
  can't do without, small and named in the report: another draft
  (`ui-ask-m3a/ways-stories`) also edits `temper/tests/client/src/**`, so
  add, never reshape.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once on the pristine base. If the base passes and your branch fails,
  it is yours.
- **Decide, don't ask.** A small edit just outside your files (a `mod`
  line, a re-export, a doc comment) is made and named in the report. A
  `Question:` is only for what truly blocks your acceptance.
- **Design files are the cut's.** A story task touches no
  `docs/design/` file: `walks-go` alone edits
  `temper/docs/design/domain/testing.status.md`, as its spec says, once
  every story has merged, so that parallel tasks never meet in it.
  `temper/docs/help/` is not touched.
- **Not to touch:** `temper/crates/temper-client-*` (the request keeps
  this plan out of them; a product bug a story finds is a `## Left`
  entry, never a fix here), the walks, and every other task's owned
  files below.

| Task | Owns |
|---|---|
| `status-stories` | `temper/tests/client/tests/suite/status.rs`; appends to `temper/tests/client/src/front/status.rs` |
| `away-story` | `temper/tests/client/tests/suite/away.rs`; appends to `temper/tests/client/src/front/away.rs` |
| `goal-story` | `temper/tests/client/tests/suite/task.rs` |
| `model-story` | `temper/tests/client/tests/suite/connect_model.rs` |
| `issuer-log` | `temper/crates/temper/tests/suite/fake.rs` |
| `issuer-sockets` | `temper/tests/real-loop/tests/suite/sign_in.rs`, `temper/tests/real-loop/src/{issuer,world,calls}.rs` |
| `note-parts` | `temper/tests/real-loop/tests/suite/notes.rs` (new), its `mod` line, `temper/tests/real-loop/src/fixtures.rs` |
| `chat-document` | its story in `temper/tests/real-loop/tests/suite/notes.rs`, after `note-parts` |
| `copy-ctl` | `temper/tests/one-process/src/copy.rs`, `temper/tests/one-process/tests/suite/copy.rs` |
| `copy-ready` | `temper/tests/one-process/src/ready_copies.rs` and `tests/suite/ready_copies.rs` (new), their `mod` lines, after `copy-ctl` |
| `walks-go` | `temper/crates/temper/tests/fake_backends/**`, `.config/nextest.toml`'s process-temper lines, and the status entries on the walks, after every other task |
