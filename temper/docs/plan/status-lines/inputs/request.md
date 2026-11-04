# Request: the missing design-gap lines in the status files

From fabrig/debt's backfill via the director, 2026-11-03 ~21:00Z (the user approved the
routing). The debt backfill found about 23 design gaps that the code shows and no status file
records. Add one line each, in the status file named, in that file's form (a Gaps and departures
entry: what the design says, what the code does, the debt item in brackets). Docs only: no code,
no design text, except the two stale design texts in jig 7, which are corrected in the design
documents themselves. Where an item says "not re-checked", look at the code first and write the
line only if the gap is still there (note in the report the ones dropped). Where an item says "the
designer decides", record it as a gap and say so.

The items, quoted from the backfill's list (`coord/debt/for-director.md` on hel8, "Design gaps
missing from status files"):


Each gap has a debt item for the code's side, named in brackets.

### jig

1. **Wire fields the engine never fills.** `jig-protocol-engine-clients/src/translate/refusals.rs` sends `None` for: `executor` (line 207), `needed` (210, 217), `asked`/`latest` (214), an ending's `reason` (374, 378), and `party`/`how` (382). It belongs in `jig/docs/design/domain/engine.status.md` and `jig/docs/design/client/domain/contract.status.md`. Either the engine fills them, or a wire version drops them.
2. **Over-ceiling limits are capped silently.** `jig-client-protocol-link/src/limits.rs` caps 15 limits with `.min(w::CEILINGS.…)`. Belongs in `jig/docs/design/client/domain/client.status.md`. The backfill recommends refusing them (D009).
3. **Public `Core` methods with no design text.** `effect_owner` and `application_part_request` (`routing/flight.rs`), `call_room` (`runs/calls.rs`), and the test-only `next_finishing_deadline` and `finishing_due` (`routing/timers.rs`). Belongs in `jig/docs/design/domain/engine.status.md`.
4. **The root fires one due core child per pass.** `engine.md` has no line on it. Belongs in `jig/docs/design/domain/engine.status.md`.
5. **The brief and notes worlds draw no seeds and check no heap,** if `jig/docs/design/domain/testing.md`, 3 asks it of every child's world. Belongs in `jig/docs/design/domain/testing.status.md`.
6. **One charter slot.** Accepting proposed settings answers `ProposalDecided { Accepted }`, not `Full`. Either a capacity check is missing or the expectation is wrong. Belongs in `jig/docs/design/domain/engine.status.md`, or it is a bug.
7. **Stale design texts (fix the documents themselves).** `temper/docs/design/protocol/configuration.md`, line 535, names `hello_bounds`, which no code has. Not re-checked: the quality run says `jig/docs/design/protocol/llm.md`, 5.6 describes a moved-record reader that has been deleted.

### skein

1. **programming-model.md 5.4 departures.** No status file records them, and each crate's says "no gap". The foundation has no status file, so this needs one line per crate, pointing at 5.4:
   - `kv.status.md` (D017);
   - `io.status.md` for the pipe, signals and FileIo (D018);
   - `channel.status.md` (D019);
   - `forge-connection.status.md` (D020);
   - `forge.status.md` for Forgejo's plans (D021);
   - `oauth.status.md` for oauth and accounts (D022);
   - `llm.status.md` and `llm-connection.status.md` (D023);
   - `examples.status.md` for the echo (D024);
   - `lib.status.md` for `RequestTable` (D025).
2. **The setup world plays the host component,** and the Forgejo stories run on a plans-only `World` (`skein/docs/design/forge-setup.md`, 7). Belongs in `skein/docs/design/forge-setup.status.md` (D031).
3. **No rule tells `Io`'s completions from `FileIo`'s** (`skein/docs/design/io.md`, 5). Belongs in `skein/docs/design/io.status.md` (D044). smith's triage asks for the same line (smith D003).
4. **GitHub's dialect stands on Forgejo's modules** (`skein/docs/design/forge.md`, 4.1). Belongs in `skein/docs/design/forge.status.md` (D010).
5. **Test layout departs from `skein/docs/design/testing.md`, 6:** sibling `*_tests.rs` files, inline test modules, and worlds used as kits. Belongs in `skein/docs/design/testing.status.md` (D027, D028).
6. **The keeper's syscalls are in skein-shell's public API,** undesigned (`skein/docs/design/shell.md`, 7.5). Belongs in `skein/docs/design/shell.status.md` (D045).
7. **forge-connection sets the deadline in one place, and the webhook connection has four states** (`skein/docs/design/forge-connection.md`, 8.1 and 8.4). Belongs in `skein/docs/design/forge-connection.status.md` (D020).
8. **"No shared types" against the fakes.** `skein/docs/foundation/testing-strategy.md`, 4 conflicts with fake-forge's server halves, fake-llm, the oauth fake, the fake channel and the fake CDP peer. Belongs in `skein/docs/design/fake-forge.status.md` and `skein/docs/design/fake-llm.status.md`. Settled when D043 is planned: the backfill recommends amending testing-strategy.md, 4.
9. **Not re-checked:**
   - every Responses request sends `"verbosity":"low"`, against `skein/docs/design/llm.md` (`skein-llm/src/translate.rs`): `llm.status.md`;
   - Forgejo's own connection is drawn from the calls' pool, against `forge-connection.md`, 7.2: `forge-connection.status.md`;
   - the kv world against `kv.md`, 9: `kv.status.md`. This looks partly stale, since tests/kv now has crash and model tests.

### smith

1. **The session matches the run's `b"wait"`** (`smith-domain-session/src/session.rs`, line 1929; `session/restore.rs`, lines 218 and 503), against `smith/docs/design/domain/session.md`, 1. Belongs in `smith/docs/design/domain/session.status.md`, which says "No gap" (D004).
2. **`tests/session` and `tests/tools` call protocol crates,** against `smith/docs/design/testing.md`, 2. Belongs in `smith/docs/design/testing.status.md` (D017).
3. **The benchmark runner branches on each agent,** against `smith/docs/design/benchmarks.md`, 6. Belongs in `smith/docs/design/benchmarks.status.md` (D010).
4. **The local domain composes person-facing text,** against `smith/docs/design/domain/host.md`, 8. This may belong in `smith/docs/design/domain/host.status.md`: the designer decides whether it is a departure (D018).

### temper

1. **`temper-forge-setup-shell/src/reconcile.rs` (524 lines) has no product caller.** The status files record only the verb. Belongs in `temper/docs/design/protocol/README.status.md` and `temper/docs/design/protocol/setup.status.md` (D006).
2. **temper-service is a copy of jig's reference service, and nothing says so.** `jig/docs/design/protocol/service.md`, 7 says that service is "written as step code, so that it can be copied", but no temper design or status file says temper-service is such a copy, or where it departs from it. Belongs in `jig/docs/design/protocol/service.status.md`, or in a temper status file the temper designer picks (temper D001, which waits on jig D005).
3. **The `temper-NN` comments make promises no status file holds.** Not checked one by one:
   - temper-07's `not_installed` on the client wire (`temper-service/src/family.rs`; `setup_github.rs` calls it a "known gap");
   - temper-12's merge-queue answers (`temper-engine-smith/src/answers.rs`);
   - temper-13's crowded installations.

   D008's task checks each against its status file.
