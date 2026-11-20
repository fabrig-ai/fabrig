# Two walks of one journey make the same bytes

Draft, 2026-11-18, on the user's word. Journey placeholders (`temper/docs/design/client/journey-placeholders.md`) stopped the build, the ports and the approval codes from changing a recording on each walk. Times, the seed and the turns' clocks still change, so the recordings themselves differ every time. That is 7 files, rewritten almost whole on each walk (11-17's regeneration, 62d25104c3), plus every print where a page draws a time. Two branches that record the same journeys again still conflict on them. That design leaves a fixed clock and seed for `again` as its first open question (8).

## 1. What happens

- **`again` walks on a fresh fake box and a headless browser** (`temper/docs/design/client/web/journeys.md`, 8.1; `temper/docs/design/client/tui/journeys.md`, 8). Both read the host's clock, and the box and its worker draw fresh randomness (`temper/crates/temper-client-journeys-tool/src/drive/io/deployment.rs`, the worker's secret).
- **A recording keeps what the walk drew:** the platform's seed, the start's clocks, each turn's clocks and deadline, and the engine's times inside its replies (`temper/docs/design/client/web/journeys.md`, 3.2). The replay gives them back as they were. So two walks share none of them, and their recordings differ throughout.
- **A print reads only the recording's clocks** (`temper/docs/design/client/journey-placeholders.md`, 2.4). So one recording prints the same bytes on any day, but two walks print differently wherever a page draws a time.

## 2. Wanted

- **`again` walks on fixed time and a fixed seed:**
  - the fake box, its worker and its fake model, and the browser or terminal client `again` drives, start from one fixed instant and one fixed seed, the same for every walk;
  - their clocks advance as `again` decides, not as the host's do, so that the same steps make the same turns at the same times.
  - The design chooses how each part is given its clock and seed: the box's configuration or a flag for fake boxes alone, the browser's own virtual time (`skein/testing/skein-browser/src/browser.rs` drives it), the TUI client's start.
  - None of it reaches a box that isn't fake.
- **The same screens make the same bytes:** two walks of one journey, on different days and hosts, make byte-identical recordings and prints. This includes the prints that draw a time.
- **Where a part can't be fixed** (an order of exchanges that real processes still decide, for one), the design says what still differs and why. The rest still holds.
- **The committed journeys are walked again once**, so their recordings take the fixed time. That is `again`'s own run, after landing (temper/tiers'), not the goal's.
- Revise `temper/docs/design/client/journey-placeholders.md` (2.4 and 8), `temper/docs/design/client/web/journeys.md` (3.2, 8.1 and 10) and `temper/docs/design/client/tui/journeys.md`, with their status files.

## 3. Out

- `add` of a journey a person walks in a real browser (`temper/docs/design/client/web/journeys.md`, 3.5), which keeps the walk's own times.
- Keeping a committed recording that still holds (`temper/docs/design/client/journey-placeholders.md`, 8).
- What a journey shows, and how its screens look.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The journeys tool's tests (`temper/crates/temper-client-journeys-tool/src/tests.rs`, `temper/crates/temper-client-journeys-tool/src/again/normalised_tests.rs`):
  - `again`'s walk gives the box and the client the fixed instant and seed;
  - a recording made from fixed inputs keeps them;
  - two runs of `again`'s walk logic over the same scripted saves make the same recording.
- The fake box's tests (`temper/crates/temper-box-shell/src/fake/tests.rs`): the fixed clock and seed are taken by a fake box and refused by any other.
- The committed journeys still replay and print unchanged, by name.

Run only the new and affected tests, by name.
