+++
id = "view-fetched"
part = "client"
kind = "feature"
needs = ["client-fetched", "progress-value"]
files = ["temper/crates/temper-client-view/src/forge/connect.rs", "temper/crates/temper-client-view/src/tests/model_forge.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/docs/design/client/journeys/welcome-playground/**", "temper/docs/design/client/journeys/welcome-set-up/**", "temper/docs/design/client/journeys/skein-ui.css"]
effort = "high"
minutes = 45
+++

# The welcome's bar fills, and the steps say it

## Goal

While the row says how far the fetch has gone, the welcome's progress node is the fetch's: named for the binary, its text the figure, and with a size a fraction that fills `temper-flow-meter`'s bar. "Each step"'s first step says the figure too. With no size the bar keeps moving and the figure is the bytes. This finishes the rows `welcome-fetch-percentage` deferred.

Acceptance:

- with a size: one `Role::Progress`, styled `look::FLOW_METER`, named "Fetching Forgejo" or "Fetching its runner", its own run "42%", its fraction the row's numbers;
- with no size: the same, its run "41 MB", no fraction;
- no line heard, and at preparing and starting: today's node, `look::FLOW_BUSY`, textless, named by the connection;
- the paragraph's name is the whole line, "… · Setting up: fetching Forgejo · 42%", and its muted detail in the browser is "Setting up: fetching Forgejo";
- the terminal says the whole line and has no bar;
- "Each step"'s first step, in the welcome and on `ConnectForge`, says "Fetching Forgejo and its runner · 42%", and with pins "Fetching Forgejo 17.0.1 and its runner 13.2.1 · 42%"; once fetched it says no figure;
- `temper.css`'s meter rule reads `var(--skein-ui-progress, 0%)`, and draws the sliding segment on a meter with no `data-progress`, still under reduced motion.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, The welcome.
- `temper/docs/design/client/browser.md`, 2.5, the welcome's look.
- `temper/docs/design/client/forge.md`, 5.4, Setting up.
- `skein/docs/design/ui-web.md`, section 6 (a fraction) and section 12.
- `temper/docs/plan/fetch-progress-1/notes/contracts.md`, sections 1 and 6, and `notes/common.md`.
- `temper/docs/plan/fetch-progress-1/notes/request/`: the look request's pages and overlay, as drawn.

## Owns and may change

Owns the files in the header. Keep `temper.css`'s edit to the meter's lines: `look-round-3` is in that file, away from them.

A departure from the implementer's text, on commits: each commit carries `Look-request: welcome-fetch-percentage` directly above `Co-Authored-By:` (or last, with none).

Pre-approved:

- A departure from the draft's look, which the request's page draws with a `<span>`: the figure is the node's own run, a text node of the grid. It takes the same grid cell.
- `temper-flow-meter`'s selector is under `.temper-welcome-status`, and the node is built there alone. If the step list's `·` shows on Details too, that is the design; leave it.
- `look::FLOW_METER` is registered already. No `Word` moves: the `·` joins as `welcome_line` joins, by bytes.
- The journeys' pages do not change, the fake box fetching nothing a walk can see: their recordings do, for `SettingUp`'s new field (see Answers). If a page differs by more than Answers allows, stop and say so in the report.

## Steps

1. `forge/connect.rs`, `welcome_line`: the paragraph's `line` stays whole, and `stacked`'s details take `forge::words::forge_connection_bare(row)`. At the node:

   ```rust
   if busy(row) && w.shell == Shell::Browser {
       b.open(Role::Progress);
       if let Some(fetched) = row.fetched {
           b.style(look::FLOW_METER);
           b.name(forge::words::fetching_name(fetched));
           b.run(Run::Plain, &forge::words::fetched_figure(fetched));
           if let Some(total) = fetched.total {
               b.fraction(Fraction { done: fetched.done, total });
           }
       } else {
           b.style(look::FLOW_BUSY);
           b.name(&forge::words::forge_connection(row));
       }
       b.close();
   }
   ```
2. `progress_in`: at the step under way with `row.fetched`, its first line gains ` · ` and the figure, after whichever of the four words it says.
3. `temper.css`: the `::before` rule's two `--temper-flow-meter-fill` become `--skein-ui-progress`; the comment above says the fill is the node's fraction. Add, in the meter's lines:

   ```css
   #app > .temper-welcome .temper-welcome-status > .temper-flow-meter:not([data-progress])::before { background: linear-gradient(var(--gf-ink-2) 0 0) no-repeat, var(--gf-line); background-size: 35% 100%, auto; animation: temper-welcome-meter 1.4s ease-in-out infinite; }
   @keyframes temper-welcome-meter { from { background-position: -54% 0, 0 0; } to { background-position: 154% 0, 0 0; } }
   @media (prefers-reduced-motion: reduce) {
     #app > .temper-welcome .temper-welcome-status > .temper-flow-meter:not([data-progress])::before { background: var(--gf-line); opacity: .3; animation: none; }
   }
   ```
4. Tests in `tests/model_forge.rs`, by its `underway()`: `the_fetch_fills_its_bar_and_says_its_figure`, `with_no_size_the_bar_keeps_moving_and_says_its_bytes`, `a_fetch_no_line_has_reached_keeps_the_busy_bar`, `the_detail_names_the_binary_and_the_name_says_both`, `the_first_step_says_the_figure_until_it_is_fetched`. In `the_terminal_says_the_status_with_no_bar`, rows for a figure with a size and without.
5. `cargo fmt`; the old-forms check; `heavy cargo clippy -p temper-client-view -p temper-client-shell-web --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-journeys`, the last holding the prints unchanged. Commit and push.

## Report

As the implementer's text has it, with each rule broken for a new test and the failure seen, and the terminal's two lines as printed.

## Answers

2026-11-04. **Question: the required journey replays are stale; should the journey owner repair them, or this task widen to investigate?** This task repairs them, and the cause is settled, so there is nothing to investigate.

- **The cause:** `wire-fetched` added `fetched` to `SettingUp`, in place, as `temper/docs/design/draft/fetch-progress.md`, 1, says. `welcome-playground` and `welcome-set-up` hold the forge family's frames as they were given, `SettingUp` without that field's byte. At the first such frame the client's strict decoder refuses the record, `forge/forges.rs` ends the invalid delivery, and the replay asks for `/jig/v1/unwatch` where the recording has a deadline. Neither this task's commit nor the view is at fault; `d3548d9091` stands.
- **The repair is the design's:** the client's exchange changed, so the recordings are stale and are recorded again while their acts hold (`temper/docs/design/client/journeys.md`, 8 and 8.1). Never edit a recording's bytes by hand. This widens the task because step 5's journey check cannot pass without it; the header's `files` now holds these journeys.
- **Keep the pushed work;** do not redo it. On the branch's head, after any merge, one at a time and with stdin closed:

  ```sh
  SKEIN_TEST_CHROMIUM="$(command -v chromium)" TEMPER_BUNDLE=none heavy cargo run -p temper-client-journeys -- again welcome-playground
  SKEIN_TEST_CHROMIUM="$(command -v chromium)" TEMPER_BUNDLE=none heavy cargo run -p temper-client-journeys -- again welcome-set-up
  TEMPER_BUNDLE=none heavy cargo run -p temper-client-journeys
  ```

  The last reprints all, and takes `journeys/skein-ui.css` up to `progress-value`'s 101 rules; `the_committed_journeys_are_their_print` compares that file too. Then run step 5's final command to its end: the 865 tests unobserved last time need a verdict.
- **What may differ in the prints:** the new walk's times and the fake box's loopback ports (8), and `skein-ui.css`. Words, roles, styles or the pages' count are not expected to move: if they do, or if `again` refuses or cannot run, stop and say so in the report with its output; change neither the tool nor a journey's `README.md`.
- **On a later merge conflict in a recording,** take either side and run `again` once more; never resolve one by hand.
- **Commits:** the journeys in a commit of their own, which says why, with `Look-request: welcome-fetch-percentage` as this task's commits carry it.
