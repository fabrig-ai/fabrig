+++
id = "world-file"
part = "world-local"
kind = "feature"
needs = ["local-link"]
files = [
  "temper/tests/client/src/component.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/src/tui.rs",
  "temper/tests/client/tests/suite/tui.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 70
+++

# The joint world's link file is the engine's local link: a TUI and a browser each sign in, in either order

## Goal

The request's temper client world (`temper/docs/plan/tui-native-secret/inputs/request.md`, *Tests*, its second and third sentences). The joint world's stand-in for the engine's link file, `tui::File::Engine`, reads the printed link today. After `local-link`, jig's component also gives `Request::LocalLink { secret }`: the world keeps it and the file reads it, so the stories show a fresh TUI signing in from the file and the printed link still signing a browser in, in either order, and the TUI never drawing a printed link. The TUI's crates don't change.

Acceptance, by `temper/docs/design/client/README.md`, 6.12, 6.14 (*Signed in from the engine's link*) and section 10's TUI joint world, and `notes/contracts.md`, section 3:

1. **The world:** `component.rs` keeps `pub(crate) local_link: Option<Secret>`, set at each `Request::LocalLink { secret }` (replacing `local-link`'s dropping arm); `front::Joint::local_link()` (contracts, section 3) makes its URL as `latest_link()` makes the printed link's; `File::Engine` reads `local_link()`, its doc comment saying the engine's link file holds the engine's local link. `latest_link()` stays the printed link's.
2. **The stories** in `temper/tests/client/tests/suite/tui.rs`, a failing test first for each change:
   - `a_fresh_tui_signs_in_from_the_engines_link_with_no_key`: the file's link is `local_link()`, which differs from `front::printed_link()` and from `latest_link()` (the file's contents are the file's secret alone); its other asserts stay; `file_secret_is_in_no_frame` checks the file's link.
   - `a_tui_after_a_browser_spent_the_link_signs_in_by_the_next` becomes a table test, `a_tui_from_the_file_and_a_browser_by_the_printed_link_sign_in_in_either_order`, rows:
     - `file first`: a fresh TUI (`File::Engine`) signs in from the file, `reads == 1` and `sign_in_sends == 1`; then a second shipped host standing for the browser (`ELSEWHERE`, pasting `front::printed_link()`, as the story does today) signs in by the printed link.
     - `printed first`: the browser's host signs in by the printed link; then the fresh TUI signs in from the file at its first read, `reads == 1`, `sign_in_sends == 1`, no approval asked, nothing opened.
     - `a second TUI`: a TUI signs in from the file; a second TUI (a third `ClientId`) started after it signs in from the file at its first read, by a link that differs from the first's (the next local link, drawn when the first spent its own); then the browser's host signs in by the printed link.

     In every row: each signs in (a credential kept, `● Live`, the launchpad's `CONNECTED` text for the TUIs); `latest_link()` stays `front::printed_link()` throughout, so no TUI made the engine draw a printed link; no local link is ever the printed link.
   - `an_expired_sign_in_is_renewed_from_the_file`: its `link` is `local_link()`.
   - `a_tui_signed_out_elsewhere_asks_for_a_link_with_its_token_erased`: unchanged; it pastes the printed link, which the person's paste may spend.
   - No other story changes but where the file's link is read; no new test function beyond the renamed table (`notes/common.md`, *Tests stay cheap*).
3. **Status:** delete the **Not yet** bullet naming `world-file` in `temper/docs/design/client/README.status.md`. No other design line.
4. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p temper-client-world --test suite -E 'test(/^tui::/)'` while working, then `cargo nextest run -p temper-client-world` whole once. A red in `temper-client-world` that `local-link` left (its report's `## Left`, or seen on your base) is yours: fix it in the world's files and name it.

## Inputs

- `temper/docs/plan/tui-native-secret/notes/common.md` and `notes/contracts.md`, sections 1 and 3.
- `temper/docs/design/client/README.md`, 6.12, *The engine's link, kept at rest*; 6.14, *Signed in from the engine's link* (*When*, *The file read*, *Answered `link_used` or `link_expired`*); section 10, *The TUI in the joint world*.
- jig's `jig/docs/design/protocol/client.md`, section 8, *The local link*.
- The world today: `temper/tests/client/src/component.rs` (`latest_link`, about lines 15, 168 and 776; `request`, about line 697); `front.rs` (`printed_link`, `link_url`, about lines 21 to 35; `latest_link`, about line 608); `tui.rs` (`File`, about line 17; the duty `local`'s read, about line 164); `tests/suite/tui.rs` (about lines 190 to 300, and the sign-out story about lines 326 to 371); `fixtures::tui_world` (`temper/tests/client/src/fixtures.rs`, about line 211).

## Owns and may change

Owns the world's `local_link`, `Joint::local_link`, `File::Engine`'s read, the stories above, the status bullet. Not the TUI's crates (`temper-client-tui`, `temper-client-shell-tui`: `post-landing-probes` is changing them), not `jig-client`, not the other stories of the world.

Options, each decided:

- **The browser** is a second shipped host pasting the printed link, as `a_tui_after_a_browser_spent_the_link_signs_in_by_the_next` stands one in today. If a world client signed in with `jig_client::With::Link` reads more simply, use it and say so.
- **The re-read after `link_used`** keeps its coverage in the run's rows (`temper-client-shell-tui`'s tests, 6.14's second read); the joint world no longer reaches it by a browser. Don't add a story for it by advancing the world past `link_span`; if the `a second TUI` row reaches it anyway (a second read), assert what it shows and say so.

## Steps

1. The world's `local_link` and `File::Engine`; the stories' failing asserts first (the file's link differs from the printed link; the `printed first` row signing in at the first read), seen failing on the base's `File::Engine`.
2. The table, the changed stories; the narrowest tests after each.
3. The status edit; the runs of acceptance 4; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit (why: the TUI's file is the engine's local link, so the printed link stays the browser's), push.

## Report

As the implementer's text says, with each test changed or added and the rule broken by hand for it (for instance `File::Engine` reading the printed link again), and any `temper-client-world` red inherited from `local-link` and how it was fixed.
