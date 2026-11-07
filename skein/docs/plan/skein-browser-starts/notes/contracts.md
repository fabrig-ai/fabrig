# skein-browser-starts: the shapes that cross tasks

These are binding. A task builds its side from here, and doesn't wait for another task unless its `needs` says so.

## Stories in turn (task `in-turn`; skein/docs/design/ui-web.md, 14.2)

In `skein/testing/skein-fake-person-browser/src/harness/turns.rs`, re-exported from `harness` as `harness::{Story, name_failure}`:

```rust
/// One of a person's stories in turn (skein/docs/design/ui-web.md, 14.2).
pub struct Story {
    pub name: &'static str,
    pub steps: Vec<Step>,
}

impl Harness {
    /// One person plays the stories in turn, in one page.
    pub fn run_stories(&mut self, stories: Vec<Story>) -> Outcome;
}

/// Where a run failed, by story: a `Failed` report opens `story NAME, its step K: `,
/// and its `person` and `step` stay. `starts` holds each story's name and the run's
/// step it starts at, in order.
#[must_use]
pub fn name_failure(outcome: Outcome, starts: &[(&str, u32)]) -> Outcome;
```

`name_failure` is public for a `Play` of the test's own that takes turns (`live_tree`'s `TreeStory`).

## The fake browser (task `harness-world`; skein/docs/design/ui-web.md, 14.2)

`harness_fixture`, started with `--headless` as its first argument, is the browser. Nothing outside `harness-world` depends on it yet.

## A `Rig`'s cases (task `rig-cases`; skein/docs/design/browser.md, 9)

In `skein/tests/browser/tests/support/real.rs`, shared by `browser_real` and `browser_facts`:

```rust
/// A fact played in its own person and page, in a `Rig`'s one Chromium.
pub struct Case { /* its name, its scenario, its pages and its check */ }

impl Case {
    #[must_use]
    pub fn new(name: &'static str, scenario: Scenario) -> Case;
    /// Serves `body` at this case's `/facts`.
    #[must_use]
    pub fn page(self, body: Vec<u8>) -> Case;
    /// Prepares this case's server, as `Rig::tab_script` and `Rig::beside_page` do.
    #[must_use]
    pub fn prepare(self, prepare: impl FnOnce(&mut Rig) + 'static) -> Case;
    /// Reads what the case left, before the next case begins.
    #[must_use]
    pub fn check(self, check: impl FnOnce(&Rig) + 'static) -> Case;
}

impl Rig {
    /// The cases in order, in one Chromium, within one `run`'s deadline.
    #[must_use]
    pub fn cases(chromium: &Path, profile: &Path, cases: Vec<Case>) -> Rig;
}
```

`Rig::new` and `Rig::run` keep their signatures.
