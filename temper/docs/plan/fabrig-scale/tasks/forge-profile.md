+++
id = "forge-profile"
part = "forge-scale"
kind = "feature"
needs = ["forge-questions", "override-relations"]
files = ["temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/startup/application.rs", "temper/crates/temper-service/src/tests/limits.rs", "temper/crates/temper-service/src/tests/startup.rs", "temper/docs/design/protocol/configuration.status.md", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 120
+++

# temper's application `standard` profile: the forge's work at fabrig's scale

## Goal

temper's application limits gain a second profile, `standard`, beside `small`, holding the
forge's work of a project of fabrig's scale (contracts K1 and K4;
`temper/docs/design/protocol/configuration.md`, section 7, "Two profiles, as jig's"): 128 live
tasks, holds and changes the forge follows, 32 landings, a change's 80 questions, tasks,
repairs, resolutions and updates, an issue plan's 384 items. A document's
`application.limits.profile` may name it. Nothing names it yet: `init-standard` makes init's
document do so.

Acceptance:

- `temper_service::limits::Profile` gains `Standard`; `limits::profile(Profile::Standard)` is
  small's `Limits` with, and only with (but what the next bullet forces):
  `forge.tasks`, `forge.holds`, `forge.changes` 128; `forge.client.resources` 128;
  `forge.landings` 32; `forge.change_policy.questions`, `.tasks`, `.repairs`, `.resolutions`,
  `.updates` 80; `forge.issue_policy.plan_items` 384. Write it as `const`s built with
  `..FORGE` / `..small's` struct update, beside small's, each new figure commented with what it
  holds (the draft's row: "live changes in a project, 18 seen", and so on: configuration.md 13.4's
  table), and the module doc naming both profiles.
- **Coupled limits, the least that passes:** run `profile(Standard)` through each domain's
  validity as `the_small_profile_passes_each_domains_validity` does for small, and through
  `limits::engine` with jig's `standard` and with jig's `small`; where a check refuses (a
  forge's `output` or `facts` per step, the landing's `landing_members` or `gates`, a store
  bound of temper's from these limits), raise that limit in `standard` alone to the least that
  passes, and list each in the report with the check that asked.
- `startup/application.rs`, `configured_limits`: `"standard"` selects `limits::profile(Profile::Standard)`,
  `"small"` small's, another name `What::Profile` as today; overrides apply over the selected
  profile.
- Tests in `src/tests/limits.rs`:
  - `the_standard_profile_holds_fabrig_scale_work`: each figure of the first bullet, and
    every other field equal to small's but those the report lists;
  - `the_standard_profile_passes_each_domains_validity` (as small's test);
  - `the_standard_profile_is_read_by_name`: an application limits member
    `{"profile": "standard", "overrides": {}}` gives `profile(Standard)`, and
    `{"profile": "large", ...}` is refused `What::Profile` at `profile` (build the document as
    the startup tests near it do).
- **The override refusal through temper:** in `src/tests/startup.rs`,
  `capacity_overrides_name_the_failed_dependency` gains two rows over init's generated document:
  `(b"tasks.message_bytes", 131072, &["tasks.message_bytes", "131072", "tasks.inbox_bytes", "131072"])`
  and `(b"authority.grants", 15, &["authority.grants", "15", "notes.scopes", "18"])`; each
  passes its loop's assertions (no "invalid override", one line).
- `temper/docs/design/protocol/configuration.status.md`: from its first **Not yet** entry, the
  clause "temper's application limits parse `small` alone (`temper-service/src/startup/application.rs`,
  `src/limits.rs`);" removed. `temper/docs/design/domain/forge.status.md`: its 8.11 entry
  removed whole (`forge-questions` built its first half, this task its second).
- `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::limits::|tests::startup::capacity/)'`
  passes; then `-p temper-service`, `-p temper-engine-domain-forge` and `heavy cargo check -p
  temper --tests`.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K1, K4 and K7.
- `temper/docs/design/protocol/configuration.md`, section 7 ("Two profiles, as jig's") and 13.4
  (the table in "The box's limits for that work").
- `temper/docs/design/domain/forge.md`, the resource capacity's bullet ("The stock resource
  capacity is 16") and 8.11 ("Bounded").

## Owns and may change

- The files of the header, for what acceptance names.
- `src/limits/names.rs` maps override names to fields; no new name is needed. If one is, stop
  and ask.
- Other places that build `Profile::Small` (`worker.rs`, `fake_backends.rs`, the playground's,
  `temper-fake`'s) stay: they are small's by design.
- The brief's budgets stay small's in `standard` (task `brief-room` sizes them, contracts K8).
- `temper/docs/help/` is not yours.

## Steps

1. The profile and its tests; run them alone.
2. The coupled limits, by the checks; the parse; its test.
3. The two startup rows.
4. The status lines; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-service --all-targets -- -D warnings`; the citations check; commit and push.

## Report

As the implementer's text asks, with each coupled limit raised and why.
