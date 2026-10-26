# browser-1: decisions

The decisions this plan rests on, numbered. The supervisor's rulings
during the run follow them, from 101. Then the design's questions with
their answers, and the choices left to the user.

## The plan's own

1. **skein's prerequisites are one session, first.** The brief puts
   them in the plan's first skein session. skein 01 starts at once and
   has four steps, ordered by what temper 04 reads first: the page's
   address, back and forward's loaded page, Shift+Enter, then the click.
   temper 04 waits for all four and repins once (decision 5).
2. **Four temper sessions, by part.** temper 01 serves (`temper-service`),
   temper 02 builds the shell (`temper-client-domain`, the new shell
   crate, the bundle), temper 03 gives the view its look and composers
   (`temper-client-view`), and temper 04 is the browser tier. 01, 02 and
   03 touch different crates and run side by side. The meeting point is
   temper 04.1's first story, the printed link signing in: it needs only
   the routes (01.2) and the bundle (02.3). The policy (01.3) is first
   needed by the attack stories (04.5).
3. **`temper.css` is the shell's, begun with the bundle.** temper 02.3
   writes its palette and font (`browser.md`, 2.5), so the bundle never
   waits for the view. temper 03.3 adds the rules for the tokens it
   sets; 03.1 and 03.2 need nothing of temper 02.
4. **The browser tier holds ten stories**, each proving what only a
   browser shows (For the user, 1). Every other rule is proved on the
   host: the routes, headers and limits in `temper-service`'s unit
   tests; start's checks and the bundle's read in the binary's start
   tests; the codec in `temper-client-domain`'s;
   the composers and words in `temper-client-view`'s; the guards'
   refusals in jig's mount, as today. The rebound Host stays with the
   process tier (`browser.md`, 4.1, story 5), already in client-api's
   `later.md`.
5. **One temper repin, at temper 04.1.** temper's skein and jig move to
   their `origin/main` in 04.1's first commit. smith, jig and hearth are
   not repinned: skein 01 breaks nothing of theirs, since
   `back_and_forward` has one caller and `Request::Key` its
   constructors in skein alone (`rg` in jig and temper at the base), and
   the other changes are additions or inside `skein-ui-web`'s listener. They take skein at their next repins
   (`later.md`).
6. **The bundle's limits:** `clients.bundle_files` 128 and
   `clients.bundle_bytes` 16 MiB, in every profile unless
   `configuration.md`, 7, gives values, as jig's test service reads its
   bundle. A file's path within the bundle is at most 1024 bytes, and
   the read visits at most twice `bundle_files` entries, so that a
   directory of empty files or deep names is bounded too. The shell's
   dev bundle is far below both (`findings/base-runs.md`).
7. **The bundle's words:** the name `temper_client_shell_web`, the
   prefix `/bundle/`, the noscript words "temper needs WebAssembly and
   scripts.", the page's title "temper", and the failure page's words
   "The application failed." (`browser.md`, 2.1, names none).
8. **The tier's crate has three binaries:** `browser_sign_in` (stories
   1 to 3), `browser_pages` (4 to 7) and `browser_attacks` (8 to 10), so
   that the gate links three test binaries, not ten.
9. **One composer story, one behaviour:** Enter starts one chat with the
   words as typed, a line break made by Shift+Enter among them
   (`browser.md`, 4.1, story 3).
10. **`browser.md` 2.2's guard citation:** it cites "(6; jig's …)", which
    holds no such text. The plan cites jig's `protocol/browser.md`, 12.1
    and 12.2.
11. **Light only.** `browser.md`, 2.5, and the mockups give one palette,
    a light one, while `skein-ui.css` follows the system's dark
    preference (skein's `ui-web.md`, 12). `temper.css` sets
    `color-scheme: light`; a dark palette is the designer's to add.
12. **The small store keeps `jig-client`'s records alone.** `browser.md`,
    2.1, also names the forge's pending adoption among them; the
    adoption form is milestone 2's, and its record comes with it
    (`later.md`).
13. **SubmitsOnEnter holds in the text shell too.** The composers' state
    (temper 03.2) is the view's, so skein-ui-text prints `(enter sends)`
    beside them and submits after `n WORDS` (skein's `ui-text.md`, 5
    and 6). `browser.md` asks it of the browser alone; giving the
    terminal its own composers would split the view by shell, and its
    goldens print synthetic trees, so none changes.
14. **The policy is on every answer `Server::answer` gives.** skein-http
    refuses a malformed, oversized or overlong head (400, 413, 431)
    before the request reaches temper, so those answers carry no policy,
    as jig's test service's do not. They hold no page to frame or sniff
    (Design questions, 5).
15. **The tier's crate is `tests/client-browser`,** package
    `temper-client-browser-world`, as jig's tier is named.
    `domain/testing.md`, 7, lists temper's test crates and has no row
    for it; the design gains one.

## Design questions

Asked and answered on 2026-10-19. temper's designer amends the design text in one pass,
after moving `testing.md` to `domain/` and `tools.md` to `client/`
(temper `5587ae49b`); the plan cites the amended sections, and that pass
fixes this plan's citations if the plan lands first.

1. **The browser tier's peers.** `browser.md`, 4, runs `temper serve`
   with skein's forge peer and fake LLM peer. No milestone-1 story needs
   either, and skein's browser harness has no slot for hosted peers.
   Recommended: the tier serves with an unreached forge and LLM endpoint
   on closed loopback ports, and the peers join with the first story that
   needs them (browser-2).

   Answer: as recommended. Milestone 1's tier serves with an unreached
   forge and an unreached LLM endpoint, on closed loopback ports, as
   jig's tier does; the peers join with browser-2's first story that
   needs them. Nothing awaits forge-kit's or real-loop's steps. The
   designer amended `browser.md`, 4 ("What it runs against"; temper
   `a959f6e10`).
2. **Reading the page's address.** `browser.md`, 4.1, stories 1 and 2,
   check the address. skein's fake person has no step that reads it.
   Recommended: skein 01.1 adds one (CDP's navigation history), and
   "before the first request" is held by the shell's start order,
   untested.

   Answer: as recommended. The stories check `/` once the page shows.
   The designer amends `browser.md`, 4.1, stories 1 and 2, and skein's
   design where the fake person's steps are named.
3. **Shift+Enter.** skein-browser's keys have no modifier. Recommended:
   skein 01.3 adds Shift, and the story stays (decision 9).

   Answer: as recommended. One story proves both "Enter starts a chat
   once" and Shift+Enter. The designer names Shift in skein's design
   with the address step.
4. **The stylesheets' order** (`browser.md`, 2.5). Recommended: the
   bundle carries `skein-ui.css`, then `temper.css`, which takes its
   rules from `mockups/style.css`.

   Answer: as recommended. `mockups/style.css` is not shipped. The
   designer amends `browser.md`, 2.5, and 2.2's citation (decision 10).
5. **The policy on skein-http's own refusals** (decision 14).
   `browser.md`, 2.2, puts the policy on "every answer", but skein-http
   answers a malformed, oversized or overlong head itself. Recommended:
   the policy is on every answer temper gives.

   Answer: as recommended; the designer added the line to `browser.md`,
   2.2 (temper `116d0a645`).

## Rulings

None yet.

## Answers received

- 2026-10-19: Design questions 1 to 4, each as
  recommended; the citation of decision 10 stands; the real-loop planner
  is told of the configuration fixtures' hotspot (README, Rules).
- 2026-10-19: Design question 5 as recommended; decision
  13 noted and accepted.
- 2026-10-19: the process tier runs on hel8, which gets
  hetzner's pinned Forgejo 17.0.0 and runner 13.2.0 under
  `~/.local/opt`, with `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` set
  (README, Prerequisites).
- 2026-10-19: the parts of the pages that no type carries
  (`findings/carriers.md`) are not this plan's. The accounts banner and
  the composer's account line go to LLM accounts' part 2, and the
  frame's Board, Changes and Accounts entries to their pages' milestones
  (`later.md`). `domain/testing.md` 7's row for `tests/client-browser`
  (decision 15) is queued for temper's next design pass.

## For the user

1. **The ten browser tests,** each with why it can't be proved lower
   (decision 4). Every story also fails on a policy violation, an
   exception or a native dialog, which skein's harness reads.
   - `the_printed_link_signs_in`: the page takes the secret from its own
     address, puts `/` in its place, and the browser keeps jig's cookie.
     Only a browser holds the address bar, the history entry and the
     cookie jar.
   - `a_reload_on_a_chat_signs_in_by_its_cookie`: a reload is a new start
     of the page, which signs in with `Transport` from the browser's
     cookie and opens the chat it was on.
   - `a_spent_link_shows_the_signed_out_words`: the page's handling of a
     refused link runs only in wasm32. The refusal (jig's mount) and the
     words (the view) are proved on the host.
   - `back_then_forward_returns_to_the_chat`: skein's `back_and_forward`
     (`browser.md`, 4.1, story 2). The history, and the page it brings
     back, are the browser's; the chat exists first, from a start of
     `temper serve` on the same scratch.
   - `a_non_page_address_opens_the_inbox_at_slash`: the page replaces
     the address. The codec's choice is proved on the host.
   - `enter_starts_one_chat_keeping_a_shift_enter_break`: the key
     listener and its `preventDefault` run only in wasm32, and the text
     area's line break is the browser's (decision 9).
   - `a_panic_shows_the_failure_page`: skein's panic story at
     `/test/panic`. The panic hook and the failure page exist only in a
     browser.
   - `a_cross_site_request_from_another_port_does_nothing`,
     `a_cross_site_request_from_another_host_does_nothing`: jig's attack
     stories. They prove that a browser sends what the guards refuse
     (`Origin`, `Sec-Fetch-Site`, the cookie's `SameSite`); the refusals
     themselves are jig's tests.
   - `a_framed_page_shows_nothing`: only a browser enforces
     `frame-ancestors`.
   Decision: accepted, 2026-10-19 (each proves only what a
   browser alone can; the rest is in the worlds and the real loop).

2. **No repin of smith, jig or hearth** onto skein 01 (decision 5).
   Decision: accepted, 2026-10-19.

3. **The bundle's limits,** 128 files and 16 MiB (decision 6).
   Decision: accepted, 2026-10-19.

4. **The bundle's words and title** (decision 7).
   Decision: accepted, 2026-10-19.

5. **Light only** (decision 11).
   Decision: accepted, 2026-10-19 (dark stays browser.md 5's
   open question).

6. **Enter sends in the terminal too** (decision 13).
   Decision: accepted, 2026-10-19.
