+++
id = "dismissed-story"
part = "client"
kind = "feature"
needs = ["dismissed-read", "dismissed-facts"]
files = ["temper/tests/client/tests/suite/dismissed.rs", "temper/tests/client/tests/suite/main.rs"]
effort = "high"
minutes = 40
+++

# The story: every client of a person hears a dismissal

## Goal

The plan's meeting point, in the world that joins temper's client to its real engine root: a person's
one client sends `Dismiss`, and their other client's facts hold it once it commits, across a restart.
It proves the two real sides meet; the rules are held lower down.

Acceptance, in a new `temper/tests/client/tests/suite/dismissed.rs`:

- `a_dismissal_reaches_every_client_of_the_person`: two clients of one person (the second client as
  `chat.rs`'s `INBOX_CLIENT`), and a client of another person in the same project. Before it
  `Side::dismissed(ToBegin { project })` reads `Some(false)` on all three; the first sends
  `dismissed::ask`, is answered dismissed; then the two read `Some(true)`, the third `Some(false)`, and
  the same dismissal on another project of theirs stays `Some(false)`.
- `a_second_dismissal_is_answered_as_the_first`: from the other client, under its own key, answered
  dismissed, the store's rows of the family unchanged.
- `a_dismissal_outlives_a_restart`: the engine restarted as `agent_recovery.rs` does, a client's facts
  hold it again.
- `a_party_with_no_role_is_refused`: no row, and jig's refusal words reach the client's pending.
- `heavy cargo nextest run -p temper-client-world --test suite -E 'test(/dismissed::/)'` passes, the
  count checked, then `heavy cargo nextest run -p temper-client-world`.

## Inputs

- `temper/docs/plan/to-begin-store/notes/common.md`; `notes/contracts.md`, K6.
- `temper/docs/design/draft/to-begin.md`, 5.
- `temper/docs/design/domain/testing.md`, 2.1 (`temper-client-world`).
- The patterns: `temper/tests/client/tests/suite/chat.rs` (`INBOX_CLIENT`),
  `agent_recovery.rs`, `adoption.rs` (a family request and its answer).

## Owns and may change

- The files of the header. `main.rs`: the `mod` line.
- `temper/tests/client/src/**`: a fixture a test needs (a second party in a project, a second
  project), added and not changed. Nothing in `temper/crates/**`.
- Status: remove "4.6, 8: `Dismiss` and the subject `dismissed`..." from
  `temper/docs/design/client/README.status.md`, and "9: what a person has dismissed..." from
  `temper/docs/design/client/forge.status.md`.
- Pre-approved:
  - If the world holds no second project without a change to its root's configuration, drop that
    one line of `a_dismissal_reaches_every_client_of_the_person` and say so; the domain's tests hold it.
  - If a tier's budget is passed, stop and ask with the measurements; four small stories should not.

## Steps

1. The fixtures, if any; the four stories.
2. Each story biting: remove the mark in the decision, and the restore arm, each once, and see the
   first and the third fail for that reason.
3. The status lines; the implementer's checks; commit and push.

## Report

As the implementer's text asks; the tests' CPU time against the tier's budget.

## Answers

2026-11-04. May this task finish with the inherited focused CPU overrun under `## Left`, or must it wait
for existing tests to meet the 30-second budget?

**Finish, with the overrun under `## Left`.** This narrows the spec's "stop and ask": it means the cost
this task adds, and the first run asked with its measurements, so that stop is spent. The foreman takes
this answer as the authority for `Left`. No budget is raised and the overrun is not called green:
`skein/docs/foundation/testing-strategy.md`, 8.1 still holds it red, against the tests that spend it.

- **The four stories pass 8.1's questions:** they belong in this tier (the plan's meeting point in
  `temper-client-world`; `temper/docs/design/domain/testing.md`, 2.1), and at 0.133 CPU s together they
  are not the cause. Of the 44.451881 CPU s, existing tests hold 44.297069.
- **The rest is outside this task:** its largest named costs are in
  `temper/crates/temper/tests/suite/forge_land.rs`, and this task may change nothing in
  `temper/crates/**`. No story here can repair it, so waiting would hold the plan for nothing it owns.
- **The measurements go to the user** through this answer's report; the repair, or a figure, is theirs.

The next run starts from the pushed `3fb1d2d32d` and keeps what it holds; the mutations need no repeat.
It does what the budget stop left unobserved, at the head it pushes, the last prerequisite merge in:

1. `heavy cargo nextest run -p temper-client-world --test suite -E 'test(/dismissed::/)'`, 4 passed,
   then `heavy cargo nextest run -p temper-client-world`.
2. `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings` and
   `heavy cargo check -p temper --tests`.
3. The report gives the four stories' CPU time and repeats the `Left` entry with the first run's figures.
   Re-measuring the direct dependent's package is not asked for.

Stop and ask again only if a story of this task is what is over: one past the focused world test's 1 s
guide (`skein/docs/foundation/testing-strategy.md`, 8).
