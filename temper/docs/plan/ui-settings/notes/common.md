# ui-settings: the rules every task names

temper's client is to look and work exactly like the greenfield mock
(`temper/docs/design/client/greenfield/`, its `README.md` says what's what). There is no supported
deployment. Read this note, then `temper/docs/plan/ui-settings/notes/contracts.md`, then your spec.
Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Decide, don't ask.** The user is asleep and takes the plan's judgement. A stored format, a
  protocol or a public interface may change freely: no compatibility, no `Question:`; say it in your
  report. A small edit you need just outside your file list (a re-export, a `mod` line, a design line,
  a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks
  the task.
- **Design text:** the cut wrote the design. A task may edit only the `docs/design/` files its `Owns
  and may change` lists: the status entries it closes, and a line the code taught, named in the
  report.
- **Generated files** your change makes stale (`temper/docs/help/reference/*`, prints, goldens, counted
  baselines, the reference examples) are yours to regenerate in the same task, `docs/help/` included;
  don't park on them. `temper/docs/help/**` is read by tests (`.config/gate/read-by-tests.txt`): run
  `temper-docs`' tests when you touch it.
- **No `Look-request:` trailer:** this plan lands no look request.

## Codecs

A schema change regenerates its code and goldens, one command:

```
SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'
```

The store's and the wire's versions stay as they are (`temper/docs/design/protocol/store.md`, 4):
schema, generated code and goldens change in place. Then a plain `test(drift::)` run must pass.

## Across plans

`ui-goals`, `ui-goal-page`, `ui-needs`, `ui-home-status`, `ui-away`, `ui-chats`, `ui-changes` and
`ui-welcome` work on the rail, Home, chats and goals beside this plan, and fab merges main into this
plan's trunk as they land. So:

- **Read the trunk, not this plan's memory of it.** Where a spec says "if X has landed", grep the
  trunk at your base: its name either exists or does not, and the spec says what to do in each case.
- **`temper.css`:** new rules go under `/* ==== ui: settings ==== */`, never at the end of the file, so
  that parallel plans merge cleanly. Replaced rules are removed where they stand.
- **`look.rs`:** this plan's tokens are its own, `SETTINGS_*`, as `GOALS_*` and the others are; each
  plan's stylesheet rules stay scoped to its tokens, so that a rule of another page never restyles
  these. A token number another plan took on a catch-up is renumbered, the higher one moving.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`** conflict across plans: on a catch-up,
  regenerate rather than hand-merge, and keep `names.txt` sorted as it is.

## Render proofs

Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's
`Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's
`onbeforeunload` hangs a CDP driver. A throwaway render script stays untracked and is gone before the
push. Render the mock from the checkout:

```
chromium --headless=new --no-sandbox --disable-gpu --allow-file-access-from-files \
  --screenshot=OUT.png --window-size=W,H --virtual-time-budget=3000 file://...
```

`settings.html?project=hello` is the project with everything hidden. The mock's state is in
localStorage: to see a state, use a throwaway copy of the page. Its Google Fonts link needs the
network; the client's own fonts are what's compared.

**Reach for the mock, not the old page;** where old tests walk a part the mock removes, change them
with it.
