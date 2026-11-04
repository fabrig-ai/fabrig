+++
id = "config-no-profile"
part = "config-fallback"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-config/src/lib.rs", "jig/crates/jig-protocol-config/src/worker.rs", "jig/crates/jig-protocol-config/src/overrides.rs", "jig/crates/jig-protocol-config/src/tests/overrides.rs", "jig/crates/jig-protocol-config/src/tests/worker.rs", "jig/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 35
+++

# A document that resolves no profile gets no bound of Small's

## Goal

When a document's `limits` resolve no profile (the member missing or not an object; `profile`
missing, the wrong kind or unknown), the rest of it is checked against Small. An operator with three
connectors who writes `"profile":"standrd"` gets a true `limits.profile` problem and a false
`connectors` bound; the same for hosts, rules, projects, policies, charters, templates, and a
worker's `slots`. A named profile with a bad override already keeps its bounds
(`limits_problems_keep_the_named_profile_and_applied_overrides_for_connectors`).

These limits are used for diagnostics alone: a document with any problem starts nothing.

Acceptance:

- With no profile resolved, `engine`, `worker` and `overrides::parse` check the document against
  `Profile::Standard`, the largest shipped profile.
- `"profile":"small"` gives Small, and every other case of a named profile is unchanged.
- The new tests fail on the old code with two problems, and pass with one.
- The tests of `jig-protocol-config` and `temper-init` stay green.

## Inputs

- `jig/docs/plan/bugs-jig/notes/common.md`.
- `jig/docs/design/protocol/configuration.md`, 6 ("A document that resolves no profile").
- The code: `jig/crates/jig-protocol-config/src/overrides.rs` (`Parsed`, `parse`), `src/lib.rs`
  (`engine`), `src/worker.rs` (`worker`).
- The tests: `limits_problems_keep_the_named_profile_and_applied_overrides_for_connectors` in
  `src/tests/overrides.rs`; `refuses`, the `table` and `more_slots_than_the_hub_can_host_are_refused`
  in `src/tests/worker.rs`.

## Owns and may change

- `jig/crates/jig-protocol-config/src/lib.rs` and `src/worker.rs`: the one `profile(Profile::Small)`
  line of each.
- `src/overrides.rs`: `parse`'s starting profile, its `b"small"` arm, and `Parsed.limits`'s comment.
  Plan `fabrig-scale` changes this file far from `parse`: touch nothing else.
- `src/tests/overrides.rs` and `src/tests/worker.rs`: the new tests, and the worker's `table` line.
- `jig/docs/design/protocol/configuration.status.md`: this plan's **Not yet** entry for 6.
- Options: if an existing test pinned Small's fallback, it is a surprise: stop and ask. A document's
  override applied over the Standard fallback may draw its own problem: that is a report line, not a
  question, as long as the tests below hold.

## Steps

1. In `tests/overrides.rs`, `limits_problems_without_a_profile_add_no_bound_of_small`: from
   `with(ENGINE_SMALL, br#""connectors": [1]"#, br#""connectors": [1, 2, 3]"#)`, replace the limits
   value by each of `{"profile":"standrd"}`, `{"profile":7}`, `{}` and `7`, and also remove the
   `limits` member entirely. For each, assert one problem, `past` 0, and its path: `limits.profile`,
   `limits.profile`, `limits.profile`, `limits` and `limits` (missing). Each message names its case.
2. In `tests/worker.rs`, `a_worker_with_no_profile_adds_no_bound_of_small`: the fixture with
   `"slots": 3` and its profile `standrd`, one problem, at `limits.profile`; a line in `table`.
3. See both fail with two problems. In `parse`, start from `profile(Profile::Standard)` and write
   `b"small" => limits = profile(Profile::Small),`; change the lines in `engine` and `worker`.
4. `heavy cargo nextest run -p jig-protocol-config --lib -E 'test(=tests::overrides::limits_problems_without_a_profile_add_no_bound_of_small)'`,
   then `heavy cargo nextest run -p jig-protocol-config -p temper-init`.
5. `cargo fmt`; old forms; `heavy cargo clippy -p jig-protocol-config --all-targets -- -D warnings`;
   the citations check; commit and push.

## Report

As the implementer's text asks, with how the tests failed on the old code.
