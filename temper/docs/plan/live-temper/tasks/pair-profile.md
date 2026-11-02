+++
id = "pair-profile"
part = "live-pair"
kind = "feature"
needs = []
files = [".config/nextest.toml"]
effort = "medium"
minutes = 20
+++

# The live-temper profile selects the owned-forge landing pair alone

## Goal

fab's after-landing run will run profile `live-temper` under `--run-ignored only`. Today its
filter, `binary(=live) | binary(=live_github)`, would then also run three stories that need a
person or github.com. Narrow it to the two owned-forge landing stories.

Acceptance:

- `[profile.live-temper]`'s `default-filter` is the one in
  `temper/docs/plan/live-temper/notes/contracts.md`, 1; its other keys unchanged.
- `heavy cargo nextest list -p temper --profile live-temper --run-ignored only` lists exactly
  two tests, both of binary `temper::live`:
  `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` and
  `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`.
- The same list without `--run-ignored` lists none (both are ignored).
- The same list over `--workspace` in place of `-p temper`, with `--run-ignored only`, lists
  the same two: the filter selects nothing outside temper.

## Inputs

- `temper/docs/plan/live-temper/notes/common.md` and `notes/contracts.md` (1, 2).
- `temper/docs/plan/live-temper/inputs/request.md`, "Select just the two".
- `tools/tiers/design.md`, 1 and 2.1: a pair is a `<tier>-<tree>` profile; why
  `tools/tiers/packages.toml` does not change.

## Owns and may change

- `.config/nextest.toml`: `[profile.live-temper]` (lines ~159-163) and the comment above the
  profile tables that names it, if it needs a word. Nothing else of the file.

Read only: `tools/tiers/packages.toml` (unchanged: `live-temper = ["temper"]`, its
`[binaries]` entry `[]`; contracts.md, 1 says why), `temper/crates/temper/tests/live.rs`
(the two names), `temper/scripts/proof-live.sh` (another task moves it onto this profile).

Pre-approved options:

- If nextest refuses the filter's form, keep its meaning (the `live` binary's two tests, in
  temper's packages) in the nearest form it accepts, and give the form in the report.
- If the workspace list (step 3) needs a build heavy refuses for disk (exit 75), stop and
  report `Infra: disk`; the `-p temper` lists stand.

## Steps

1. Replace `[profile.live-temper]`'s `default-filter` with contracts.md's, and give the table a
   one-line comment above it, in the file's register, e.g.:

   ```toml
   # After every landing, under --run-ignored only: the owned-forge landing pair on held
   # logins (temper/docs/design/domain/testing.md, 2.7). The owner's scripts ignore this
   # filter and name their own story.
   [profile.live-temper]
   ```

2. Lists, each through heavy, each count in the report:
   `heavy cargo nextest list -p temper --profile live-temper --run-ignored only`;
   `heavy cargo nextest list -p temper --profile live-temper`.
3. `heavy cargo nextest list --workspace --profile live-temper --run-ignored only` (builds the
   workspace's test binaries; one command, let it finish).
4. Run no test of the `live` binaries (notes/common.md). There is no Rust change: no fmt or
   clippy step.
5. One commit; push your branch.

## Report

As the implementer's text says, and also: the three lists' counts and the two names; the
nextest version (`cargo nextest --version`).
