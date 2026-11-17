# A landing red only from known flakes gates once more

Draft, 2026-11-16, on the user's word. On 11-16, a landing's gate failed only on a known fuzzy flake. Its two changes then went round a loop: a landing repair, "ended without making the required change", a hold, and after each release another repair. They never tried another landing. Even a fresh landing would have failed at once, since the failed gate receipt stands while the tree is unchanged. The workaround was a repair that merged main, to make a new tree (P67).

## 1. What happens

- A landing repair that changes nothing is turned into a failure, "ended without making the required change" (`delegate_result`, `temper/crates/temper-engine-domain-forge/src/domain.rs`). Main treats an empty CI repair specially, but excludes landing repairs.
- A release then resumes `Repairing`, and `repairing` asks a fresh repair with the same reason, as `temper/docs/design/domain/forge/change/repairs.md`, *A release*, says (`temper/crates/temper-engine-domain-forge-change/src/domain.rs`).
- Receipts are shared by gate and content key, so an existing failed receipt makes the next landing of the same content fail at once (`temper/docs/design/domain/landing.md`; `temper/crates/temper-engine-domain-forge/src/landing.rs`).
- `temper/docs/design/domain/findings.md` says a landing whose every failing test matches an open flake finding gates once more. `landing.status.md` lists that as not yet built.

## 2. Wanted

- **The designed rerun, built:** before a landing publishes a failed receipt, `failed_jobs` and `failed_receipt` check every failing test against the open flake findings. If all match, the landing gates once more, with the same content and a fresh attempt. Only that attempt's verdict makes the receipt. A second red is published as usual. The rerun is durable across a restart and happens once per landing.
- **A repair can say "flake":** a landing repair that finds nothing to change, because every failure is a known or newly seen flake, ends with a typed verdict saying so, rather than a failure.
- **A release after that verdict re-gates once:** the change supersedes the failed receipt with a fresh gate attempt, durably named, rather than asking another repair. Mixed or unknown failures keep today's repair behaviour.
- **Bounds stay:** a repeated red holds as before, and nothing needs a dummy commit.
- Revise `repairs.md` (the repair's result and *A release*), `landing.md` (receipts and supersession) and `temper/docs/design/domain/forge/README.md` where it describes receipts, with their status files.

## 3. Out

- How flakes are found and recorded: the existing findings.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- In the forge domain's change tests (`temper/crates/temper-engine-domain-forge/src/tests/change.rs`) and the empty-repair tests (`temper/crates/temper-engine-domain-forge-change/src/tests/empty_repair.rs`):
  1. A red receipt whose failures all match open flake findings gates once more; green proceeds.
  2. A no-change landing repair ending as "flake", then a release, gives one fresh gate attempt across a restart; green proceeds with no new commit.
  3. A repeated red stays bounded.
  4. Mixed or unknown failures still repair.
- A landing world story: the loop seen on 11-16 ends in a landing.

Run only the new and affected tests, by name.
