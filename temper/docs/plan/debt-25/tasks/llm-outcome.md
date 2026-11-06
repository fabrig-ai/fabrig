+++
id = "llm-outcome"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-llm/src/client.rs", "skein/docs/design/llm.status.md"]
effort = "high"
minutes = 50
+++

# skein-llm: the client's outcome is its state

## Goal

Finding 6 in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. `Client.outcome` is a flag beside `State`. A call is pending exactly in `Prepared`, `Head`, `Streaming` and `ErrorBody`, with one exception: `ErrorBody` stays after `Up::End` has sent its terminal, until the HTTP `Done` closes it (`skein/docs/design/llm.status.md`, 3). `fail` is followed by `closing` at most of its sites.

Acceptance:

- `fail_closing(client, env, failure, detail, above, below)` replaces each `fail` + `closing` pair, and the `fail_report` + `closing` pairs get what's shortest.
- `Outcome` goes. `Closing { cancelled: bool }` holds what `Outcome::Cancelled` said. A new state, `ErrorEnded`, follows `ErrorBody` at `Up::End`: it holds nothing and behaves as `ErrorBody` does for every other event, except that it sends no terminal again and `Done` closes it. "Pending" is one method on `State`.
- `Event::Cancelled` still goes up once, from `closed`. `client.state` is still `Closing` after `close_machines`, whichever way it got there.
- `receiving.content = List::with_capacity(0)` in `fail_report` and `body.clear()` in `error_end` go, since the state that holds each is dropped in the same step.
- `Done` from `ErrorEnded` still closes rather than going `Draining`: reusing an errored call's connection would be a visible change.
- No test's visible events or assertions change. Net production lines fall.
- `skein/docs/design/llm.status.md`, 3: the clause "and `outcome`, because ErrorBody stays after its terminal has gone" becomes "and it adds `ErrorEnded`, which waits for the HTTP `Done` once `ErrorBody`'s terminal has gone up, because closing at the end of the body would move a test's events". Date the file as your commit's day.

**If `Outcome` can't go without growing production:** the many match lists gain a variant, and rustfmt may wrap them. Measure first. Then keep `Outcome` and its status clause as they are, land only `fail_closing` and the two dropped lines, and report the measured size of the move. Don't land a partial flag (`Closing { cancelled }` beside `Outcome`).

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, finding 6
- `skein/docs/design/llm.md`, 3's table
- `skein/docs/design/llm.status.md`
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** `client.rs`; `client/**` and `skein-llm`'s unit tests in their spelling only; in `llm.status.md` only the clause above, which departs from the implementer's text.
- **Stay off:** `Waiting`'s values. `ErrorEnded` reports what `ErrorBody` reports. Public types don't change.
- **A path that reaches `closing` still pending, with no failure sent** would make `Closing { cancelled }` too small, since `closed` fails such a call. By reading, none does: `up`'s `closing` is under `Idle`, and `http::Event::Done`'s follows its own `fail`. If you find one, it's a third value in `Closing`, not a flag; say so.

## Steps

1. `fail_closing` and the two dropped lines. Run `heavy cargo nextest run -p skein-llm --lib`. Commit.
2. The state move, then the status clause.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-llm --all-targets -- -D warnings`. This changes how the client keeps a call's end, so run its readers: `heavy cargo nextest run -p skein-llm --lib`, then `-p skein-llm-world`, `-p skein-llm-connection`, `-p skein-llm-connection-world`, `-p smith-protocol-llm-world`. Commit, run the citations tool and push.

## Report

The common note's items; whether the move landed, with both measures; any path you found closing while pending. Existing tests hold the rule, so there's no new pin; break it once anyway: make `ErrorEnded`'s `Done` go `Draining`, name the test that fails, and restore.
