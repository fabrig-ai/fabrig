# live-temper: the plan

From `inputs/request.md` (the user's decision, 11-02): run the two owned-forge landing
stories after every landing on shelf logins, under fab's `--live` with `--run-ignored only`;
keep every other live story the owner's. One direct part, three independent tasks, each built
to `notes/contracts.md`; shared rules in `notes/common.md`.

## Parts and tasks

- **live-pair** (`temper/docs/design/domain/testing.md, 2.7`), slice 1.
  - `tasks/pair-profile.md`, medium, ~20 minutes, needs nothing: `[profile.live-temper]`'s
    default filter becomes the `live` binary's two landing stories; `nextest list` counts
    (2 under `--run-ignored only`, 0 without, the same 2 over the workspace).
  - `tasks/pair-stories.md`, high, ~70 minutes, needs nothing: each landing story takes its
    own store under `TEMPER_LIVE_STORE` (`forge-land`, `forge-land-claude`), the Claude story
    asks for `claude-haiku-5-5`, `proof-live.sh` exports the stores' directory, and both
    owner scripts move from the missing `live` profile to `live-temper` with
    `--ignore-default-filter`; suite tests for the stores and the launcher.
  - `tasks/pair-design.md`, medium, ~30 minutes, needs nothing (a `docs` task: it edits
    `testing.md` 2.7 and `testing.status.md`, the request's design text, given nearly whole
    in the spec).

## Critical path

`pair-stories` (~70 minutes, up to ~140 with the buffer), then fab's end check on the trunk.
The other two run beside it.

## Overlaps

None found. `sign-in-wait` changes smith's files only. No open `v0.1.0-ladder` or `qaladder`
branch touches `.config/nextest.toml`, `tools/tiers/`, the `live`/`suite` forge_land files,
the `proof-live` scripts or `testing.md`/`testing.status.md`.

## Calls made

- **The same profile, narrowed.** fab's `--live` runs pairs (`<tier>-<tree>` profiles), so
  `live-temper` keeps its name and is narrowed rather than joined by a new pair (which would
  need its own package map, binaries list and budget).
- **`tools/tiers/packages.toml` does not change.** The request asks for `[binaries]`'s
  `live-temper` (line 46) to name the `live` binary. That table names Cargo `[[bin]]`
  targets the pair's `-p` does not build, each built by `cargo build -p P --bin B`
  (`tools/tiers/design.md`, 2.1): `live` is a `[[test]]` target, so the build would fail and
  stop the tier, and the stories' one binary, `temper`, is built by `-p temper`. The
  profile's filter is what selects the binary.
- **A store per story, by the script's argument.** `TEMPER_LIVE_STORE` names the stores'
  directory for the two landing stories; each takes `forge-land` or `forge-land-claude`
  beneath it, so fab sets one fresh directory and runs the pair in one invocation. The
  launcher's default directory is `$HOME/src/rust/tmp/live-store`, so the owner's kept
  stores keep their paths. The rung keeps `TEMPER_LIVE_STORE` as its own store: the request
  keeps the person-only stories unchanged.
- **The owner's scripts were broken; they are fixed here.** `proof-live.sh` and
  `proof-live-github.sh` name `--profile live`, which left with temper's own nextest
  configuration (feb46b7e25, 10-28); nextest reads only the root's, which defines no
  `live` profile. Once `live-temper` is narrowed they also need `--ignore-default-filter` for the
  person-only stories. "Keep both runnable by the owner" needs both. jig's
  `live-sign-in.sh` has the same fault; jig's, left to its owner (`pair-stories` lists it).
- **The 42 duplicates:** none remain (the request's own finding, checked: 1425451fb7 is on
  main and `suite/support_tests.rs` holds 22 harness tests); the design text and status say
  so, and nothing is deleted.
- **What fab's runner owes the pair** (a private `TMPDIR`, the held record's give-back, the
  logins' `stat`, stopping what is left) goes into the design text: the stories take it
  from `proof-live.sh` today, and fab's LIVE runner replaces the script.
- **The question:** the profile's 50-minute global timeout against two 40-minute stories in
  one invocation is the user's (`plan.toml`, `pair-timeout`); default taken, 50 minutes
  unchanged.
