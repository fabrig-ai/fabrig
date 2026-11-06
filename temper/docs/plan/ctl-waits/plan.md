# ctl-waits: `temper ctl` says what it waited for, and a kept message gets its answer

The request: `inputs/request.md`. Three things, ranked, in four tasks, so nothing is dropped from the bottom.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `kept-answer` | `tasks/kept-answer.md` | `deadline-words` | Item 1: the wait reproduced at both sides and fixed where it lies; a retry's span; the lines naming a retry's why (`temper/docs/design/client/tools.md`, 6.3, 8.2, 12) |
| `settings-answer` | `tasks/settings-answer.md` | | Item 1's earlier walk: a settings change naming an unoffered effort gets its answer, or the report says it already does |
| `deadline-words` | `tasks/deadline-words.md` | | Item 2: the deadline's four lines, and what's true of an act (`temper/docs/design/client/tools.md`, 6.3) |
| `long-pass` | `tasks/long-pass.md` | | Item 3: one line for a pass past 1 s (`temper/docs/design/protocol/README.md`, 8.1) |

Critical path: `deadline-words`, then `kept-answer`. `long-pass` and `settings-answer` run beside them.

`kept-answer` waits for `deadline-words` although it ranks first: it adds rows to the same module and tests, and one writer holds
that crate at a time.

## What this plan decided

- **Item 1 is cut as a hypothesis, not a finding.** Reading alone, nothing run: the 23.2 s fits retry spans of 1, 2, 4, 8 and 16 s, each
  jittered to half or more, which put a sixth attempt at 15.5 to 31 s, midpoint 23.25 s. The next span then passes the deadline with
  nothing in flight, which is the state the investigation saw. Who answered *again* isn't known: a full inbox or the sign-in's two
  exchanges are the candidates. The task reproduces it first, and has a pre-approved fix for each thing it may find.
- **A retry's span goes from at most 30 s to at most 2 s, in the tools' preset alone,** as a sign-in's link already retries. This is
  no budget or timeout of the user's: the 30 s deadline stays, and the span only lowers. The terminal and the browser keep theirs.
- **The explicit pending result is 6.3's line:** exit 1, what the engine last answered, and that the act is kept. `kept-answer` adds
  three rows to `deadline-words`' table, which is compatible detail: its rows keep their words.
- **Item 3's bound is 1 s, one line in 10 s at most.** A thirtieth of ctl's deadline, and the walk's longest whole read under load
  was 1.95 s. Long passes inside the 10 s are held as the longest and a count, so a long stall never hides behind a short one.
- **Item 3 costs seven clock reads a pass when nothing's slow,** with no allocation and no write. `long-pass` measures a read.
- **Item 3's stages are temper's own,** so nothing generic goes to skein. A step crate takes no closure or clock (skein's programming
  model, 10), so `iterate` splits into stages and the binary's adapter reads the clock between them; a world has no watch.
- **Item 2 stays in `temper-client-shell-cli`,** with its one line of help. A sign-in's and a sign-out's line stay as they are.

## Not in this plan

- **A kept request the engine truly can't read:** any HTTP status but 200 is `Lost`, so a 400 retries until retention. Telling it apart
  is a protocol question for its owner, and only `settings-answer`'s person-writable case is answered here.
- **The bounds that may answer *busy*:** `exchanges_per_client` and an inbox's room stay. Changing one is a question.

## Overlaps

- `temper/docs/help/ctl.md` sits in `model-check/trial-close`'s glob, `temper/docs/help/**`. One line of Limits changes.
- `temper/crates/temper/src/host.rs`, `temper/crates/temper-service/src/lib.rs` and `protocol/README.status.md`: `landing-fixes-2/refusal-words` touched each and has merged.
- `temper/tests/client/tests/suite/main.rs`: one `mod` line, beside `long-words/composer-view`'s.
- `jig/crates/jig-protocol-engine-clients/src/translate/**` sits in `ui-proposal-card/proposal-facts`'s glob: `settings-answer` edits it only if it reproduces.
