+++
id = "root-checks"
part = "root-checks"
kind = "feature"
needs = []
files = [
  "jig/testing/jig-channel-kit/src/referee.rs",
  "jig/testing/jig-channel-kit/src/world.rs",
  "jig/testing/jig-channel-kit/src/fixtures.rs",
  "jig/testing/jig-channel-kit/src/stories.rs",
  "jig/tests/channel/tests/suite/referee.rs",
  "jig/tests/channel/tests/suite/fuzzy/channel.rs",
  "jig/docs/design/protocol/channel.status.md",
]
effort = "medium"
minutes = 40
+++

# The channel referee formats the roots where a grant can arrive, and at every step in fuzzy

## Goal

`observe_roots` (`jig/testing/jig-channel-kit/src/referee.rs`) formats every root with `{:?}` whenever a root has stepped, so that `Rules::roots` can look for a credential grant in the text. In focused, format them where a grant can newly arrive and at the story's end. `fuzzy::channel` keeps every step.

**Formatting only what can hold a grant isn't open here:** `Service::fmt_roots` and `root_steps` are in `jig/testing/jig-test-service`, which is another plan's. Use them as they stand.

Acceptance:

1. `Rules::roots` and `a_grant_value_planted_in_a_root_s_state_is_caught` are unchanged and pass.
2. `roots_at_end` is unchanged and still runs where it runs.
3. In focused, a host's roots are formatted when `RootChecks::due` says so, as below. In `fuzzy::channel`, at each root step as today.
4. `fuzzy::channel` holds `drawn_channel_worlds_keep_the_referee_s_rules` at every root step, and a new `a_replaced_grant_s_roots_are_checked_at_every_step` for `Story::Key { replaced: true }` and `Story::ApplicationGeneration`, which `stories::drawn` doesn't draw.
5. `RootChecks` has its own tests, and each bites.

## Inputs

- `skein/docs/foundation/testing-strategy.md`, 8 (what focused and fuzzy are for).
- `jig/docs/design/protocol/channel.md`, 8 and 12. Section 12 states the rule, "no credential grant's value appears in any domain's state", and, written by this cut, when each suite looks for it. The rule itself doesn't change.
- `jig/docs/design/protocol/channel.status.md`, 12.
- `jig/docs/plan/jig-world-cost/plan.md`, *What the channel's focused suite gives up*: that's the promise this task keeps.
- `jig/docs/plan/jig-world-cost/notes/common.md`.

## Owns and may change

- **Owns:** the spec's `files`.
- **Departure from the implementer's text:** it makes one design edit, in `jig/docs/design/protocol/channel.status.md`: remove the **Not yet** line on section 12's looking for a grant's value, which this cut added, because this task builds it. Another plan's task is also editing that status file, so touch that line alone.
- **Readers, read at the cut:** `Settings` is built by name in one place, `fixtures::calm`. `observe_roots`, `grants_observed`, `roots_observed` and `formatted` are the referee's own. `make_referee` in `world.rs` builds the `Referee`.
- **Leave alone:**
  - `observe_engine`'s late `format!` at `now >= 300 s`: a diagnostic, not this check.
  - Everything in `jig/testing/jig-test-service`.
- **Options:**
  - `make_referee` may take `roots_every_step` as a parameter, or set the field after it's built. Your choice.
  - If `Story::Key { replaced: true }` or `Story::ApplicationGeneration` needs a fixture only `jig/tests/channel/tests/suite/channel.rs` has, use what its `run` helper does and move what's needed into the kit.
  - If a story's outcome turns out to depend on when the referee formats (it shouldn't: formatting only reads), stop and report it as a `Question:` with what you saw.

## Steps

1. **`referee.rs`:**

   ```rust
   /// When `observe_roots` formats a host's roots. A root's state can newly hold
   /// a grant only once the grant exists, so a host is due when the wire's count
   /// of grants has moved since it was last checked, and at its next root step
   /// after that, when the receiving root has taken the grant up. `every_step`
   /// keeps the check at each root step, for `fuzzy::channel`.
   pub struct RootChecks { /* inside is yours */ }
   impl RootChecks {
       #[must_use]
       pub fn new(every_step: bool) -> Self;
       /// Whether this host, at this many root steps and with this many grants
       /// seen on the wire, is due; records that it was checked when it is.
       pub fn due(&mut self, grants: usize, host: u64, steps: u64) -> bool;
   }
   ```

   `Referee` holds a `RootChecks` in place of `grants_observed` and `roots_observed`. `observe_roots` keeps its shape: no check with no grants, `write!` into `self.formatted`, `rules.roots`, the verdict's assert.

   **Where the referee itself replaces a host's service** (the restart after drained, near `host.revision.0 += 1_000_000`), format that host's roots first, since `roots_at_end` won't see what it held. Keep this to that one site.
2. **`world.rs`:** `pub roots_every_step: bool` on `Settings`, with a one-line doc, passed to the `Referee`. **`fixtures.rs`:** `calm` sets it false.
3. **`stories.rs`:** `drawn` sets `settings.roots_every_step = true`. Add `pub fn every_root_step(seed: u64, story: Story) -> Outcome`, for the new fuzzy test.
4. **`tests/suite/referee.rs`** (reach `RootChecks` as `fixtures::referee()` reaches `Rules`):
   - `roots_are_due_where_a_grant_arrives_and_at_the_next_root_step`: not due with steps moving and no new grant; due when the count moves; due once more at the host's next step; not after. Two hosts don't share their state.
   - `every_step_keeps_each_root_step_due`: due whenever steps move, not when they don't.

   Break each once by hand and report it.
5. **`tests/suite/fuzzy/channel.rs`:** `a_replaced_grant_s_roots_are_checked_at_every_step`, asserting `Verdict::Passed` for both stories, with the seed named in the file.
6. Run narrow tests, then the implementer's closing steps for `jig-channel-kit` and `jig-channel-world`, and `--profile fuzzy -E 'test(fuzzy::channel)'` on `jig-channel-world`. `Settings` gains a field, so run `heavy cargo check -p <each> --tests` on the kit's other dependents, by name, as `cargo tree -i jig-channel-kit` lists them: compile only, even for a slow tier's crate. Commit and push.

## Report

The implementer's report, plus: each rule you broke and what the test showed, and anything focused gives up beyond what `plan.md` says, so the plan's statement can be corrected.
