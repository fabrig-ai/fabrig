# playground-browser: the plan

From `inputs/request.md`: the playground in the browser, part three, on skein's kit (plan
`playground-skein`) and temper's composition (plan `playground-temper`), both on main:
`temper/docs/design/playground.md`, 3 (the visitor in the composition, the shell's feature), 6
(the page), 7 (served), 9 (the native walk through the web client's host, a log replayed, the
wasm32 builds, the browser tier's walk) and 10 (its bar). The design is complete for this work;
the shapes that cross tasks are `notes/contracts.md`. Every task names `notes/common.md`.

## Parts and tasks

1. **shell-clippy** (`temper/docs/design/client/browser.md, 4.3`), slice 1.
   - `tasks/shell-turn-split.md`, feature, medium, ~45 min, needs nothing: `Shell::turn` split
     with no lint exception, its platform touched in one method; `temper-client-shell-web` in
     `.config/gate/wasm32.txt`; browser.status.md's 4.3.
2. **visitor** (`temper/docs/design/playground.md, 3`), slice 2.
   - `tasks/playground-visitor.md`, interface, high, ~90 min, needs nothing: `Process::Visitor`,
     the visitor's config from the deployment's document, the link printed to the world's
     printer, `story::{reach, parameters, slices}`, `stories::named` (C1, C2).
3. **native-walk** (`temper/docs/design/playground.md, 9`), slice 3: the meeting point.
   - `tasks/native-walk-a.md`, feature, high, ~150 min, needs `playground-visitor`: the page
     natively in `tests/playground` (C3), story A walked by skein's fake person through temper's
     web client host on the in-memory bridge, its log kept as a fixture and replayed (C7).
4. **worker** (`temper/docs/design/playground.md, 3`), slice 4.
   - `tasks/playground-worker.md`, feature, high, ~60 min, needs `playground-visitor`:
     `temper-playground` a `cdylib` whose wasm32 start runs skein's worker loop (C4).
5. **page** (`temper/docs/design/playground.md, 6`), slice 4.
   - `tasks/page-feature.md`, feature, high, ~150 min, needs `shell-turn-split`: the shell
     generic over its platform, `pub mod playground` (the starter, the frame, the failure page)
     on wasm32 with or without the feature, the feature choosing the entry; the page's HTML and
     style (C5); browser.status.md's 2.1, playground.status.md's 3 and 6.
6. **served** (`temper/docs/design/playground.md, 7`), slice 4.
   - `tasks/page-policy.md`, feature, medium, ~45 min, needs nothing: temper-service's policy
     function for the service and the playground, the cache constants, the `headers` template
     held equal by a test (C6).
   - `tasks/bundle-playground.md`, feature, high, ~75 min, needs `page-feature`,
     `playground-worker` and `page-policy`: `temper/scripts/bundle.sh --playground OUT`, the
     `playground` profile, `OUT/headers`, two builds alike (C6, C8); playground.status.md's 7.
7. **replay** (`temper/docs/design/playground.md, 9`), slice 5.
   - `tasks/one-process-replay.md`, feature, high, ~120 min, needs `native-walk-a`: the kept log
     replayed over the playground's composition with the one-process world's forge referee as
     skein's `Turns`, judging the CSV change landed and `main` green.
8. **browser** (`temper/docs/design/client/browser.md, 4.6`), slice 6, **walked** (story A in the
   browser tier, and by hand on Firefox and Safari on a desktop: `playground.md`, 10).
   - `tasks/browser-walk.md`, feature, high, ~150 min, needs `bundle-playground` and
     `native-walk-a`: the tier's runner builds the playground, its static host
     `playground-serve` maps `headers`, the twelfth browser test walks story A at 10×, downloads
     its log and replays it natively, and prints the figures; `playground-figures.md`; the tiers'
     and browser.md's status.
   - `tasks/status-nine.md`, docs, medium, ~20 min, needs `native-walk-a`, `one-process-replay`,
     `bundle-playground` and `browser-walk`: playground.status.md's 9 edited once, from what the
     trunk holds.

## Decisions

- **The visitor is the composition's** (C1), admitted at a fixed index in every world
  `story::world` makes, so a browser's world and a native test's are one.
- **The link goes to the world's printer, not a simulated pipe** (C2): skein's loops call
  `World::step` and `World::printed` with no hook of the application's, and skein's `world.md`, 2
  makes the printer the pipe the world captures. The factory parameters are installed around a
  native loop and for the worker module's life.
- **The playground's page code builds on wasm32 without the feature** (C5), so the gate's plain
  wasm32 clippy of `temper-client-shell-web` (its list holds packages, not features) lints it;
  the feature chooses only the entry.
- **The page's HTML is temper's own** (`--page`), holding the frame around `#app`.
- **The `headers` file is a template filled by the build** (C6), held equal to temper-service's
  policy function by a test, rather than a build tool linking the service.
- **The kept log is the native walk's** (C7), re-recorded under `SKEIN_CODEGEN_WRITE` as temper's
  goldens are; the browser's own log is replayed by the browser test itself.
- **"Under the one-process world's referee" is its forge referee** (`one-process-replay`): the
  parts of the referee that watch the terminal client's person or keep the links have no place
  in a visit's replay, and the log's digests already prove the visitor's answers.
- **The browser tier serves the built playground with a static host of its own** (C8), since the
  tier's servers are all `temper serve` and skein's harness serves no directory; the tier builds
  the playground in its real profiles, so its figures are a visitor's (design written:
  `tools/tiers/design.md`, 2, and `temper/docs/design/client/browser.md`, 4.3, with the tiers'
  status's **Not yet** 2).
- **playground.status.md's 9 is edited once, last** (`status-nine`): four branches close its
  words, and parallel edits of one line would not merge.
- **The bars and the download's address are the user's** (`plan.toml`'s questions `bars` and
  `download-address`); the defaults let every task finish.

## Critical path

`shell-turn-split` (45) → `page-feature` (150) → `bundle-playground` (75) → `browser-walk` (150)
→ `status-nine` (20): about 440 implementer-minutes. Beside it: `playground-visitor` (90) →
`native-walk-a` (150) → `one-process-replay` (120), which `browser-walk` meets (it needs the
native walk's person) and `status-nine` waits for; `playground-worker` (60) and `page-policy` (45)
before the bundle.

## Not in this plan

- **The playground forge's links shown as text** (`playground.md`, 6): not in the request's
  scope; `page-feature` leaves it as playground.status.md's 6.
- **The reset without the box** (`playground.status.md`, 4) and **the documents' bounds** (8):
  other plans' (`domain/forge.md`, 4.1; playground-temper's measurement).
- **Firefox and Safari by hand, the bars, publishing** (10): the walk of slice 6, the user's
  questions, and the release.

## Overlaps with drafts beside this one

- `.config/gate/wasm32.txt`: plan `wasm32-temper` (not in the brief's list; its inputs landed at
  14:41Z) adds `temper-playground` after `skein-playground`; `shell-turn-split` adds
  `temper-client-shell-web` after `jig-client-shell-web`, a line apart.
- `temper/docs/design/playground.status.md`: `wasm32-temper` may close words of its 9; this
  plan's tasks remove words in place, never reflowing (`notes/common.md`).
- `Cargo.lock`: `playground-visitor`, `playground-worker`, `native-walk-a` and `page-feature`
  change their packages' dependency lines, ordered by `needs` where they share a package.
- `tools/tiers/` (`run`, `test_run.py`, `design.md`, `design.status.md`): this cut writes the
  design; `browser-walk` changes the temper bundle step alone.
- `temper/docs/design/client/browser.md`: this cut adds one bullet to 4.3.
- `temper-service`: `page-policy` changes `src/clients/{policy.rs,bundle.rs}` and
  `src/tests/server.rs`, none of the files `configuring-p2-build` lists.
