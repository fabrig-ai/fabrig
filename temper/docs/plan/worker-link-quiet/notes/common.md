# worker-link-quiet: rules every task keeps

Every task of this plan reads this note first.

## What the plan is for

On a deployment (`temper/docs/plan/worker-link-quiet/inputs/request.md`),
one task's start made a remote worker's link go quiet each time it was
released: about 15 s later the worker lost the link by silence, the
engine saw the stream end, every run on the worker was lost, and after
eight losses the task held. Nothing crossed the link during the quiet,
pings included. The evidence is outside the repository: the request's
words are all there is.

The cut found by reading one cause that fits every symptom:

- skein's channel asks its stream for room for a whole frame
  (`skein/crates/skein-channel/src/machine.rs`, `Output::Free`), and
  skein-io leaves an ask past its output cap unanswered
  (`skein/crates/skein-io/src/stream.rs`, `OutputDown::Room`, `bytes >
  env.limits.output`; `joined/end.rs` the same in one process). The
  frame never drains, and nothing behind it goes either.
- While a frame is queued the engine's component arms no ping
  (`jig/crates/jig-protocol-engine-workers/src/connection.rs`,
  `next_deadline`): the engine falls silent, and the worker's silence
  deadline (15 s) drops the link. The stall deadline (30 s) would come
  later.
- The standard profile admits an assignment far past the 1 MiB
  `io.output`: a 256 KiB charter, a 256 KiB window, 32 committed calls of
  up to 132 KiB each, 82 inbox messages and the grants. A resumed
  producer carries the calls committed since its last turn.
- The fleet keeps a lost worker's runs for its grace (120 s); 120 s plus
  the 15 s silence is the report's 2 min 16 s cycle.

The other candidates are ruled out by reading, and the rows say so:
both sides refuse an opening whose terms leave a gap
(`jig/crates/jig-protocol-worker-wire/src/kinds.rs`, `terms_gap`), so a
frame past the peer's terms is a broken contract; the worker reads its
link on every pass (the `worker-relinks` plan); a frame past the
worker's terms arriving is a framing loss, named.

## Reading

- fabrig's `AGENTS.md` and the tree's `AGENTS.md`.
- This plan's design, never a whole document:
  `jig/docs/design/protocol/channel.md`, 5.2 (*A start the link cannot
  carry ends there too*), 9 (*Every frame fits the stream below*, *A
  start past the room fails its run, never the link*), 11 (*The
  component's own limits*, *The room is the stream's, not the
  domains'*), 12 (*A frame past the room, in the machine worlds*);
  `temper/docs/design/domain/worker.md`, 4.3 (*A start the link cannot
  carry*); `temper/docs/design/protocol/README.md`, 15.3 (the `worker
  unsent` row).
- skein's rule, for the why: `skein/docs/design/lib.md`, 7.1 (a Room's
  bytes fit the lower's cap) and `skein/docs/design/channel.md`, 7.
- `temper/docs/plan/worker-link-quiet/notes/contracts.md`: the shapes
  that cross tasks. They bind.

## Rules

- **Failing test first.** A task that fixes a fault commits the row
  that shows it, failing on the trunk for the fault's reason, alone,
  before the commit that fixes it. The report quotes the failure seen.
  A row that passes on the trunk rules its candidate out and stays.
- **A new case of one behaviour is a row** of an existing table test, or
  of the one table your spec names.
- **Tests stay cheap:** machine worlds and step tests on a simulated
  clock; no test reaches a real host or network; no sleep.
- **Only the focused and fuzzy suites run,** through `heavy`, one
  command at a time, narrowest first (`AGENTS.md`, 5). Never
  `tools/tiers/run`, a `<tier>-<tree>` profile, `scripts/browser.sh`, a
  walk, a render check or a throwaway driver.
- **No figure.** No limit, bound, deadline, grace or default changes in
  `jig/crates/jig-protocol-config/src/profile/*.rs`, temper's profiles
  (`temper/crates/temper-service/src/limits.rs`) or the stock settings.
  A fix changes what happens past a bound, never the bound. A world's
  own fixture may give its scripted stream a room for a row; that is the
  row's input, not a figure.
- **No stored format or protocol change:** no wire or store schema,
  golden or codec changes. The refusals are local answers in the
  vocabulary that exists (`jig_domain_host::Ending::Refused`,
  `JobEnding::Refused`).
- **Trees:** skein names none of the others; jig never names temper. No
  skein change is planned; needing one is a `Question:`.
- **Cross-tree lines.** A jig task that adds a field or a variant
  temper reads carries temper's compiler-forced lines only, each edited
  in the field's name, type path or value, or an arm that does nothing;
  `temper-room` writes what temper does with them.
- **Status lines:** the design is written. A task removes only the
  **Not yet** entry its spec names, its lines alone. That departs from
  the implementer's text, which closes `docs/design/` to you: the entry
  is your task's.
- **Not yours:** any other `docs/`, drafts, `docs/help/` and this plan's
  files.
- **Old forms:** `python3 -I -B <tree>/scripts/old_forms.py` for each
  tree you touch, before the push; `--write` only to lower a count.
- **Citations:** after committing, from the repository root,
  `python3 -I -B tools/citations/citations.py --base origin/main`. It
  reads HEAD, not the working tree.
- **A run's failure your change can't reach** runs once alone on the
  pristine base before you call it outside your change.

## Packages

| Package | Path | Its tests here |
|---|---|---|
| `jig-protocol-engine-workers` | `jig/crates/jig-protocol-engine-workers` | `--lib` |
| `jig-engine-workers-world` | `jig/tests/engine-workers` | `--test suite` |
| `jig-protocol-worker-link` | `jig/crates/jig-protocol-worker-link` | `--lib` |
| `jig-worker-link-world` | `jig/tests/worker-link` | `--test suite` |
| `jig-test-service` | `jig/testing/jig-test-service` | `--lib` |
| `jig-channel-world` | `jig/tests/channel` | `--test suite` |
| `temper-service` | `temper/crates/temper-service` | `--lib` |
| `temper-engine-shell` | `temper/crates/temper-engine-shell` | `--lib` |

Check each name with `cargo metadata --no-deps --format-version 1`
before you run it; a package named otherwise is the one at that path.

## Who owns what

| Task | Owns |
|---|---|
| `engine-room` | `jig/crates/jig-protocol-engine-workers/**`, `jig/tests/engine-workers/**`, the joined world's files `jig/tests/worker-link/src/joined.rs` and `jig/tests/worker-link/tests/suite/joined.rs` |
| `worker-room` | `jig/crates/jig-protocol-worker-link/**`, `jig/tests/worker-link/**` but the two joined files |
| `temper-room` | `temper/crates/temper-service/src/tests/link_room.rs`, the `Note` and its routing in `temper-service`, `temper/crates/temper-engine-shell/src/serve/host.rs` |

A file another task owns is changed only as your spec's *Owns and may
change* allows, and the report names each such edit.
