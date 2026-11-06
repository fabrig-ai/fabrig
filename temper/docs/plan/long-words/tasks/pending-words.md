+++
id = "pending-words"
part = "kept-words"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-client/src/thread/words.rs",
  "jig/crates/jig-client/src/thread/model.rs",
  "jig/crates/jig-client/src/tests/thread.rs",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "medium"
minutes = 20
+++

# A thread bounds a person's kept words by their bytes

## Goal

A thread keeps a person's words by count alone, each within `text`, and temper's view prices them as `lines × text`. A composer is about to send far past `text`, where that price overflows. Bound their total by the thread's `bytes`, so the view can price what they total.

Acceptance:

- `KeptWords::bytes` is the bytes of the words kept.
- `Thread::sending` returns false and keeps nothing when the words pass `limits.text`, as it does now, or when they would take the words kept past `limits.bytes`. At exactly `limits.bytes` it keeps them.
- Words a committed turn has read free their bytes, as `admits` prunes them: a send that fits only after that pruning is kept.
- The clause `10: a thread keeps a person's words by count alone, with no bound on their bytes.` leaves `jig/docs/design/client/domain/client.status.md`'s **Not yet**.

## Inputs

- `jig/docs/design/client/domain/client.md`, section 10, *A person's words, in three states*.
- `temper/docs/plan/long-words/notes/contracts.md`, *The thread*: binding.
- `temper/docs/plan/long-words/notes/common.md`.

## Owns and may change

Owns the files in the header. `client.status.md` is shared with `line-surface` and `read-ends`: remove your clause alone.

Options, each decided:

- If pruning read words needs to happen before the count of bytes, make `sending` prune first (`admits` does) rather than counting read words as kept.
- `begin`, `begin_frame` and `restore` keep their signatures: only `Thread::sending` takes a person's new words outside tests. If a non-test caller of those three appears, leave it and say so in the report.

Not this task: `thread_limits` in temper, and what a refused send says: `composer-domain`'s. The view's prices: `line-prices`'.

## Steps

1. Red first, in `tests/thread.rs`: `kept_words_stop_at_the_threads_bytes` (at the bound kept, one past refused, the earlier words still there), and `words_read_free_their_bytes`.
2. `KeptWords::bytes`; the clause in `Thread::sending`.
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-client --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-client -p jig-client-view -p temper-client-domain`. Commit and push.

## Report

As the implementer's text asks.
