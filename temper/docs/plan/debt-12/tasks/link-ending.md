+++
id = "link-ending"
part = "link"
kind = "feature"
needs = ["link-surface", "forge-close"]
files = ["skein/crates/skein-link/src/link.rs", "skein/tests/link/**", "skein/docs/design/link.status.md"]
effort = "high"
minutes = 60
+++

# `Link` holds how it ends in one state

## Goal

`held` and `Ask` restate `ending` while `socket` is `None`, and `pump` and `close` move `ending` out and back
for it. One `Copy` state and one `end(next, below)` say it all, about 30 lines fewer, and that closes
`skein/docs/design/link.status.md`'s 3.3. First pin what no test drives: `Up::Failed` and `Up::End` during
a close, and a close after TLS failed. `Link::socket` goes last, now that no crate reads it.

Acceptance:

- `Ask`, `held`, `mem::replace` and `Link::socket` are gone from `skein-link/src/link.rs`.
- The behaviour of `skein/docs/design/link.md`, 3.2 and 3.3 holds: the existing link world passes with no
  assertion changed but the two reads of `socket` (step 4), and the three new tests pass before and after step 3.
- `heavy cargo nextest run` passes for `-p skein-link -p skein-link-world -p skein-forge-connection -p
  skein-forge-machine-world -p skein-forge-setup -p skein-forge-setup-world -p skein-llm-connection -p
  skein-llm-connection-world -p skein-oauth-accounts -p skein-oauth-accounts-world`: this changes what every
  owner stands on.
- `heavy cargo clippy -p skein-link -p skein-link-world --all-targets -- -D warnings` passes.
- `link.rs` ends at least 25 lines below its base, tests apart.

## Inputs

- `temper/docs/plan/debt-12/notes/common.md`
- `temper/docs/plan/debt-12/notes/contracts.md`, 1 and 3
- `temper/docs/plan/debt-12/inputs/link-review.md`, findings 4 and 10's last sentence
- `skein/docs/design/link.md`, 3.2, 3.3 and 4
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/crates/skein-tls/src/client/close.rs`, for what TLS answers while closing

## Owns and may change

- **Owns:** `skein/crates/skein-link/src/link.rs`, `skein/tests/link/`.
- **May change:** `skein/docs/design/link.status.md`, step 5 only. This departs from the implementer's text, for
  that file.
- **Options:**
  - If step 3's shape breaks a rule of `skein/docs/foundation/programming-model.md`, 10 (a `match` must
    list every variant, so a tuple `match` needs one arm per pair or `|` patterns), list the pairs.
  - If a new test of step 2 shows the link doing something else than the expected lines say, and
    `link.md`, 3.3 says what the test expects, stop and ask: that's a fault, and this task doesn't change
    behaviour.

## Steps

1. Read `Link` whole. The cases that matter: a close asked with no socket (Graceful or Aborted) makes its request
   in `connecting`; an abort after such a close replaces it; `closed()` ends it. `connecting` is called once.
2. A first commit, on the code as it is: three tests in `skein/tests/link/tests/suite/close.rs`, driving a close
   with `world.room = false`:

   - `a_stream_failure_during_a_close_closes_the_socket`: `Up::Failed` while Flushing. TLS answers `Closed`:
     Graceful, the socket's `Close`, no `Abort`, `next_deadline` is `None`.
   - `a_stream_end_during_a_close_waits_for_its_deadline`: `Up::End` while Flushing. TLS keeps its demand for room:
     nothing moves, `next_deadline` stays `DEADLINE`, and `fire` at `DEADLINE` gives the `Abort`.
   - `a_close_after_tls_failed_closes_the_socket`: `Up::Failed` while Open, so `above` holds the stream's failure
     and `Event::Failed`; then `close(false)` and `settle`. TLS answers `Closed` at once: the socket's `Close`, no
     `Abort`, nothing more above.

   Break each by hand: drop the Flushing arm's `Closed` handling for the first and third, and `fire` for the
   second.
3. The shape, in a second commit:

   ```rust
   #[derive(Clone, Copy, Debug)]
   enum Ending { Open, Flushing { until: Time }, Graceful, Aborted, Closed }

   /// Ends the stream: nothing queued leaves, and the socket hears what `next` says, or will once it is known.
   fn end(&mut self, next: Ending, below: &mut Queue<io::Request>);
   ```

   - `end` clears both queues, sets `ending`, and makes the request of Graceful (`Close`) or Aborted (`Abort`) if
     the socket is known. Clearing in Flushing's Graceful is safe: `pump` pops `cipher` first, and nothing
     follows TLS's `Closed`.
   - `connecting` keeps the socket, then makes that same request if `ending` is Graceful or Aborted. Share it with
     `end` in one private function.
   - `pump` matches `(self.ending, event)` once, with no `mem::replace`. `tls::Event::Closed` in Open stays
     `unreachable!`.
   - `close` matches `self.ending`; its `if abort` arm and TLS's `Close` in Open stay as they are.
   - `fire` is `if self.due(now) { self.end(Ending::Aborted, below); }`.
   - `closed` has no queue to speak to, and stays apart.

4. A third commit: delete `Link::socket`. Its two readers are `close.rs`'s `assert_eq!(world.link.socket(), ...)`
   lines: the first test's `settle` checks what `SOCKET` each request names, and `trace` says what was asked, so
   delete them. That's the only assertion this task deletes.
5. `skein/docs/design/link.status.md`: from **Not yet**, delete what's left of 3.1 and of 4, so **Not yet** goes;
   from **Departs**, delete 3.3 and keep 1 word for word.
6. Fmt, clippy, the tests above, citations. `python3 -I -B scripts/old_forms.py` in `skein/`. Commit and push.

## Report

The report of your implementer text, and: what each new test saw and what you broke for it; `link.rs`'s lines
before and after.
