+++
id = "connection-close"
part = "connection-worlds"
kind = "feature"
needs = ["connection-residue"]
files = ["skein/tests/forge/machine/tests/suite/**", "skein/tests/forge/machine/src/hand.rs", "skein/testing/skein-forge-kit/src/fixtures.rs", "skein/tests/forge/protocol/src/github.rs", "skein/docs/development/debt/D030-forge-connection-harness-tests.md", "skein/docs/design/forge-connection.status.md"]
effort = "high"
minutes = 30
+++

# Forge-connection's copies go, and D030 goes

## Goal

Skein D030's **Paid when** holds and its copies leave. The move kept every test; here two tests telling one story become one, and two fixtures that say one thing become one.

Acceptance:

- **Duplicates go, by reading.** For each test name in `tests/suite/hand/` with a match in `tests/suite/` (start from `second_installation_settings_...` and `another_installation_proceeds_...`, close duplicates, and look at the survey's 48 names by `rg`), if one of the two holds every assert of the other, the other goes. If each holds something the other doesn't, both stay. Expect 15 to 30. The hand-side test usually asserts more: `start_backoff_doubles_to_its_maximum` also checks `Start::Waiting`, and `redelivery_is_skipped_...` also checks `kept`. The report lists each pair and what it kept.
- **Fixtures.** A helper in `hand.rs` and the kit's `fixtures.rs` with the same body goes from `hand.rs`, about 100 lines. `response()` and `github_settings()` differ, as `temper/docs/plan/debt-24/notes/hand.md` says: both stay. `skein/tests/forge/protocol/src/github.rs` shares about 67 lines with `joint.rs`, and `joint/github.rs` about 31: a line that's truly the same goes to the one the other reads, if it takes a line or two to call. Otherwise it stays, and you say so.
- `rg -lw Harness skein/crates/skein-forge-connection/src/tests | wc -l` reads 0. `rg -l Harness` there reads only the files on `ConnectionHarness` or `HookHarness`.
- `skein/docs/development/debt/D030-forge-connection-harness-tests.md` is deleted. Its **Not yet** entry goes from `skein/docs/design/forge-connection.status.md` (10). `python3 -I -B tools/citations/citations.py --all` reads 0, and `python3 -I -B tools/debt/debt.py | rg 'D030'` shows no skein row.
- `heavy cargo nextest run -p skein-forge-connection --lib -p skein-forge-machine-world -p skein-forge-protocol-world -p skein-forge-kit` passes.
- **Lines.** The report gives test lines in `skein-forge-connection/src` and `skein/tests/forge/machine` (with the kit's share) at this plan's base and now. If they haven't fallen by at least 300, it says what's left and why.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/development/debt/D030-forge-connection-harness-tests.md`

## Owns and may change

- **Owns:** the files in the header. This is the task that cuts a test: only a pair where one holds all the other's asserts.
- **A helper's copy elsewhere** (`skein/tests/fake-peers/tests/forge/fixtures.rs`, 23 lines; temper's `protocol-forge` fixtures, 35) isn't yours: report it as left.

## Steps

The pairs; the fixtures; the checks; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-machine-world -p skein-forge-kit --all-targets -- -D warnings`. Commit and push.

## Report

The common note's items; the pairs; the lines. The item's **Paid when**, in a sentence each.

## Answers

### 2026-11-05: resume connection-close after forge-sockets supplies its shared Sockets implementation (recommended), or explicitly defer D030 closure to a later task? (debt-24/connection-close-1, foreman debt-24/connection-close/foreman-1)

Resume connection-close after forge-sockets supplies its shared Sockets implementation; D030 closure remains required.
The task declares `needs = ["forge-sockets"]`.
Its goal says: “Skein D030's **Paid when** holds and its copies leave.”
D030 requires “one io harness for the forge worlds.”
`skein/docs/design/fake-forge.md`, 10.6 specifies that the worlds “play their wire through one io harness (`skein_forge_kit::sockets`, testing.md, 6)”.
Acceptance requires D030’s deletion and removal of both named **Not yet** entries; it provides no option to defer closure to another task.
The checkout has neither `sockets.rs` nor `pub mod sockets`, confirming the missing prerequisite.
The next run resumes from the pushed branch after that prerequisite is supplied, verifies **Paid when**, removes D030 and the named status entries, completes the required checks, and commits and pushes.

### 2026-11-05: forge-sockets ended at +131 net lines against -100; what now for this task? (fabrig/debt, the plan's owner, to debt-24/forge-sockets-2)

This replaces the answer above. `Sockets` is dropped, `forge-sockets` has left the plan, and this task no longer waits on it. Don't merge or read `debt-24/forge-sockets`. Each forge world keeps its own arms. D030's **Paid when** no longer asks for one io harness, and `skein/docs/design/fake-forge.md`, 10.6 and its status entry already say so on the trunk, so `fake-forge.status.md` isn't yours. You carry D030's other clause: the copies go, as above, and the item goes.
