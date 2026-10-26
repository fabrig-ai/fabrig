# Session 11: webhooks on the box

The process tier's owned Forgejo (temper 10.2: provisioned by reconcile, with the runner and CI) gets the webhooks that
setup.md 10 promises: the configuration names the listener and the forge's secret file, `app.ini` allows the listener,
reconcile's org hook carries the secret, and Forgejo's deliveries reach `temper serve`. 11.1 turns webhooks on for every
owned story and shows a real delivery leave the box signed; 11.2 adds the hinted story (setup.md 13.4): the landing story
with `poll`, `poll_max`, `slow` and the change's `stall` raised past its budget and its CI's job outlasting `settle`,
which lands only if the job's end is delivered. A wait no delivery ends is a question (below), not a fix here. Test code
alone.

## Read first

- **Design:** setup.md 10 (the waits; "Checked"), 13.4, 5 (step 6), 6.2, 8; domain/testing.md 2.6; forge.md 5, 7; configuration.md
  6 (a secret file's final line feed), 7 (the keep's paces, the hooks' limits); skein's testing-strategy.md 2.9 and 9;
  workflow.md 2; development.md 5.5, 5.6.
- **Plan:** decisions.md 5, 12, 16, 19; findings/webhooks.md 6, forge-and-tiers.md 4, measurements.md 3; temper 05.3
  (`support/webhooks.rs`, the wiring), 06.2 (`settle`), 08.1 (`support/owned.rs`), 10.2 (`owned::tier_configuration`).
- **Code, at the base** (08.3 and 10.2 move lines in `support/`; find them again by name): process.rs:70-85;
  support/forgejo.rs:8-40, :105, :114; support/mod.rs:5, :15-40, :432-436, :462-469, :492-494; temper-service
  `src/limits.rs`:75-81, `src/limits/names.rs`:105-108; forge-client `src/limits.rs`:66, `src/keep.rs`:1193-1209; the
  throwaway's `src/lib.rs`:73-93 (`Outgoing`), `src/instance.rs`:29, :73.

## What the design asks

- **setup.md 10, 13.4, domain/testing.md 2.6:** "Checked" is the hinted story: a test of its own on the owned Forgejo with CI
  and webhooks on, `poll`, `poll_max`, `slow` and `stall` raised past its budget, its job outlasting `settle`, no
  conflict resolved. It checks the table's third row, CI on a head; the landing's other waits end on temper's own writes,
  and the other rows are checked from hints in the worlds (the designer's answer to Q2, 2026-10-18). The tier aims at 3
  minutes (Q1).
- **forge.md 5, 7:** a delivery is signed with HMAC-SHA256 in `X-Forgejo-Signature` and named by `X-Forgejo-Event`; 11.1
  checks both on a real one. Past `hooks` connections a delivery waits and none is refused: the cap raises no question.

## Choices this plan makes

- **Webhooks are on for every owned story** (`Throwaway::start`'s users: process.rs:36, :73, live.rs:8): one
  provisioning, no flag to forget. The live test gets them too and is not run by this plan (decision 16).
- **The hinted story is the landing story's function called with four overrides** (`forge.client.poll_ms`,
  `poll_max_ms`, `slow_ms` and `forge.change_policy.stall_ms` = 600 000) and a slow job (the person's `slow` on `main`),
  no second scenario. `hinted_ms` (1 s), `settle_ms` (30 s) and the hooks' limits stay the profile's.
- **11.1's receiver is std's** and the signature is checked with skein's `Hmac` (05.3's `webhooks::signature`), so neither
  the engine's listener nor temper's decoder reads Forgejo's delivery. The push is a contents-API commit, not git.
- **`application.webhooks` reaches reconcile and Forgejo by 05.3's wiring** (05's file: whichever of 05.3 and 10.1, and of
  05.3 and 08.1, lands second wires each); 11.1 proves both. An increment of 11 that lands after 14.1 takes the owned
  members, 11.1's two included, from 14.1's `owned::application`; 14.1 converts those of 11 landed before it.

## Rules that bind it

- The tier needs `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER`, runs directly (README), never in `scripts/check.sh`
  (workflow.md 2), and a test refuses to run without either, naming it (domain/testing.md 2.6). The exports, run before any
  command here that needs the binaries:
  ```sh
  export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 \
    TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  ```
- A time below is read from nextest's `PASS [ t ]` line in the test log; a miss is noted in the landing report.
- Limits come from the shipped profile (`limits.rs`:247; the hooks': `forge::small`, `forge.rs`:110). The hooks' are not
  overridden; a value of the story's own is named beside the profile's, with its reason (11.2).
- `process.rs` and `support/*` are client-api's hotspot until about 10-18 20Z (README): rebase, add beside, never reorder.
- A Size here counts the support alone (code-organization.md 6): `git diff -M --numstat origin/main...HEAD --
  crates/temper/tests/support | awk '{a+=$1} END {print a}'`; the tests of `process.rs` are not counted.

## Increments

### 11.1 The tier's owned Forgejo delivers, signed, to a listener

**Corrections against main (ruling 139, at temper 64f610ae6, before 05.3 landed; they take precedence over the text
below):**
- 11.2's `mod.rs` sketch: delete its `use skein_lib::Duration;` and `use std::path::Path;`, which `support/mod.rs:11` and
  `:76` already import (E0252); without them it passes clippy.
- 06.2 has landed: `settle::window` is at `domain.rs:2025`, the `settle` limit at `limits.rs:79-80` (:68 and :131 read
  "taken as conflicted today"). `poll <= poll_max` is at `limits.rs:68`; the process profile's 10 m is
  `.config/nextest.toml:34-37`.
- Users gain the box's tests, `process.rs:258` and `:281`, which call `tier_configuration` and run with webhooks on after
  11.1; `assert_box_streams` wants each stderr line to start with a step name.
- The three paces also act at `keep.rs:702`, `:777`, `:870` (`poll`) and `:962` (`slow`), besides `rest_resource`
  (`:1193-1213`).
- Line references now: `process.rs` the story :69-83, `Throwaway::start`'s users :36 and :71; `forgejo.rs` start :21, stop
  :132, configuration :142, `landing_scripts` :253; `mod.rs` `WITHIN` :7, `Scratch::new` :51, `ANSWER`/`EDIT` :342-343,
  the judge's messages :437 and :449, `run_until_landed` :465-472 (its literal :471), the judge's deadline :494-497;
  `person.rs:261` `found`, `limits.rs:249` `profile`, `forge.rs:115` `small`, `lib.rs:102` `Outgoing::json`,
  `instance.rs:110` the nap.
- `temper_open_adopts_a_repository_by_commands` took 5.07 s in the check (the file says 13.1 s): the ceiling stands.
- After 05.3 (ruling 141, at temper 01e467afc): the `webhooks.rs` sketch's `use skein_lib::Duration;` and `use
  std::net::TcpListener;` go too (`support/webhooks.rs:4` and :11 import both). `support/mod.rs` moved by one line
  (`pub mod webhooks;` at :7): `WITHIN` :8, `Scratch::new` :52, `ANSWER`/`EDIT` :343-344, the judge's messages :438 and
  :450, `run_until_landed` :466-473 (its literal :472), the judge's deadline :495-498, the imports :12 (`Path`) and :77
  (`Duration`); `limits.rs:252` `profile`; `forge.rs:116` `small`. `owned::document` (`support/owned.rs:21`) takes no
  webhooks input and writes `"webhook_secret": null`: set `application.webhooks` and the forge's `webhook_secret` in
  `forgejo::configuration` after it, or give `owned::Names` a field for them.
Scope: the tier's owned document names the webhooks' listener and the forge's `webhook_secret` (setup.md 8), which 05.3's
wiring hands to the preparation (`app.ini`'s `ALLOWED_HOST_LIST`, setup.md 6.2) and to reconcile (the org hook, setup.md 5
step 6); one test shows the box's Forgejo deliver a push to that address, signed with the secret reconcile drew (decision
19: reconcile's org hook is proved here, not in its world). That the engine takes the delivery is 11.2's.
Needs: temper 05.3 (`application.webhooks`, `forges[].webhook_secret`, their wiring, `support/webhooks.rs` with
`free_address` and `signature`; the ready check reads their signatures), 06.2 (`settle`: a pull read at once after its
push is taken as conflicted today, domain.rs:2025-2029) and 10.2 (`owned::tier_configuration`, the owned `Throwaway`).
Files: `crates/temper/tests/support/webhooks.rs` (05.3's: gains `Delivery`, `receive`), `support/owned.rs` (08.1's; 10.2's
`tier_configuration`, `document`), `support/forgejo.rs` (`impl Throwaway`: fields `directory`, `webhooks`; `configuration`
:114), `support/mod.rs` (`Forge::Throwaway`, which 10.2 trims of `token` and `user`: gains `webhooks: Option<SocketAddr>`),
`crates/temper/tests/process.rs` (one test; hotspot). Users of what changes: `git grep -n 'Forge::Throwaway' --
crates/temper/tests`; `git grep -n 'tier_configuration' -- crates/temper/tests` (13.1's tests then run with webhooks on).
Shape: sketches below.
- `owned::tier_configuration` (10.2's) writes `application.webhooks = {"listener": {"socket": <address>}}` and
  `forges[0].webhook_secret = "forge-1/webhook_secret"` (setup.md 8's path), where 05.3 left `null`s (configuration.md 3,
  10); the address is `webhooks::free_address()`, chosen beside Forgejo's `listen`. `Throwaway::start` reads it back from
  that document and keeps it (`None` for `null`); `forge()` hands it to `Forge::Throwaway`, so `forgejo::configuration`
  writes the same members for `temper serve`, both through `owned::document` (10.2's shape 3): one address for all.
- `receive` accepts within `within` on a non-blocking listener (a 50 ms nap between tries, as instance.rs:73), reads one
  request whole (the head to its blank line, then `Content-Length` bytes), answers `204`, and returns the two headers
  by lower-cased name and the body.
- `push` is the contents-API call of 10.2's shape 2: `POST /api/v1/repos/acme/widget/contents/<file>` on `main` through
  `Instance::exchange(&Outgoing::json(..))` (lib.rs:91) under the person's `Token`, as `get` does (:36), the message
  `Add <file>` and the content `aG9vawo=` (`hook` and a line feed).
Test shape: `process.rs`, beside :70; one test, since each starts a Forgejo and a runner. Reuse `Scratch::new` (mod.rs:48),
`Throwaway::start` (forgejo.rs:16, as 10.2 leaves it) and `stop` (:105), `support::WITHIN` (mod.rs:5),
`skein_lib::bytes::find` (process.rs:13). The test listens after `start` returns: what Forgejo tried before (the
repository's creation) met no listener and is not sent again (forge.md 5), and one still in flight is skipped by the
loop. Limits: the profile's. The reference test, in full:

```rust
// crates/temper/tests/process.rs (11.1); `support` as the file's head has it
#[test]
fn a_push_is_delivered_signed_with_the_secret_reconcile_drew() {
    let scratch = support::Scratch::new("hook_delivery");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    let listener = std::net::TcpListener::bind(forgejo.webhooks()).expect("nothing holds the address");
    forgejo.push("hook.txt");
    let delivery = loop {
        let delivery = support::webhooks::receive(&listener, support::WITHIN);
        if skein_lib::bytes::find(&delivery.body, b"Add hook.txt").is_some() {
            break delivery;
        }
    };
    assert_eq!(delivery.event, "push", "Forgejo names the event");
    assert!(skein_lib::bytes::find(&delivery.body, b"refs/heads/main").is_some(), "a push to main");
    assert_eq!(delivery.signature, support::webhooks::signature(&forgejo.secret(), &delivery.body));
    forgejo.stop();
    scratch.remove();
}
```

Ceiling: 60 s (a box starts in about 15 s: adopt-by-commands takes 13.1 s, measurements.md 3); `receive` waits `WITHIN` (30 s).
Proof: process, `a_push_is_delivered_signed_with_the_secret_reconcile_drew`, by the exports, then `cargo
nextest run -p temper --profile process -E 'test(=a_push_is_delivered_signed_with_the_secret_reconcile_drew)'`, shows a
push made through the API arrive at the configured address with `X-Forgejo-Event: push` and a signature equal to the
HMAC of its body under `webhook_secret` less one final line feed; within 60 s. Then the landing story (process.rs:71),
by the same command and `-E 'test(=a_persons_change_lands_on_the_forge_and_is_found_after_a_restart)'`, passes with
webhooks on at the profile's paces within 180 s. Nothing runs higher (live needs an outside service, decision 16).
Don't: hold the address from anywhere but this test's listener (no `temper serve` runs in it); read Forgejo's delivery
log (no API; the page is HTML); set `ALLOWED_HOST_LIST` or the hook by hand (the preparation and reconcile do); compute
the expected signature with temper's code.
Size: 30 lines of support code; the receiver, the three methods and the members may take it to about 50 (tolerance).
Removes: nothing.

### 11.2 The hinted story
Amended by the designer's answer to Q2 (2026-10-18; setup.md 10, "Checked"): the fourth limit, `stall`, and the slow job.
Scope: setup.md 13.4's hinted story, the fourth test of domain/testing.md 2.6: the landing story with `poll`, `poll_max`, `slow`
and the change's `stall` raised past its budget, and its CI's job outlasting `settle`, so it lands within the budget only
if the job's end is delivered and the reads after it: CI on its head is the one wait it meets that the forge ends. Its
changes never conflict (one edit of `README.md`; the person's `slow` lands on `main` before the goal), so none is
resolved (Q3).
Needs: 11.1 (`Throwaway::push`), and 06.2's `settle` (a hinted pull read before Forgejo settles its mergeability is not
taken as conflicted; a job's hint whose CI reads pending is read again within `settle`).
Files: `crates/temper/tests/process.rs` (the story's body moves into `fn landing`; hotspot), `support/mod.rs`
(`LANDING`, `RAISED_MS`, `raise_paces`; `run_until_landed` :462-469 uses `LANDING` for its literal at :468),
`support/forgejo.rs` (`repository`'s `check.sh`, :353).
Shape: sketches below. `raise_paces` runs after `support::configuration`: it reads `config.json`, inserts the four
names into `application.limits.overrides` and writes the file private (`write_private`, mod.rs:15). The names
(names.rs:105-108 and :120, `forge.change_policy.stall_ms`) sit beside the profile's 5 s, 30 s, 40 s and 30 s
(temper-service `src/limits.rs`:75, :77, :81, :154) at `RAISED_MS` = 600 000, because the story's budget is 180 s
(`LANDING`; a compile-time assertion keeps the relation); `poll <= poll_max` holds (forge-client `src/limits.rs`:66).
The paces act in the keep's `rest_resource` (keep.rs:1193-1209); `stall` in the change's `waiting`
(temper-engine-domain-forge-change `src/domain.rs`:174-181), whose wake reads the pull and CI afresh.
The slow job: `repository` writes `check.sh` as `test ! -e slow || sleep 60` and then `test -s README.md`, so a job on
a tree holding `slow` sleeps 60 s: twice `settle` (30 s, `src/limits.rs`:83). The settle window a not-mergeable read
opens ends `settle` after the change enters `Checking` (temper-engine-domain-forge `src/settle.rs`:21-44), within a few
seconds of its push, which starts the job; so the window's last read comes before the job ends. In the hinted case,
`landing` calls `forgejo.push("slow")` (11.1's) after `Throwaway::start` and before `temper serve` starts; the landed
case never does, and its job stays as it is.
Test shape: `process.rs`, one new test calling the existing story's function. Reuse `landing_scripts` (forgejo.rs:218),
`person::adopt_and_change` (person.rs:233), `person::found` (:266), `ANSWER` and `EDIT` (mod.rs:339-340),
`run_until_landed`, `merged_pull`, `main_readme`. `landing`'s body is the story's (process.rs:72-85) as 10.2 leaves it.

```rust
// crates/temper/tests/process.rs (11.2)
#[test]
fn a_persons_change_lands_on_the_forge_and_is_found_after_a_restart() {
    landing("landed", false);
}

#[test]
fn a_change_lands_by_deliveries_with_every_poll_raised() {
    landing("hinted", true);
}

/// The landing story; `raised` pushes `slow` and sets the four limits to `RAISED_MS` before `temper serve` starts.
fn landing(case: &str, raised: bool) { todo!() }
```

Ceiling: `LANDING` (180 s; the judge's deadline is 175 s, mod.rs:492-494).
Proof: process, `a_change_lands_by_deliveries_with_every_poll_raised`, by the exports, then `cargo nextest
run -p temper --profile process -E 'test(=a_change_lands_by_deliveries_with_every_poll_raised)'`, shows the change
merged on the owned Forgejo after its CI passed, found again after the restart, with every poll and the stall at 600 s
and its job sleeping 60 s; within 180 s.
Once, by hand, before landing, the control, a hinted story that could pass without deliveries proving nothing:
`owned::tier_configuration` writes `application.webhooks` and `forges[0].webhook_secret` as `null` (the handle's
`webhooks` is `None`; `forgejo::configuration` follows), committed on its own and
dropped after (`git reset --hard HEAD~1`). With the exports and `mkdir -p ~/src/rust/tmp/temper/forge-setup/logs/temper-11`
done, the same command with `--tail 400` after `--gate` and `2>&1 | tee
~/src/rust/tmp/temper/forge-setup/logs/temper-11/control.log` at its end exits non-zero (the pipe's status is tee's) and
the log holds `native chat did not settle` (mod.rs:432-436) or `native person's chat did not finish and settle` (:446),
the judge's deadline. With no deliveries, nothing reads the change's CI within the budget: the settle window's reads
end before the job does, and the stall's wake comes 600 s after the change enters `Checking`. If the story lands, it
proves nothing: raise Q2. The landing report quotes the matching line.
Don't: change `hinted_ms`, `settle_ms`, a hook limit, `LANDING` or the story's scripts to make it pass; shorten the
job's 60 s; push `slow` in the landed story; add a sleep or a poll to the test; give the hinted story a scenario of its
own; make a conflict.
Size: 12 lines of support code; `LANDING` and its guard take it to about 18, inside the tolerance.
Removes: nothing (the story's body moves to `landing`).

## Interface sketches

```rust
// crates/temper/tests/support/webhooks.rs (11.1), appended to 05.3's module
use skein_lib::Duration;
use std::net::TcpListener;

/// One delivery to the listener: its event, its signature (hex HMAC-SHA256 of the body) and the body whole.
pub struct Delivery {
    pub event: String,
    pub signature: String,
    pub body: Vec<u8>,
}

/// Accepts one connection within `within`, reads one request whole, answers `204`; panics if none arrives.
#[must_use]
pub fn receive(listener: &TcpListener, within: Duration) -> Delivery { todo!() }
```

```rust
// crates/temper/tests/support/forgejo.rs (11.1), appended to the module: no new `use` lines
impl Throwaway {
    /// The address `application.webhooks.listener` names; nothing listens there until a test or `temper serve` does.
    #[must_use]
    pub fn webhooks(&self) -> SocketAddr { todo!() }
    /// The bytes of `forge-1/webhook_secret` under the handle's `directory` less one final line feed (configuration.md 6).
    #[must_use]
    pub fn secret(&self) -> Vec<u8> { todo!() }
    /// The person commits `file` to `main` of `acme/widget` through the contents API, message `Add <file>`.
    pub fn push(&self, file: &str) { todo!() }
}
```

```rust
// crates/temper/tests/support/mod.rs (11.2), beside `run_until_landed`
use skein_lib::Duration;
use std::path::Path;

/// The landing story's deadline: `run_until_landed`'s (a literal at :468 today).
pub const LANDING: Duration = Duration::from_secs(180);
/// The keep's `poll`, `poll_max` and `slow` and the change's `stall` in the hinted story, in milliseconds: past `LANDING`.
pub const RAISED_MS: u64 = 600_000;
const _: () = assert!(Duration::from_millis(RAISED_MS).as_nanos() > LANDING.as_nanos());

/// Sets the three paces and the stall to `RAISED_MS` in `directory/config.json`, which stays private.
pub fn raise_paces(directory: &Path) { todo!() }
```

## Questions to raise when reached

- **Q1, which 3 minutes.** setup.md 13.4 aims the tier at 3 minutes; decisions.md 12 reads it as the landing story's
  180 s. With two CI stories the tier's total may pass it (the profile stops at 10 minutes, `.config/nextest.toml`:30-33):
  report README Done when 4's `Summary [ t ]` time; change neither number. Recommended: nothing.
- **Q2, a wait that no delivery ends.** If the hinted story misses its deadline, or the control lands, name the wait
  (setup.md 10's table) and the event from the run's log; do not raise a limit or lower a pace. Candidates: a job-end
  event missing from the hook's list; a job's hint that wakes no change; a hint dropped before the domain was ready
  (forge.md 5); a connection closed at `hook_deadline` (forge.md 8) or kept waiting behind `hooks` past Forgejo's own
  timeout. Raised once, at 11.2's ready check (2026-10-18): the control landed in 52 s, its CI read at the change's
  `stall`; the designer's answer added the fourth limit and the slow job above.
- **Q3, "no conflict resolved".** Read as a property of the scenario: the story asserts nothing about resolution. If the
  design wants an assertion, which observable: the pull's commit count, or the fake LLM's queries?

## Done when

- The tier passes, in README Done when 4's run (this session does not run it): in the log whose path the command's last line
  printed, `grep -c -e 'PASS.*a_push_is_delivered_signed' -e 'PASS.*a_change_lands_by_deliveries' <log>` prints 2, and its
  `Summary [ t ]` line gives the time, reported (Q1 if over 180 s).
- Both tests exist: `grep -c -e 'fn a_push_is_delivered_signed_with_the_secret_reconcile_drew'
  -e 'fn a_change_lands_by_deliveries_with_every_poll_raised' crates/temper/tests/process.rs` prints 2.
- The four limits are named once each: `grep -rho -e 'forge\.client\.poll_ms' -e 'forge\.client\.poll_max_ms'
  -e 'forge\.client\.slow_ms' -e 'forge\.change_policy\.stall_ms' crates/temper/tests | wc -l` prints 4.
- The job is slow only on `slow`: `grep -c 'test ! -e slow || sleep 60' crates/temper/tests/support/forgejo.rs` prints 1.
- They stay out of the gate: `grep -cF 'not binary(/^(process' .config/nextest.toml` prints 2.
- No forbidden word: `! grep -rn -e actual -e genuine -e legacy -e compat -e workaround
  crates/temper/tests/process.rs crates/temper/tests/support` prints nothing.
- The control ran: `grep -q 'chat did not' ~/src/rust/tmp/temper/forge-setup/logs/temper-11/control.log` exits 0.
