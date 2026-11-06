+++
id = "harness-horizon"
part = "again-setup"
kind = "interface"
needs = []
files = [
  "skein/testing/skein-fake-person-browser/src/harness.rs",
  "skein/testing/skein-fake-person-browser/src/harness/deadline_tests.rs",
  "skein/docs/design/ui-web.status.md",
]
effort = "medium"
minutes = 25
+++

# A program that is no test may set the harness's horizon

## Goal

skein's browser harness ends its whole run at forty seconds, which holds no walk as long as `chat-draft`'s. `temper-client-journeys again`, a person's command with its own bound on each step, needs a longer one. Every test keeps the forty: this is no budget and no test's timeout, so no test may call it.

Acceptance:

- `Harness::horizon(self, horizon: Duration) -> Self`, in the builder's form, so that no `Setup` literal changes.
- Unset, a run is as it is now: `close_at` at 36 s, the deadline 4 s after it (`harness.rs`, near line 996).
- Set, `close_at` is the horizon less the same 4 s close window, and the deadline the horizon. The lent peers' startup keeps its 36 s.
- A test in `deadline_tests.rs`, of the pure sum: unset gives 36 and 40, ten minutes gives 596 and 600, and a horizon under the close window is refused by an `assert!` that names it. It needs no browser. Break the sum by hand once and see it fail.
- The method's comment says no test sets it, and why (`skein/docs/design/ui-web.md`, 14.2).
- `ui-web.status.md`'s 14.2 entry goes.

## Inputs

- `skein/docs/design/ui-web.md`, 14.2, *Its run ends at forty seconds*.
- `temper/docs/plan/test-speed/notes/common.md`, for *Status lines*; the tier sections don't apply.

## Owns and may change

Owns the files in the header.

```rust
impl Harness {
    /// Sets the whole run's horizon; ...
    #[must_use]
    pub fn horizon(mut self, horizon: Duration) -> Self;
}
fn bounds(horizon: Duration) -> (Duration, Duration); // close_at's patience, the deadline's
```

`Runtime::run` (near line 990) takes `setup: &Harness`, so it reads the field there.

Readers: no call site changes. `chat-reading-c/reading-browser` changes `harness/script.rs` and `face.rs`: stay out of them.

If the run's io or a peer has a bound of its own that a longer run meets (a `Deadline` armed at 36 s other than the startup's), it takes the same horizon, and say which.

## Steps

1. The bounds and the method; the test; break it once.
2. `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-fake-person-browser --all-targets -- -D warnings`.
3. `heavy cargo nextest run -p skein-fake-person-browser`, and the harness's own browser stories: `heavy python3 -I -B tools/tiers/run skein browser -- -E 'binary(=browser_harness)'`, checking the count isn't zero.
4. Commit and push.

## Report

The method, the test, the rule broken, what ran.
