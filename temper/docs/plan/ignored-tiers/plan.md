# ignored-tiers: the plan

Twelve of temper's ignored tests were run by hand (`inputs/request.md`).
This plan gives the ones that need no person a tier, by what each
checks (`skein/docs/foundation/testing-strategy.md`, 8.1). The ones that
need a person say in their `#[ignore]` reason what the person gives, and
`temper/docs/design/domain/testing.md`, 2.7 lists them once. Three
tasks. No test of a slow tier is run by the plan: the process-tier
members are compiled and listed, and after landing runs them.

The cut wrote the design first:

- `temper/docs/design/domain/testing.md`, 2.6 (*Two drivers' offline
  proofs*), 2.7 (B0's offline proofs; *What needs a person*, which
  replaces *What stays the owner's*), and 7 (two layout lines).
- `temper/docs/design/domain/testing.status.md`: two **Not yet** entries
  the tasks close. The pair's old reasons clause moved into the first.
- `temper/docs/design/playground.md`, 9 (*The native heap at start, in
  the gate*), and a **Not yet** entry in `playground.status.md`, 9.

## Parts and tasks

| Part | Task | What | Needs | Effort | Minutes |
|---|---|---|---|---|---|
| `b0-offline` | `b0-offline` | `proof_live`'s two lose `#[ignore]`; `.config/nextest.toml`'s selection contract for both process members; `proof-live.sh rung-b0-offline` under `process-temper`, held by a new launcher stub test; the five live reasons | | medium | 50 |
| `world-checks` | `world-checks` | the playground memory test bound at figure + 25%; the one-process memory test bound if story A walks, else its reason names why not; the hint profile asserts its counts; `playground-memory.md` | | medium | 70 |
| `managed-sitting` | `managed-sitting` | `managed_ready_client` sets up the managed sitting's fake half and drives Ready through sign-in and adoption; the driver still runs it as its client; `managed_relay`'s reason | `b0-offline` | high | 180 |

`notes/common.md` holds the rules every task keeps, and the selection
contract.

## Decisions

- **Where each goes.** `proof_live`'s two and `managed_ready_client`
  start the binary and its processes: they go to `process-temper`, never
  `live-temper`, since none needs a real model. The two memory binaries
  and `hints.rs` already sit in binaries the focused profile selects, so
  dropping `#[ignore]` puts them in the focused suite.
- **By name, not by binary.** `proof_live` also carries
  `support::forge_land`'s four unit tests, and `live_github_managed`
  carries `relay_managed`'s six. Both sets run in the focused suite now.
  Taking whole binaries into `process-temper` would move them out of it.
  So the profiles select and exclude the new members by name
  (`notes/common.md`).
- **One writer for `.config/nextest.toml`:** `b0-offline` writes both
  members' selection. `managed-sitting` needs it for its list check,
  and it is short.
- **The script keeps working.** `proof-live.sh rung-b0-offline` runs the
  two under `process-temper` by its default filter. The managed driver
  still runs `managed_ready_client` as its client, its mode in
  `TEMPER_MANAGED_CLIENT`. Without that variable, the test sets up the
  fake sitting itself. The driver's `ready_client` drops `--ignored`,
  which would otherwise select nothing and pass silently.
- **The managed test's scope:** the driver's fake mode through the
  adoption, then its secrets scan. The push, the planted refusals, the
  cleanup and `fake-wide`'s refusal stay the driver's, since they test
  the relay's guard, not Ready. Those stay the owner's dry run.
- **The bounds:** `playground-memory.md` gives no margin, so each
  bound is its figure plus a quarter (the request). If the trunk has
  already grown past one, the task records the trunk's figures, binds
  them, and reports the growth as a finding, so it never loosens the margin.
- **Story A's one-process high water:** the doc records none, because
  the story stalls there (`playground.status.md`, 9). The task checks
  once whether it walks now. If it doesn't (expected), that test keeps
  `#[ignore]`, its reason naming why, and this one of the request's six
  stays unbound. If it walks, the task measures it, records it and binds it.
- **The hint profile** asserts each hint's wakes, that its steps send
  only the woken changes' fresh reads, and that the second landing sends
  no more requests than the first (forge.md, 5.2).
- **The six that need a person** (testing.md, 2.7):
  - `a_real_model_answers_a_chat_and_pushes_a_small_change` needs a
    fresh ChatGPT sign-in. It could run on a held login, but the landing
    pair already runs its story unattended and further, so that would
    spend a real model's run on no new check.
  - `rung_b0_...` needs a held account on the rung's own sol. Its bounds
    are that model's walk, so the cheapest model would prove another.
  - The landing pair already runs unattended in `live-temper` on held
    logins, the tier's inputs. It stays ignored (the tier runs
    `--run-ignored only`), and its reasons now name the held login.
  - `live_github`'s test needs github.com and the probe App's fixtures.
  - `managed_relay` needs github.com, where a person makes the App.

  None is made unattended.
- **Effort:** the request asks medium. `managed-sitting` is high, since
  it writes a new process story of a few hundred lines that the plan
  cannot run.

## Critical path

`b0-offline` (50) then `managed-sitting` (180), about 230 minutes.
`world-checks` (70) runs beside them.

## Risks after landing

- `rung_b0s_walk_passes_offline_on_its_twin` walks for minutes with
  real Forgejo and its runner (its own bound is 700 s). Its CPU against
  `process-temper`'s per-test and suite budgets is unknown. If it is
  red there, the repair asks 8.1's questions. It belongs to the tier by
  what it checks.
- `managed_ready_client` is a new process story that no one has run.
  Its first run, and the owner's next `fake` dry run, are after landing.

## Overlaps with other drafts

- `ui-p1-rail/readers` changes `temper/tests/one-process/src/**` and
  `crates/temper/tests/process.rs`. `world-checks` owns only
  `tests/one-process/tests/memory.rs`, and no task touches `process.rs`.
- `ui-reaches/decide-root` changes
  `temper/tests/engine/domain/tests/suite/forge_routes/decided.rs` and the
  world's `Cargo.toml`. `world-checks` owns only `forge_routes/hints.rs`.
- `b0-offline` and `managed-sitting` both edit
  `temper/docs/design/domain/testing.status.md`, each its own bullet at
  the file's end and, for `managed-sitting`, one Departs entry's opening
  words. `managed-sitting` starts after `b0-offline` merges, so neither
  edits the other's lines.
