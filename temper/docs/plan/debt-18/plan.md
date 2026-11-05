# debt-18: the eighteenth debt round

Two independent parts (`inputs/request.md`): the spot review of debt-14's smith part, and four items that exist to remove
a copy. This file grows with each release.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| review 1 | `smith-review` | `effort-pin` | | high | 25 | a step test on the effort rule, a no-newline case; the seam goes |
| review 2 | `smith-review` | `session-fake-strict` | | high | 40 | the fakes panic on what no fixture sends; `feedback_text` goes |
| review 3, 4 | `smith-review` | `local-residue` | `effort-pin` | medium | 35 | notices read by type; residue goes; a row for the clips |
| review 5, 6 | `smith-review` | `terminal-words` | `local-residue` | high | 50 | the person's words move; one UTF-8 helper; forwarders go |
| skein D031 | `forge-setup-world` | `setup-one-world` | | high | 150 | 77 tests move onto `HostWorld`; `World`, `exchange.rs` and `decode.rs` go |

## Critical path

So far: `setup-one-world`, 150 minutes, alone. `effort-pin`, `local-residue`, `terminal-words` is 110.

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

## Overlaps

None so far: no draft beside this one touches smith or skein's forge setup.
