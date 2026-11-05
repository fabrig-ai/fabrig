# debt-18: the eighteenth debt round

Two independent parts (`inputs/request.md`): the spot review of debt-14's smith part, and four items that exist to remove
a copy.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| review 1 | `smith-review` | `effort-pin` | | high | 25 | a step test on the effort rule, a no-newline case; the seam goes |
| review 2 | `smith-review` | `session-fake-strict` | | high | 40 | the fakes panic on what no fixture sends; `feedback_text` goes |
| review 3, 4 | `smith-review` | `local-residue` | `effort-pin` | medium | 35 | notices read by type; residue goes; a row for the clips |
| review 5, 6 | `smith-review` | `terminal-words` | `local-residue` | high | 50 | the person's words move; one UTF-8 helper; forwarders go |
| skein D031 | `forge-setup-world` | `setup-one-world` | | high | 150 | 77 tests move onto `HostWorld`; `World`, `exchange.rs` and `decode.rs` go |
| skein D032 | `protocol-kit` | `protocol-kit` | | high | 65 | `skein-protocol-kit`; every http world on it |
| skein D032 | `protocol-kit` | `kit-json` | `protocol-kit` | medium | 20 | json's copies go |
| skein D032 | `protocol-kit` | `kit-tls` | `protocol-kit` | medium | 20 | tls's copies go; its end check keeps its one difference |
| skein D032 | `protocol-kit` | `kit-protocol` | `protocol-kit`, `kit-json`, `kit-tls` | medium | 15 | the last tally; the item rewritten to tests/io |
| skein D035 | `codegen-once` | `names-once` | | medium | 20 | `snake`, `pascal`, `rust_type` once |
| skein D035 | `codegen-once` | `diff-on-plan` | | high | 120 | `plan` exposes its matching and list; `diff` spells them |
| skein D035 | `codegen-once` | `emit-structure` | `names-once` | high | 90 | a splitting function is a value; no edit of emitted text |
| skein D035 | `codegen-once` | `codegen-close` | `names-once`, `diff-on-plan`, `emit-structure` | medium | 5 | checks **Paid when**; deletes the item |
| jig D012 | `tasks-operation` | `cancel-release` | | high | 110 | `Cancel`, `Release`; `Control` and `ProcedureCancel` go |
| jig D012 | `tasks-operation` | `amend-move` | `cancel-release` | high | 80 | `Amend`, `Move`; `ProcedureAmendment` goes |
| jig D012 | `tasks-operation` | `message-type` | `amend-move` | high | 55 | `Message`; `ProcedureMessage` goes; the item is deleted |

## Critical path

`cancel-release`, `amend-move`, `message-type`, 245 minutes. Then `setup-one-world`, 150 alone; `diff-on-plan`, `codegen-close`, 125;
`effort-pin`, `local-residue`, `terminal-words`, 110; `names-once`, `emit-structure`, `codegen-close`, 115; `protocol-kit`, `kit-tls` or `kit-json`,
`kit-protocol`, 100. None of the five parts waits on another.

## Decisions: the review, checked against the code

Every finding was read in the code at this plan's base. They hold, but for these:

- **Finding 3, "one `Notice` arm":** merged carelessly, it changes what the local world's referee sees, since only the
  second arm reaches `self.notices` and `self.saw`. The spec guards that.
- **Finding 4, the clips:** of "drop `cap`, or add a row", a row. Dropping `cap` makes a notice copy a whole segment.
- **Finding 4, `const ALL` and `exited`:** the copy is in `smith/tests/tools/tests/suite/simulation.rs`, another test crate.
  They meet in `smith_tools_world::fixture`, which `stories.rs` already reads.
- **Finding 6, the status counts: skipped, already right.** `testing.status.md` and `STATUS.md` say three domain worlds call
  protocol crates, the review's own count; debt-14's retire corrected them after the merge the review read. `tests/local`
  also renders with `smith-local-protocol`, but `smith/docs/design/testing.md`, 2 makes it a system world.
- **Finding 6, the words:** the request names `service.rs` and `headless.rs`, and that's the scope. `service/accounts.rs`,
  `service/choices.rs`, `front/interactive.rs` and `front/remote.rs` hold more of a person's words: `terminal-words` reports
  them as left, and they're worth a smith item if fabrig/debt wants them.

## Decisions: D031

Re-measured: `World` has 77 tests. The survey's net is about 480 to 540 lines out, not the item's 766: `HostWorld` must gain a
request log, a refusal, hook seeding and the plans' entrances. One task, since the fixtures take `&mut World` and moving
stories piecemeal would mean two copies of the helpers.

- **Four `draws` assertions move to step tests; this cut decided it.** The host draws inside itself, and its `Need` is `Read`
  or `Write` (`skein/docs/design/forge-setup.md`, 6.3), so a story through the host can't see one, by the design this item
  asks the world to follow. A draw counter in the host just for a test would be worse. The rule isn't lost: step tests hold
  three of them, and the spec adds the fourth, with a bite. This is the request's "stop and ask" read narrowly: no event or
  outcome of the code changes, only the tier that pins a plan's rule. fabrig/debt can overrule it.
- **`secrets_absent` reads what the host hears,** the scope GitHub's stories already use. The spec adds a step test if none
  renders `Need::Send` and `Event::Reply` for a secret.
- **No user question.** Both are within the request.
- **The fake gains three additions** (a request log, a refusal, hook methods), which temper's readers don't see. The stories keep
  their spelling this way, and a fixed connection-index script would have been brittle.

## Decisions: D032

Re-measured: 16 `struct Seen`, as the item says, of which 7 are one record.

- **The tests/io clause stays open, and D032 stays: this plan pays the kit clause only.** The request asks every item's **Paid when** to
  hold, so this is a named shortfall, for fabrig/debt to accept or overrule. The stage can't meet "behaviour unchanged" by construction: the worlds
  fire deadlines with completions waiting, where `stage.rs` returns first, and they have no place in it for the per-call `MAX_OUT` check or
  the ledger's `request`, since `Request` isn't `Clone`. It needs a witness inside a step crate, or a panic path in shipped code: a decision,
  not a refactor. Moving the loop alone onto `skein_world` is about 30 lines out for 75 minutes, a `Referee` change and an `Rc`
  across the owner, so it isn't worth a slot in a round asked to stay small. `kit-protocol` rewrites the item to what's left, with both steps.
- **The item's measure can't read 0.** It reads 10 after: the kit's and nine different records under one word, one a public API of the browser
  kit (`skein/docs/design/browser.md`). The request exempts those. `struct Seen\((pub )?BTreeMap` tracks the copy, 7 to 1.
- **`below_acts` stays per world.** They differ in the order of their draws, so a shared one would change every seeded schedule.
  **Paid when** names only the four functions, and those move with the same Rng calls.
- **TLS keeps one difference in its end check.** For TLS `Read::Nothing` is met by any byte left; with http's form the assert could never
  fire. Its spec pins it, with a bite.
- **The kit is `skein-protocol-kit`:** `skein-protocol-world` is taken, and D028's gate is to flag a kit named `-world`.
- **`protocol-kit` may rise by up to 30 lines,** being the kit and one consumer. The part as a whole is about -130.

## Decisions: D035

Re-measured: as the item says, but for `plan.rs` and `diff/`'s line counts, which match. This cut wrote `skein/docs/design/codec.md`, 10 and 11.6,
and a **Not yet** that `diff-on-plan` closes.

- **`Op` stays as it is, against the survey's first design.** A new `Op::Rename` in `changes` would widen a public enum, add an arm no parser
  reaches, and grow the list's capacity. Renames are the plan's input, not a change it has to see covered, so `diff` places them itself, about 30
  lines more in ordinary Rust. No stop and ask follows.
- **11.6's additions are additions.** A `pub` on `Matching`, `Comparison`, `Change`'s fields; `RenameRefused`, split out of the plan's two
  rename functions, which only check, so a plan's `Twice` and `Mismatch` keep their precedence and spans.
- **One real disagreement stays.** `rename D D` is refused by a plan and accepted by `diff`. It's behaviour, and `codec.md`, 10.1 doesn't list it as a refusal.
- **"As structure" is read as:** nothing parses or rewrites text after an emitter writes it, and a function that may split is a value of
  statements. Statement text stays `format!`; no syntax tree. The module-wide `format_push_string` expects are in the item's **What**, not its **Paid
  when**: rewriting 96 sites adds length, so they stay. The split of `emit_tests.rs`'s template stays too, being a source template.
- **Drift tests are the proof of `emit-structure`,** across all four trees. `skein-codegen check` doesn't exist.
- **jig's mirrors request** (`jig/docs/development/debt/D007-contract-written-by-hand.md`) needs nothing here: this only unblocks it.

## Decisions: D012

Re-measured: `boundary.rs` is 2,263 lines. `ProcedureInput` spells no operation, so the item's naming of it is stale; the real copies are
`ProcedureAmendment`, `ProcedureCancel` and `ProcedureMessage`.

- **One type per operation, and `Operation` their sum.** The item's words, "one `Operation` carried by each entrance", don't fit: a tool control
  takes two of the six, a procedure three, an event one, and carrying the whole enum needs an arm per entrance for what nobody can send. **Paid when** says
  "has one type", which this meets. A compound of one isn't equivalent either: refusal order, the project check, `Party::Task`, capacity and
  answers all differ. `notes/contracts.md`, 2 lists what stays.
- **This changes public items, without a stop and ask: this cut decided it.** `jig_domain_tasks::Event`, `Operation`, `ProcedureDecision` and
  `jig_domain_core::NamedAction` change spelling. The crates are `publish = false`, the compiler finds every reader, and temper's seven lines change
  only in a name or type path. No stored or wire format has these types; `ProposalAction` and `people::Operation` do have one, and stay.
  The item can't be paid otherwise. fabrig/debt can overrule it.
- **Three steps, one chain.** Each compiles and is green alone, and they share files. The third may rise, by up to 40 lines: it removes little and nests a
  `Word`. All three must fall together (estimate: -45 to -155), and `message-type` stops and asks if they don't.

## Left out, by name

- **D032's tests/io clause,** above.
- **The other person's words in smith,** above.
- No temper client view or `temper.css` is touched, as the request asks.

## Overlaps

- **`temper/crates/temper-engine-domain/src/route/forge_route.rs`:** `cancel-release` changes two lines, near line 3912, in the 5,000-line file that
  `ui-welcome/setup-intent` also changes. It can't be avoided, and it's small.
- **`jig-domain-tasks/src/boundary.rs` and `domain.rs`:** `ui-goals/delegate-counts` adds a line to each, far from `Event` and its dispatch.
- **`Cargo.toml`, `Cargo.lock`:** the `protocol-kit` tasks and `setup-one-world` each add or drop a line.
- Nothing else: smith, skein's forge setup, skein's codec and the protocol worlds are in no draft beside this one.
