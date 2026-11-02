# live-temper: the contracts

The binding calls the plan's three tasks build to; each builds its half without the others.
The request is `temper/docs/plan/live-temper/inputs/request.md`.

## 1. What the profile selects

`[profile.live-temper]` in the root `.config/nextest.toml` keeps its name, `global-timeout =
"50m"`, `test-threads = 1` and `retries = 0`. Its `default-filter` selects exactly the two
owned-forge landing stories of `temper`'s `live` binary:

```toml
default-filter = "(binary(=live) & (test(=a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge) | test(=a_person_asks_claude_code_to_land_a_change_on_the_owned_forge))) & (package(/^temper(-|$)/) | package(=git-transcripts))"
```

Both are `#[ignore]`d, so the profile alone runs nothing; fab's after-landing run with
`--live` adds `--run-ignored only` and runs the pair. `cargo nextest list -p temper
--profile live-temper --run-ignored only` lists exactly these two.

`tools/tiers/packages.toml` is unchanged: `live-temper = ["temper"]`, and its `[binaries]`
entry stays `[]`. That table names Cargo `[[bin]]` targets the pair's `-p` selection does
not build (`tools/tiers/design.md`, 2.1); `live` is a `[[test]]` target, and the stories'
`temper` binary is built by `-p temper` itself.

## 2. How the owner's scripts select their story

`temper/scripts/proof-live.sh` and `proof-live-github.sh` run over `--profile live-temper`
(the profile `live` they name today no longer exists; nextest reads only the root
configuration). Their build is `--profile live-temper --no-run`; their run adds
`--ignore-default-filter` and names its own story with
`-E 'binary(=B) & test(=NAME)'` (`B` `live` or `live_github`), as `rung-b0-offline` already
ignores the default filter. So the person-only stories stay runnable by their owner although
the profile's filter no longer names them.

## 3. A store per story

`TEMPER_LIVE_STORE` names the directory that holds the owned-forge landing stories' stores.
Each story takes its own store beneath it, named by `scripts/proof-live.sh`'s argument for
it:

| Story | `LiveModel` | Store |
|---|---|---|
| `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` | `ChatGpt { .. }` | `$TEMPER_LIVE_STORE/forge-land` |
| `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` | `ClaudeCode { .. }` | `$TEMPER_LIVE_STORE/forge-land-claude` |

- The directory is made (0700, with its missing parents) when absent. Each story's store keeps
  today's rules (`reserve_live_store`, unchanged): absent or empty is fresh, made 0700; one
  that holds a deployment is kept and checked by metadata; a held ChatGPT record
  (`TEMPER_LIVE_ACCOUNT`) moves into a fresh store only.
- So one invocation with one fresh `TEMPER_LIVE_STORE` runs both stories, each on a fresh
  deployment, and a second story never reuses the first's.
- The rung (`rung_b0_a_person_agrees_a_plan_before_the_change_lands`, `forge_land::rung`) is
  unchanged: `TEMPER_LIVE_STORE` is still its own store, through `live_store(true)`.
- `scripts/proof-live.sh forge-land` and `forge-land-claude`: the directory is
  `${TEMPER_LIVE_STORE:-$HOME/src/rust/tmp/live-store}`, exported as `TEMPER_LIVE_STORE`;
  the script's own `$store` (its rerun check, the give-back hook, the processes it stops,
  `kept store` line) is `<directory>/$1`. The owner's default stores keep their paths
  (`$HOME/src/rust/tmp/live-store/forge-land` and `.../forge-land-claude`). `rung-b0` is
  unchanged.
- The give-back hook stays in the story's store: `$TEMPER_LIVE_STORE/forge-land/give-back`.

## 4. The models

The cheapest models for automatic live runs (the user, 11-01): `LiveModel::name` is
`gpt-6-luna` for ChatGPT (unchanged) and `claude-haiku-5-5` for Claude Code (today
`"sonnet"`). The name reaches the agent's endpoint model, the engine's model and provider,
every charter and `chat start --model`, as `"sonnet"` does today.

## 5. What an after-landing run gives the pair (fab's side, not this plan's)

The design text states it; fab's runner, in the agents repository, provides it after this
plan lands:

- a fresh `TEMPER_LIVE_STORE`; a private `TMPDIR` (`support::Scratch::live` needs one);
- `TEMPER_LIVE_ACCOUNT`, a held ChatGPT record, a 0600 regular file;
- `TEMPER_LIVE_CLAUDE`, a held Claude Code login directory, and `claude` on `PATH`;
- after the invocation, whatever its end, as `proof-live.sh`'s exit trap does:
  `$TEMPER_LIVE_STORE/forge-land/give-back` run where it exists (the record goes back), both
  logins checked by `stat` alone, and any process left under the stores stopped.
