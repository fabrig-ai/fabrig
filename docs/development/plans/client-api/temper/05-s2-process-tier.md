# Supplement temper-05.s2: the whole process tier passes

Added by ruling 367 (the request of 2026-10-18, for
forge-setup's Done when 4, which runs the whole tier within 180 s). Read
first: `temper/05-s1-landing-story.md`, whose Read first, Proof and Don't
hold here unchanged.

Scope: `temper_serve_answers_a_signed_delivery_204`
(`crates/temper/tests/process.rs:309`) passes again on `main`, and so does
each test of the tier that the 13:49Z run of `scripts/proof-one-process.sh`
did not reach. At `main` `acc7496e2`, run once with the whole tier, it
failed at 30.077 s: "at 29999 ms, the referee failed the real loop:
webhook delivery or drain did not finish" (`support/webhooks.rs:115`:
the POST was not answered, or `temper serve` had not exited after its
`SIGTERM`); four tests did not run.

Needs: nothing. It runs off jig's chain, beside temper 06 and jig 19.

Shape:
1. **Every failure first:** the whole tier once with `--no-fail-fast`:
   `sh -c 'export NEXTEST_DOUBLE_SPAWN=0 && cargo build -p
   temper --bin temper && cargo nextest run -p temper --profile process
   --no-fail-fast'`. Each failing test, with its first failure line, is a
   cause of this step.
2. **Focused, ruling 229:** the signed delivery alone, traced at the
   lowest layer that shows which part stops: whether serve printed its
   link, the POST's status and when, and after the `SIGTERM` which of
   serve's components has not closed (the webhooks listener, the clients'
   HTTP server, the embedded worker, the engine). The trace's lines are
   the author's choice, and go before the commit.
3. **If two traced runs do not name it,** one run of the test alone at
   each first-parent commit of `git log --first-parent 7f5ce036f..origin/main --
   crates/` (`7f5ce036f` last changed the referee): the first that fails
   names the change. By their subjects, `3262df8f9` (temper-04.s1,
   "Finish held HTTP calls once their clients component closes":
   `server.rs`, `stop.rs`, `system.rs`) and `63f6174dc` come first.
4. **Repair** where the log names it: temper's production code or the
   tier's support, one commit for each cause, as 05.s1's Shape says. A
   cause in jig (its forge kit hosts the webhooks listener,
   `crates/temper-service/src/forge.rs:77`) stops the step, with the log's
   cause line.

Bound: three focused runs and 40 minutes from the unblock; past it, the
report gives what the trace shows.

**Ruling 369 (the request of 2026-10-18T14:02Z):** forge-setup's
bisect on x1 names the cause, so point 3's bisect is not run. At temper-04.s1's
`3262df8f9` ("Finish held HTTP calls once their clients component
closes"), the signed delivery and `temper_serve_answers_an_unknown_forge_404`
fail every run, and `temper_serve_prints_its_link_and_drains_on_a_signal`
passes 2 of 4 ("exit: None" after 55 s). The same commit with
`crates/temper-service` as at `63f6174dc`, the jig pin and
`temper-client-view/src/confirm.rs` kept, passes them, the signal story 3
of 3. Its change: `Server`'s `finished` and `Server::finish`
(`server.rs:219`), called from `Clients` (`clients.rs:368`); the embedded
worker aborted once `drain.closing` is set (`system.rs:413`); held calls
ended through that path; `stop.rs`'s closing phase made public. It ends a
webhook delivery in flight.
- The repair keeps what rulings 349, 351 and 353 asked of the drain (the
  embedded worker stopped; a held call ended once its clients component
  has closed), and ends no delivery in flight: 04.s1's
  `closing_ends_a_held_call_after_its_component_finishes`
  (`crates/temper-service/src/tests/server.rs:88`) and
  `a_signal_mid_run_drains_and_the_next_start_settles_the_claims`
  (`tests/one-process/tests/one_process.rs:5`) pass with it.
- Its focused test (ruling 229) is a delivery in flight when the drain
  begins, at the lowest layer that shows it.

**Ruling 370 (temper-06.5's block of 2026-10-18T14:04:08Z): a second
cause in the same server.** With 06.5's candidate (`0650656cc`),
`a_chat_reads_then_replies_and_waits`
(`tests/one-process/tests/fuzzy_one_process.rs:31`) fails at seed 7:
"process 0 requires policy deadline 31000000000 ns after the last word".
06.5's trace (`logs/temper-06/chat-proof.md`, `server-trace-368.rs`): at
1 s, `POST /jig/v1/unwatch` (owner 395, call 425) has its whole body
received, its response awaited, and its peer's End. The server keeps it,
with `closing` and `finished` false and the 31 s stall as its deadline
(`server.rs:132`, refreshed on the End; `:229-235`; `:264-284`, closing
waits for the call or `finished`). Its stream, `GET /jig/v1/stream`, ends
on `BrokenPipe` at 1 s, and no `Server::finish` comes (`clients.rs:367-368`).
- First, at the lowest layer (a `temper-service` server or clients test),
  find who owes the unwatch's answer once its peer has ended and its
  stream has closed. If jig's clients component never answers it, that is
  a jig cause: stop with the cause line. If temper's server or mount
  holds it, repair it here with the drain's repair, so that a call whose
  peer has gone holds no deadline past the last word.
- Skein's HTTP server keeps an unfinished response across a read half-end
  (`crates/skein-http/src/server.rs:886-903`), and that stays.
- The bound gains 30 minutes for this cause. Its focused test stays as a
  test of the step. 06.5 proves seed 7 once this step has landed.

**Ruling 371 (05.s2's block of 2026-10-18T14:05:22Z): the bound renewed.**
Three more focused runs and 40 minutes from its unblock for the drain;
then ruling 370's cause, with its 30 minutes.
- Start from `git diff 63f6174dc 3262df8f9 -- crates/temper-service`. The
  first session's trace (`logs/temper-05/s2-diagnosis.md`,
  `s2-trace.patch`) shows the webhooks listener, the clients' HTTP and the
  engine's workers closed, but no end: the drain closing, the store idle
  and not closed, the embedded worker not empty.
- The focused test is a deterministic one, at the lowest layer: a drain
  begun while a delivery is in flight or just answered, traced at
  the drain's accounts check (`stop.rs:46`, `accounts.is_empty()`), the engine's scheduling and the embedded worker's
  abort once `drain.closing` is set (`system.rs:413`).
- A test that fails one run in two may be repeated up to five times in
  one focused run, stopping at its first failure.
- `a_running_engine_lends_the_record_temper_sign_in_kept`
  (`process.rs:106`) failed in the first whole-tier run at 30.142 s
  ("sign-in world deadline; stopped: false") and passed alone. It is the
  same drain, and joins the proof below.

**Ruling 372 (05.s2's block of 2026-10-18T14:23:16Z): two causes, one
here.**
- **Ruling 370's cause is jig's** (`logs/temper-05/s2-unwatch-cause.md`):
  jig-17.s3 answers the unwatch, and temper-06.5 takes it with its repin.
  Its 30 minutes are dropped here. Save the public-component test as
  `logs/temper-05/s2-clients-unwatch.rs` and leave it out of this step's
  commits, with its `views` dev-dependency and `Cargo.lock` change.
- **The drain's cause is temper's**, and the repair is this step's
  (`logs/temper-05/s2-drain-cause.md`). `Accounts::work_pending`
  (`crates/temper-service/src/accounts.rs`) omits the root-close
  continuation that `Accounts::iterate` consumes after `drive_files`. The
  candidate adds it, and the accounts test that names it is the step's
  focused test (ruling 229). No delivery-in-flight test is asked: ruling
  369's "a delivery in flight" was the symptom, not the cause.
- **From here, no more diagnosis.** In one focused run, the signed
  delivery up to five times, stopping at its first failure, then the
  sign-in story. If either fails with the candidate, report its trace.
  Otherwise go to clippy, then the Proof below, the gate, and land.
  Bound: 60 minutes from the unblock.

Proof (ruling 369), in the process profile with both exports: the six
stories, the signed delivery, `temper_serve_answers_an_unknown_forge_404`
(`process.rs:327`), `temper_serve_answers_an_unsigned_delivery_403`
(`process.rs:318`), `temper_serve_prints_its_link_and_drains_on_a_signal`
(`process.rs:10`) four times, the sign-in story (`process.rs:106`, ruling
371), and the landing story with its restart; then
the whole tier once, `scripts/proof-one-process.sh`, within
180 s.

Size: about 100 production lines. Past 200, it asks.
