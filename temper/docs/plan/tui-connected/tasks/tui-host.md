+++
id = "tui-host"
part = "tui"
kind = "feature"
needs = ["tui-view", "screen-render", "screen-keys"]
files = [
  "temper/crates/temper-client-shell-tui/Cargo.toml",
  "temper/crates/temper-client-shell-tui/clippy.toml",
  "temper/crates/temper-client-shell-tui/src/lib.rs",
  "temper/crates/temper-client-shell-tui/src/host.rs",
  "temper/crates/temper-client-shell-tui/src/host/**",
  "temper/crates/temper-client-shell-tui/src/limits.rs",
  "temper/crates/temper-client-shell-tui/src/tests.rs",
  "temper/crates/temper-client-shell-tui/src/tests/**",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 160
+++

# temper-client-shell-tui: the TUI's host and its seam

## Goal

The TUI's one seam, the host that the loop, the client's joint world and
the later recorder all call: temper's client root linked, the TUI's state
and view, the focus and the frame, behind reports in and duties out. It
signs in by a pasted link, keeps or erases the credential as the link
says, and lays the frame with skein's renderer. Nothing drives it yet:
`tui-world` runs it against the engine, `tui-run` on a terminal.

Acceptance:

1. `temper/crates/temper-client-shell-tui` exists: a member of the root
   `Cargo.toml` and an entry of its `[workspace.dependencies]` beside
   `temper-client-shell-terminal`; ordinary Rust, `#![forbid(unsafe_code)]`;
   its lint block from `python3 -I -B tools/lints/lints.py --set binary
   temper/crates/temper-client-shell-tui`, its `clippy.toml` a copy of
   `tools/lints/clippy/ordinary.toml`; its dependencies those of
   `notes/contracts.md`, section 1, but `skein-io` and `skein-shell`,
   which `tui-run` adds with the loop.
2. `host` is `notes/contracts.md`, section 6, exactly, with
   `Settings { pub limits: crate::limits::Limits }` (`notes/tui.md`,
   section 5); `Duties` derives `Default`. `limits` is `notes/tui.md`,
   section 5, but `terminal` (`tui-run`'s): `MOST`, `PASTE`, `INTERVAL`,
   `Limits`, `tui()`, `valid()`; `tui()` is `valid`.
3. **Its reports** do what `temper/docs/design/client/README.md`, 6.13,
   says:
   - `Start`: the TUI's state made at the start's address, the theme of
     its scheme, its size; the root started at the inbox's address
     (`Address::Inbox { filter: Filter::ALL }`) with the records kept and
     `with: credential.map(With::Held)`, as `temper open`'s native client
     starts it (`client.rs`, `loaded`); the credential remembered as kept.
   - `Key`: `skein_ui_screen_terminal::command(&key, view's tree, &mut
     focus, &view limits)`; an `Act` given to `temper_client_tui::event`,
     its event handled (below); `Close` closes; `Moved` and `Nothing` do
     nothing more.
   - `Due`: the root's `fire`. `Resize`: the size. `Kept`: told the state.
   - `Heard`: the root steps the event; a `Reply` or `Lost` of a sign-in
     exchange the host saw go down tells the state `Answered { lost }`.
   - before `Start`, any report panics ("a report before the TUI's
     start"); after a close, every report but `Heard` and `Kept` gives
     `Duties::default()`.
4. **The TUI's events:** `Typed` tells the state; `SignIn` checks the
   field with `jig_client_protocol_http::printed` under
   `Mapping::new(b"/jig")`: an error tells `Refused { NotALink }`; a link
   whose `host` is not the start's address tells `Refused { OtherAddress {
   address: host } }`; this address's link steps the root with
   `Event::Client { event: jig_client::Event::SignIn { with:
   With::Link { secret, held: the credential kept } } }`; `Quit` closes.
5. **After each report**, in this order: the root's requests routed with
   `temper_client_shell_terminal::route::down` (a `Shell` request to
   `asked`, but `ServiceCredential`, dropped as the native client drops it;
   each `Exchange` whose message is `Message::SignInLink(_)` noted by its
   exchange and told the state `Asked`; `Copy` answered `Copied { written:
   Refused }`; every other duty and notice dropped), then the root's
   `take` and its requests routed again; the state told `Link { link }`;
   the credential: `root.client().link().credential` differing from the
   one kept, and `Some` or the link `SignedOut`, is `keep` and is kept;
   the address: the first time the link is `Live` with a party, `address`
   is the start's; the view built.
6. `frame()` lays the view's tree with `skein_ui_screen::render` at the
   size, in the start's theme, with the host's focus, into the host's
   frame (`Frame::new(limits.most)`); before `Start` it is that frame
   cleared to the size with the blank cell. `tree()` is the view's.
   `event_of(&key)` is the event `Key` would give, from a copy of the
   focus, changing nothing. `deadline()` is the root's
   `next_deadline`, none after a close.
7. Rows in `src/tests.rs`, each named in the report (Steps, 3).

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, sections 1, 2, 3
  (`Key`, `command`, `Command`) and 6: they bind.
- `temper/docs/plan/tui-connected/notes/tui.md`, sections 1 and 5.
- `temper/docs/design/client/README.md`, 6.13 (the host and its seam),
  6.14 (the pasted link's checks, the credential at a sign-out
  elsewhere), section 7 (the printed link, `held`, the credential).
- `skein/docs/design/ui-screen.md`, 5.4 (the focus) and 6.2 (the key
  reader).
- The models: `temper/crates/temper-client-shell-terminal/src/client.rs`
  (`NativeClient`: `new`, `loaded`, `event`, `transport`, `outputs`: the
  root's steps, the credential read after each, the sign-in exchanges
  noted by their message) and `temper/crates/temper-client-shell-web/src/host.rs`
  (`report`, `duties`, `event_of`: the seam's shape).

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours:** none: no crate names this one yet. No change to
`temper-client-domain`, `temper-client-view`, `temper-client-tui` or
`temper-client-shell-terminal`; `route` and `limits` there are used as they
are.

**Options, each with its default:**

- The view's words: `temper_client_shell_terminal::limits::view(&limits.native)`
  gives the view's limits and `Words` (`Shell::Terminal`), and
  `temper_client_tui::words::Words::english(words, &limits.native.client)`
  the TUI's. If `view` refuses, or `Words::english` refuses the native
  limits, that is a defect in `tui-view`'s bounds: report it under
  `## Left` with the refusal, and make `tui()`'s view limits from the
  largest bound that passes.
- `Duty::Show` from the root: dropped, since the TUI's screens are not the
  domain's pages. If a test finds the root waiting on a `Went` after a
  `Show`, give it `Went { place: Place::Address(address) }` as the native
  client's `go` does, and say so.
- A sign-in exchange's set: a `skein_lib::Set<Token>` bounded by the
  client's exchanges (`temper_client_shell_terminal::limits::EXCHANGES`),
  each removed at its answer.
- A held credential at start is never a reason to write: equal to the one
  kept, it writes nothing.

## Steps

1. The crate: manifest, `clippy.toml`, `lib.rs` citing
   `temper/docs/design/client/README.md`, 6.13 and 6.16, at module level,
   `pub mod host; pub mod limits;`.
2. `limits` (acceptance 2) and `host` (acceptance 3 to 6). The host's
   fields: the root, the TUI's state (made at `Start`), the TUI's view,
   the env, the requests' and patches' queues, the theme, the size, the
   focus, the frame, the mapping, the credential kept, the sign-in
   exchanges, whether the address was given, whether it closed.
3. The rows, without an engine (`src/tests.rs`, with
   `src/tests/fixtures.rs` if helpers grow):

   ```rust
   fn host() -> Host { Host::new(&Settings { limits: limits::tui() }, 7, Time::ZERO, Wall::EPOCH) }
   fn start(host: &mut Host, credential: Option<Secret>) -> Duties { /* Start at 80 by 24, Dark, b"127.0.0.1:8367", no records */ }
   fn link(address: &str) -> Vec<Key> { /* Key::Paste of http://ADDRESS/jig/v1/sign-in/link?secret=<43 base64url chars>, then Key::Enter */ }

   #[test]
   fn a_pasted_link_is_checked_before_it_is_sent() {
       // rows: (pasted, what asked holds, the frame's text holds)
       //  b"not a link"                     -> nothing asked; "That is not a sign-in link"
       //  link("127.0.0.1:9000")            -> nothing asked; "That link is for 127.0.0.1:9000"
       //  link("127.0.0.1:8367")            -> one Exchange with Message::SignInLink(_), credential None; "Signing in…"
       // each from a fresh host: start, frame() (the field takes the focus), the keys, frame()
   }
   #[test]
   fn the_credential_is_kept_and_erased_as_the_link_says() {
       // rows: (start's credential, then, keep's duties seen)
       //  None,           nothing heard                         -> no keep
       //  Some(40 bytes), nothing heard                         -> no keep; asked opens the stream with it
       //  Some(40 bytes), Heard(Streamed { SignedOut })         -> keep Some(None); the frame says "Signed out."
   }
   #[test]
   fn reports_keep_their_order() {
       // a Due before Start panics (should_panic, its own test); Ctrl+C closes once (close true, then
       // a Key gives Duties::default()); after the close Heard still steps the root (no panic);
       // event_of(Enter) with the field focused is Some(SignIn), event_of(Char('h')) Some(Typed),
       // event_of(Control(b'c')) None; a Resize to 40 by 12 makes frame().size() 40 by 12
   }
   #[test]
   fn the_shipped_limits_are_valid() { /* limits::valid(&limits::tui()), and Host::new with them */ }
   ```

   Bite: send the link without checking its host once, and keep the
   credential only when `Some` once; see the two rows fail, and restore.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-shell-tui --all-targets -- -D warnings`; `heavy
   cargo nextest run -p temper-client-shell-tui --lib`. Commit
   ("temper-client-shell-tui: the TUI's host, its seam and its limits"),
   push.

## Report

As the implementer's text says, and: what `Duty::Show` met in the rows,
the sign-in exchanges' bound, the tests' time, and anything of the seam
you found M3's recording would need and the contract lacks (named, not
added).
