+++
id = "issuer-connections"
part = "issuer-connections"
kind = "feature"
needs = ["peer-rejected"]
files = [
  "temper/crates/temper-fake/src/oauth.rs",
  "temper/crates/temper/src/fake.rs",
  "temper/crates/temper/tests/fake_backends/walk/sign_in.rs",
  "temper/crates/temper/tests/support/boxed.rs",
  "temper/docs/design/client/journeys.status.md",
]
effort = "high"
minutes = 90
+++

## Goal

A fresh fake box's ChatGPT sign-in fails in a browser, so no recording reaches past Ready. The diagnosis is read from the code and has not been run: the fake issuer's cap of 2 connections, filled by the sockets Chromium keeps after the authorize 302. Prove it with a red walk, fix it in the lowest tier, make a rejection visible, and walk it in Chromium to Home.

Acceptance:

- The new walk, run at the trunk's cap of 2, fails with a `Failed` `last_end` of `later`. The report quotes the line.
- `temper-fake/src/oauth.rs` passes `CONNECTIONS` (8) as the peer's `connections`, and the walk passes.
- A connection rejected at the cap writes one line to the fake issuer's stderr, and the walk reads it in `fake-issuer.log`.
- The Chromium walk reaches Home with "work/hello is in Hello", with its screenshots on the builder in `~/src/fabrig/ui-signin/`. The report names the builder.
- `journeys.status.md`, 9 says what stands after that walk (step 7).

## Inputs

- `temper/docs/plan/ui-signin/inputs/request.md`: the failure and the diagnosis.
- `temper/docs/plan/ui-signin/notes/common.md`.
- `temper/docs/design/protocol/setup.md`, 7.5: the welcome's fake backends.
- `temper/docs/design/protocol/setup.md`, 7.2: the box's diagnostic logs.
- `skein/docs/design/oauth.md`, 5: the fake issuer.
- `temper/docs/design/client/journeys/project-landing/README.md`: the intended walk.
- `fake-signin-model/record-landing`'s report, quoted in step 6, is how that walk was driven last.

## Owns and may change

Owns: `temper/crates/temper-fake/src/oauth.rs`, `temper/crates/temper/src/fake.rs`, `temper/crates/temper/tests/fake_backends/walk/sign_in.rs`.

May change:

- `temper/crates/temper/tests/support/boxed.rs`: `visit_address` becomes `pub(crate)`, nothing else.
- `temper/docs/design/client/journeys.status.md`: step 7's one edit. This departs from the implementer's text, which keeps `docs/design/` out of a task's hands. Only that edit, and only on a passing Chromium walk.

Leave `temper/docs/design/client/journeys/project-landing/**` and the recording: they are `ui-welcome`'s. Leave `PLAN_SLOTS`'s value, `CODE_SLOTS`, and the `llm` and `forge` peers.

Options, each with its default:

- The red fails with an ending other than `later` that a rejected connection explains (say the key set's `Unverified`, because `account connect` printed its address before the key set was fetched): carry on, and quote the ending in the report.
- The red passes at cap 2, or fails for a cause a rejected connection does not explain: the diagnosis is wrong. Change nothing else. Reproduce in Chromium through CDP (step 6's way), print `temper ctl account list --json </dev/null`'s `last_end` after each step, push what you have, and end with a `Question:` naming what you found.
- The Chromium walk stops at a point this cap does not explain: keep the pushed walk and fix, print `last_end` after each step, and end with a `Question:` naming the step.
- `issuer-connections` is the first reader of `Peer::rejected()`. If it does not fit the sketch in step 4, adapt at the call site and say so.

## Steps

1. **The red walk first.** The request opens the two sockets "just before the visit". This spec opens them at the authorize hop, a stated departure: `support/sign_in_browser.rs`'s `Visitor` closes each hop, and at cap 2 two sockets held before the visit would reject the visit itself, so the walk would panic in the visitor and never reach `Failed Later`. Chromium's sockets are there after the 302, not before it, so the walk makes the browser's hops itself, on plain `std::net::TcpStream`s. The first socket is the authorize hop's own, which makes the order certain: both sockets are the peer's before the engine's POST connects.

   In `temper/crates/temper/tests/fake_backends/walk/sign_in.rs`, beside `a_chatgpt_browser_return_on_the_fake_box_leaves_ready_able_to_create`:

   ```rust
   #[test]
   fn a_chatgpt_sign_in_beside_a_browsers_idle_issuer_sockets_leaves_ready_able_to_create() {
       let walk = Walk::owned("walk-idle-sockets");
       // The sockets outlive the acts: an Rc shared by the second act and the check.
       walk.run_then(
           vec![
               Act::Now(Call::new(&["forge", "setup", "--key", "walk-here", "--json"])),
               Act::Now(Call::new(&["account", "connect", "1", "--key", "walk-plan", "--json"])),
               Act::After(Box::new(move |seen| {
                   // browser_hops(address printed by connect), kept in the Rc.
                   Call::new(&["account", "list", "--until", "settled", "--json"])
               })),
               Act::After(Box::new(|seen| {
                   // Assert accounts.usable, its message printing last_end and signing,
                   // as a_chatgpt_sign_in_on_the_fake_box_... prints them.
                   Call::new(&["account", "list", "--json"])
               })),
           ],
           |config, client| {
               ready_can_create(config, client);
               // Step 5's rejection, then the sockets drop.
           },
       );
       walk.scratch.remove();
   }

   /// What Chromium leaves on the issuer: its authorize hop's socket, kept alive, and
   /// a spare opened beside the navigation. Only then does it follow the redirect.
   fn browser_hops(address: &[u8]) -> [TcpStream; 2] {
       // kept = connect to address's authority; hop(&mut kept, address) answers 302.
       // spare = connect to the same authority; it sends nothing.
       // tab = connect to the Location's authority; hop(&mut tab, location); tab drops.
   }

   /// One GET of `url` on `socket`, read to the end of the response's head, with its
   /// status and Location. It sends `host`, and no `connection: close`.
   fn hop(socket: &mut TcpStream, url: &[u8]) -> (u16, Vec<u8>)
   ```

   - The callback's status doesn't matter here, `last_end` judges. Assert only that it is below 400.
   - Set a read timeout of `WITHIN` on each socket.
   - The address comes from `support::boxed::visit_address`.

   Run it at the trunk's cap:

   `heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(=fake_backends::walk::sign_in::a_chatgpt_sign_in_beside_a_browsers_idle_issuer_sockets_leaves_ready_able_to_create)'`

   Check the count, then read the failure: `last_end`'s `Failed` ending renders as `later` (`temper/crates/temper-client-tools/src/jig/render/document/accounts.rs`). Quote it in the report. Commit the walk.

2. **The fix, in `temper/crates/temper-fake/src/oauth.rs`:**

   ```rust
   /// Connections the issuer serves at once: a browser's six per host, the
   /// engine's one, and one closing. `Peer`'s `valid` needs one socket more,
   /// within the Small profile's 16.
   pub const CONNECTIONS: u32 = 8;
   /// Token plans topped up before each peer pass, for the token requests one
   /// pass can meet. It bounds no connection: `CONNECTIONS` does.
   pub const PLAN_SLOTS: u32 = 2;
   ```

   `connections: CONNECTIONS` in `Document::peer`. Nothing else in `Limits` changes. With no plan queued the issuer answers `Refused`, so a pass of more than two token requests stays bounded.

3. Run the walk: it passes, with both sockets held until Ready is read.

4. **The line, in `temper/crates/temper/src/fake.rs`:** `Host` gains `rejected: u64`, starting at 0, and `drain` calls a new `rejections` beside `self.observations()`:

   ```rust
   /// A line for each connection the issuer turned away since the last drain:
   /// the issuer keeps no records, and this is what its log shows.
   fn rejections(&mut self) {
       let Peer::Oauth(peer) = &self.peer else { return };
       let rejected = peer.rejected();
       for _ in self.rejected..rejected {
           eprintln!("{}: rejected a connection at the cap of {}", self.command, oauth::CONNECTIONS);
       }
       self.rejected = rejected;
   }
   ```

5. **The walk reads the line.** In the new walk's check, after `ready_can_create`, connect `temper_fake::oauth::CONNECTIONS + 1` further sockets to the issuer and keep them open. The peer rejects at least one, since the two held sockets stay open. Wait, within `WITHIN`, for `walk.sandbox().join("data/box/logs/fake-issuer.log")` to hold `rejected a connection`. `temper/crates/temper-box/src/logs.rs` names the file from the step's name; if the file is elsewhere, read that module and use the path it writes.

   Bite: take the `rejections` call out, see the walk fail on the log, and restore.

6. **The walk in Chromium**, on your builder. It is a throwaway driver, untracked, and gone before the push.

   - Build through heavy: `temper` and `smith-fake-claude`, then `temper/scripts/bundle.sh --loopback --record DIR` into a fresh scratch directory.
   - Copy the builder's pinned Forgejo and runner (`TEMPER_FORGEJO`, `TEMPER_FORGEJO_RUNNER`) into the box's fresh cache, in the layout `support::forge_land::cache_forgejo_binaries` writes.
   - Check that callback port 1455 is free.
   - Start the built `temper box --fake-backends --bundle DIR` directly, with fresh configuration, data and cache directories, and keep its stderr.
   - Use Chromium in new headless, by skein's browser kit or its debugging port directly, with `--disable-background-networking` and a fresh profile. Handle `Page.javascriptDialogOpening` (`notes/common.md`).

   Open the box's printed sign-in link and walk it as a person does:

   1. `/welcome`: "Set up temper".
   2. `/welcome?at=forge`: "Run on this machine". On Forgejo's sheet, keep the empty repository `work`/`hello`, and press its "Run on this machine".
   3. `/welcome?at=model`: "Continue with ChatGPT".
   4. `/welcome?at=ready`: the model line "Model ChatGPT plan…", with "Create your project" enabled, "Hello" offered and no "Try later". Screenshot.
   5. "Create your project": Home at `/projects/1` says "work/hello is in Hello". Screenshot.

   Print `temper ctl account list --json </dev/null`'s `last_end` after the model step. Keep the screenshots in `~/src/fabrig/ui-signin/`, with a copy of that box's `fake-issuer.log`, and report how many lines it holds. Stop Chromium and the box (the box logs `Stopped`), and remove the rest of the scratch. No sign-in link or token goes in the report.

7. On a passing Chromium walk, in `temper/docs/design/client/journeys.status.md`, keep **Not yet:** 9's first two sentences. Replace everything from "The fresh walk in Chromium reaches Ready" to the paragraph's end with:

   > The fresh walk in Chromium reaches Home; the journey is still to be walked again and recorded (`journeys/project-landing/README.md`).

   Run `python3 -I -B tools/citations/citations.py --base origin/main`.

8. Checks:

   - `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/`.
   - `heavy cargo clippy -p temper-fake -p temper --all-targets -- -D warnings`.
   - `heavy cargo nextest run -p temper-fake -p temper`.
   - The cap's readers, since it changes what the issuer admits: `heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(/^fake_backends::walk::sign_in::/)'`.
   - Report the new walk's wall time beside its siblings'.

   Commit and push.

## Report

As the implementer's text asks, plus:

- The red's `last_end` line.
- The builder's name and the screenshots' paths.
- `fake-issuer.log`'s line count.
- What the walk's rejection line looked like.
