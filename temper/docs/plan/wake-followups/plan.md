# wake-followups: what per-wake's plans and the first replays left

The request's seven items (`inputs/request.md`), ranked, after the plans `per-wake` and
`per-wake-temper` landed `temper/docs/design/draft/per-wake.md`'s jig and temper halves.
The standing rules are `notes/common.md`; the shapes that cross tasks are
`notes/contracts.md`. The design is the cut's, pushed with the second batch, each section
with its status's **Not yet**, one bullet per task that closes it:

- item 2: jig's `jig/docs/design/domain/engine.md`, 17.9 (the `stored_policy` facts);
  `temper/docs/design/domain/root.md`, section 10; the draft's status lag 8 closed;
- item 4: `temper/docs/design/protocol/configuration.md`, 13.9, the cut;
- item 5: `temper/docs/design/protocol/llm.md`, section 4;
- item 6: `smith/docs/design/domain/run.md`, 5.2;
- item 7: jig's `jig/docs/design/domain/tasks.md`, section 3, and `engine.md`, 17.11.

## Parts and tasks

Six tasks, within the request's seven; item 3 is behind a question (below).

| Part | Task | Needs | What | Request item |
|---|---|---|---|---|
| catalogue | `catalogue` | | the engine's domain world reopens its store with the catalogue it was last given; the per-wake story's ignore lifted (landed on the trunk) | 1 |
| size-refusal | `size-refusal` | | `SettingMember::StoredPolicy` across jig's core, client wire 19 and people store, in place; temper's form, ctl and JSON words; a row in ctl's refusal table | 2 |
| subscribe | `subscribe` | | charter 5's cut no longer subscribes to its changes' landings, which the core does (forge.md, 7.2); the figure stays | 4 |
| task-read | `task-read` | | temper's words for a task read refused as unknown say what to read instead; jig unchanged | 5 |
| busy | `busy` | | smith's relay asks a busy host again after a backoff doubling from `host_backoff` to 64 times it | 6 |
| foreman | `foreman` | size-refusal | jig's charter resolution keeps the effort a charter selects, so a foreman given no choice runs at its charter's medium | 7 |

What the cut decided, beside the request's words:

- **Item 4:** the design holds the charter at fault. The core subscribes a goal to the
  landings on its changes' branches (`forge.md`, 7.2), so subscription 4 was the core's,
  the goal's call a duplicate, and "(existing subscription 4)" not a path count. The bound
  is a fixture's figure no design sizes, so it stays; the refusal already names the bound
  and limit as `llm.md`, section 4 asks.
- **Item 5:** the read worked as designed (jig's `engine.md`, 7.3: an unavailable terminal
  report is unknown); the words change, in temper alone.
- **Item 6:** what retried without visible backoff is smith's relay, at a fixed 50 ms within
  the caller's deadline. The root's half (a fresh read refused Busy answers the host's Busy)
  is held already by the engine's domain world; the failing test is smith's, the lowest tier.
  A second, untimed retry exists in the forge connector's own reads (a brief's and a lost
  read's retry on each reclaim, `temper-engine-domain-forge/src/domain.rs`), not the path the
  run met: the report lists it, unplanned.
- **Item 7:** the fault is jig's resolution, which drops a charter's selected effort that jig's
  `tasks.md`, section 3 promises; fixing it there gives the foreman its charter's effort with
  `llm: None`, as temper's design has it, and no application root choosing an LLM.

## Question

- Item 3, `too-large-facts` in `plan.toml`: carrying a too-large read's bound and measure
  needs skein's `api::Error::TooLarge` to change and temper's store-v11 call record to keep
  them, a public interface and a stored format; the request makes any such change beyond
  item 2's a question first. No task waits for it.

## Critical path

`size-refusal` (200 min), then `foreman` (70): about 270 implementer-minutes, `foreman`
waiting only because both write `jig-domain-core`. `subscribe` (60), `task-read` (50) and
`busy` (90) run beside `size-refusal`.

## Overlaps with drafts beside this one

- `tui-approval` (now on main) listed `jig/crates/jig-client-protocol-wire/**`,
  `jig/crates/jig-client-protocol-link/**` and `jig/crates/jig-protocol-engine-clients/src/**`:
  `size-refusal` edits the client wire 19's schema and regenerates its code, and the two
  translations. Unavoidable, as the answered question puts the member on that wire; the
  trunk meets main's version of them when it lands.
- `ui-stop-coding-row/jig-stop-interrupts` lists `jig/docs/design/domain/engine.status.md`:
  this plan's design adds two **Not yet** bullets beside 17.12's, and `size-refusal` and
  `foreman` delete them. Unavoidable: they are that document's own gaps. Its code
  (`jig-domain-core/src/routing/**`, `jig/tests/core/tests/suite/{restart,stopping}.rs`) is
  apart from this plan's files.
- `temper-engine-smith` is written by `task-read` (`answers.rs`, `tests.rs`) while
  `size-refusal` adds one compiler-forced arm to its `settings.rs`; each spec says so.
