+++
id = "world-front"
part = "tui-front"
kind = "feature"
needs = ["tui-seam", "tui-welcome", "tui-front-views", "domain-front", "serve-sign-in"]
files = [
  "temper/tests/client/src/tui.rs",
  "temper/tests/client/src/component.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/src/front/account_acts.rs",
  "temper/tests/client/src/fixtures.rs",
  "temper/tests/client/tests/suite/tui.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 120
+++

# The joint world's TUI in front of a box: signed in by the handed link, following the browser to the playground, the box's state and log

## Goal

The TUI side's meeting point (`temper/docs/design/client/README.md`, 6.17, *Its tests*; section 10, *The TUI in the joint world*): the shipped host in front of a box, joined to the engine's clients' component in temper's joint world, with a browser beside it. A TUI in front signs in from its handed link with no key and draws the welcome; the browser signed in by the printed link ticks "Signed in as the owner"; the playground the browser starts shows in the TUI's header and quiet line; lines of the box's given as reports show its state and `/log`. The box itself is not here (`front-command`'s and `binary-front`'s stories drive it).

Acceptance, by 6.17 and `notes/contracts.md`, sections 3, 7 and 8:

1. **The world's engine says `linked`:** `component.rs` keeps `pub(crate) linked: bool`, false at start, set at `Request::LinkSignedIn` (replacing the empty arm `jig-sign-in` left), and the forges document the world delivers carries it (`temper_service::family::forges` as `serve-sign-in` left it, given the flag), each forges watcher delivered again when it turns true (the world's engine marks its forges, as `serve-sign-in` does in `temper-service`). If `serve-sign-in` already gave the world this, use it and say so.
2. **The world's TUI in front** (`tui.rs`): `Tui::front(world, who, credential) -> Self` starts the shipped host with `Start { front: true, local: true, .. }` and answers its first duty `local` with the handed link, the world's local link at that moment (`world.front().local_link()`, standing for the box's first local link, contracts section 5), every later one as `File::Engine`; `Tui::boxed(&mut self, world, line: BoxLine)` reports `Report::Front(line)` and carries its duties. The handed and the printed link are compared by equality and named in words, never printed (`notes/common.md`, *Never shown*); `file_secret_is_in_no_frame` holds of the handed link.
3. **A first-run world for the TUI:** `fixtures::tui_first_run_world(seed)`, the printed link offered (`Joint::offer_printed_link`, as `with_printed_link` does), the deployment showing the welcome, and the person holding no project's role, so that `me` names none, as the welcome's worlds make one (`front/account_acts.rs`' `fixture_root`, `first_run_projects`); reuse what they have before adding.
4. **Three stories** in `tests/suite/tui.rs` (`notes/common.md`, *Tests stay cheap*), a failing test first for each:
   - `a_tui_in_front_signs_in_by_its_handed_link_and_ticks_the_browsers_sign_in`, in the first-run world: a TUI in front, the box's `link` line given (the printed link, `front::printed_link()`), run: signed in with no key (a credential kept, "● Live", `reads == 1`, `sign_in_sends == 1`, no approval asked, nothing opened), the welcome drawn with the printed link whole (its frame `contains` it) and "In the browser"; then the browser signs in by the printed link (a second shipped host pasting it, as the either-order story stands one in, or a world browser client), run: the TUI's welcome says "Signed in as the owner", with "1 Welcome" current; the handed link is in no frame.
   - `a_playground_the_browser_starts_shows_in_the_tuis_header_and_quiet_line`, in the playground's world (`front/account_acts.rs`' `playground_fixture(seed, false)`, its scripted start publishing the fake row, `publish_playground_forges`): a TUI on the playground's project's launchpad, in front or not; the browser's root presses "Try the playground" (as `tests/suite/playground.rs` does), run: the TUI's forge side reads the playground again (the `forges` watch gained the fake row) and its header holds "playground" before the place and the banner's words, its quiet line "/ for commands · playground"; Home's headline is not drawn.
   - `a_tui_in_front_shows_the_boxs_state_and_its_log`: a TUI in front signed in; `restarting forgejo-3` given: the quiet line says "forgejo-3 restarting"; three `log` lines given and `/log` typed and sent: its note shows them oldest first under "/log"; `running forgejo-3`: the quiet line's word gone; `gave-up forgejo-3`: the give up's note "The box gave up: forgejo-3. /log shows why; Ctrl+C closes."; then Ctrl+C closes at once (`closed`, nothing asked).
5. **Status:** delete the **Not yet** bullet naming `world-front` in `temper/docs/design/client/README.status.md`. No other design line.
6. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p temper-client-world --test suite -E 'test(/^tui::/)'` while working, then `cargo nextest run -p temper-client-world` whole once (the forges document's `linked` reaches every story's watch).

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`, sections 3, 5, 7 and 8.
- `temper/docs/design/client/README.md`, 6.17 whole; section 10, *The TUI in the joint world*; `temper/docs/design/domain/testing.md`, 6.3.
- The world: `temper/tests/client/src/tui.rs` (`File`, `Tui::with_file`, `carry` and its duty `local`, `press`, `type_words`, `run`, `text`, `status`); `src/component.rs` (`Request::Link`/`LocalLink` about 793, `Delivery::Forges` about 231); `src/front.rs` (`printed_link`, `local_link`, `latest_link`, `start_playground` about 1632, the forges delivery about 2007); `tests/suite/tui.rs` (`a_fresh_tui_signs_in_from_the_engines_link_with_no_key` about 207, the either-order story about 233, `file_secret_is_in_no_frame` 196); `tests/suite/playground.rs` (the playground pressed and run); `fixtures::tui_world`.
- The landed tasks: `tui-seam` (`Start.front`, `Report::Front`), `tui-welcome`, `tui-front-views`, `domain-front` (`linked`, `signed_in`, the read again), `serve-sign-in` (`family::forges` and the flag).

## Owns and may change

Owns the files listed: the world's `linked`, `Tui::front`, `Tui::boxed`, the first-run world, the three stories and the status bullet. Not the TUI's crates or the client domain (landed by their tasks: a gap there is reported under `## Left` with the story's failing line, not fixed here), not `temper-service`, not the world's other stories.

Options, each decided:

- **The browser:** a second shipped host pasting the printed link, as the either-order story stands one in; if a world browser client signed in with `jig_client::With::Link` reads more simply, use it and say so. The playground is started by a browser's root as `tests/suite/playground.rs` starts it.
- **The playground's project in `me`:** the scripted playground answers project 1, which the playground world's person may already hold; the story needs the TUI on that project's launchpad and the playground seen running there, not the move from the welcome to it (the host's rows cover that). If the TUI can't reach it there, say so under `## Left` with the failing line.
- **Run until:** `tui.run(&mut world, fixtures::WITHIN)` between steps, as the stories do; no new deadline.
- **Ctrl+C:** the key the stories' key reader takes for closing (`Key::Control('c')`, or what `skein_ui_screen_terminal`'s reader reads as `Command::Close`).

## Steps

1. The stories, seen failing; then `component.rs`' `linked`, `tui.rs`' `front` and `boxed`, the first-run world; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(/^tui::a_tui_in_front/)'` after each.
2. The status edit; the runs of acceptance 6; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
3. Commit (why: the TUI in front meets the engine and the browser: the welcome, the sign-in ticked, the playground, the box's state), with `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says, with each story and the rule broken by hand for it (for instance `linked` never set, or the playground's read again dropped), how the browser and the playground were driven, and anything under `## Left`.

## Rebase (2026-11-08, step merge, foreman tui-box-front/world-front/foreman-1)

Resolve the status-file conflict by retaining both tasks’ bullet deletions.
The conflict is in `temper/docs/design/client/README.status.md`.
Integration commit `62e3bf868f` (`front-command`), merged by `d1efa97711`, deletes the **Not yet** bullet for descriptor 3 and `temper internal tui --box`.
This branch’s `1a173ce387` deletes the adjacent **Not yet** bullet naming `world-front`.
Rebase onto the updated integration tip and resolve the hunk with both bullets absent.
Keep the following **Not yet** bullet for 6.17, *The place*, unchanged.
No Rust changes are needed for this merge refusal.
