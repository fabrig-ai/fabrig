+++
id = "story-step"
part = "story"
kind = "feature"
needs = ["workflow", "fetch", "habits"]
files = [".github/workflows/macos.yml", "skein/docs/design/forge.status.md", "temper/docs/design/protocol/setup.status.md", "temper/docs/design/domain/testing.status.md"]
effort = "medium"
minutes = 45
+++

# The workflow runs the one story

## Goal

The workflow fetches the pinned binaries through `temper forge fetch`, shows each one runs, and runs the acceptance story alone, so that its first run on `main` answers whether the story holds on macOS. This is the plan's last task; the story green is the runner's to show, after landing.

Acceptance:

- the workflow's new steps, in the order below, with the seam of `notes/contracts.md`;
- the fetch's shell, run by hand on a Linux builder with the Linux binaries, ends with both variables' files in place and their versions printed;
- `cargo nextest run -p fabrig-boundary` passes;
- a Darwin compile, by `habits`' step 5, still ends clean over what `fetch` and `habits` left.

## Inputs

- `temper/docs/design/domain/testing.md`, 2.8.
- `temper/docs/design/protocol/setup.md`, 6.6.
- `temper/docs/plan/macos-cheap/notes/common.md`; `temper/docs/plan/macos-cheap/notes/contracts.md`, "The workflow's seam".
- `tools/tiers/packages.toml`, `process-temper`'s binaries.

## Owns and may change

Owns `.github/workflows/macos.yml`.

Pre-approved:

- **It fetches,** only in the hand run of step 3, through `temper forge fetch` itself, on the builder.
- The stand-in of `habits`, step 5, as that spec has it, untracked and gone before the push.
- If the hand run shows `temper init` writes no owned forge a fetch can read, the same document's `application.machine.owned` is what to give it: adjust the shell, not the code.
- A Darwin compile failure from `fetch` or `habits`' code, one `cfg` to fix: fix it. More: under `## Left`.

Nothing else in the file changes.

A departure from the implementer's text: this last task leaves the status files true for the landing. It removes the `macos-cheap` entries that the plan's tasks closed, each its own `- **Not yet:**` line: from `skein/docs/design/forge.status.md`, the line on 4.3, unless `conformance` left its facts open; from `temper/docs/design/protocol/setup.status.md`, the line on 6.1, 6.6, 7.2 and 8.2; and in `temper/docs/design/domain/testing.status.md`, the line on 2.8 becomes "2.8: the one story has not run green on macOS yet."

## Steps

1. After `Save the target`, before `The disk`, each under the seam's `if`, `id: fetch` on the first:

   ```yaml
   - name: The pinned binaries, through temper forge fetch
     id: fetch
     run: |
       $RUN build cargo build -p temper --bin temper
       $RUN build-fake-claude cargo build -p smith-fake-claude --bin fake-claude
       root="$RUNNER_TEMP/fetch"
       mkdir -m 700 "$root" "$root/config" "$root/data" "$root/cache"
       flags=(--config "$root/config" --data "$root/data" --cache "$root/cache")
       $RUN init target/debug/temper init "${flags[@]}"
       mkdir -p "$root/data/box/groups"
       printf 'owned\n' >"$root/data/box/groups/1"
       $RUN fetch target/debug/temper forge fetch "${flags[@]}" 1
       forgejo=$(echo "$root"/cache/forgejo/*/forgejo)
       runner=$(echo "$root"/cache/forgejo-runner/*/forgejo-runner)
       $RUN forgejo-runs "$forgejo" --version
       $RUN runner-runs "$runner" --version
       echo "TEMPER_FORGEJO=$forgejo" >>"$GITHUB_ENV"
       echo "TEMPER_FORGEJO_RUNNER=$runner" >>"$GITHUB_ENV"
       echo "TEMPER_TEST_CLAUDE=$PWD/target/debug/fake-claude" >>"$GITHUB_ENV"

   - name: The one story
     if: ${{ !cancelled() && steps.fetch.outcome == 'success' }}
     run: |
       $RUN story cargo nextest run --profile process-temper -p temper --test process \
           -E 'test(=a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart)'
   ```

   Take the flags' spelling from `temper init --help` and `support::configuration::flags`, if they differ from the sketch.
2. The file's opening comment says it runs the one story too.
3. Hand run on the builder: the block of the first step, with `RUNNER_TEMP` a directory under `target/`, the two `$RUN` wrapped by nothing, `GITHUB_ENV` a file there, and `TEMPER_BUNDLE=none`, each cargo through `heavy`. Delete it after.
4. A Darwin compile as in `habits`, step 5.
5. The status lines above, then `python3 -I -B tools/citations/citations.py --base origin/main`.
6. `heavy cargo nextest run -p fabrig-boundary`. Commit and push.

## Report

As the implementer's text has it, with the hand run's two version lines.
