# macos-cheap: temper's acceptance story on macOS, cheaply

At the user's request (2026-11-04): carry out `temper/docs/design/draft/macos.md`, as revised and reviewed twice by Codex. **The user's rule, at the top of the draft, governs every choice: "make sure that anything we do is relatively cheap".** Linux-only things may be degraded or less sophisticated on macOS. The only important thing is that the acceptance story basically works; it doesn't need to be perfect, and rough edges are fine.

## Acceptance (the draft's 2.4)

On GitHub's macos-26 runner, the existing process test `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart`: a box with its owned Forgejo and a fake LLM, driven through `temper ctl`. A change is asked for in a chat, Forgejo runs CI, temper lands it.

## The scope: the draft's section 6, steps 1 to 5

1. **The pin bump** to Forgejo 17.0.1 and runner 13.2.1, the versions Homebrew bottles. Conformance is the check, since no release notes were found. Builders need the new binaries through `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER`: if that takes a machine action, end with a `Question:` naming it, and it goes to ops.
2. **`.github/workflows/macos.yml`** compiling every target, starting from skein's former workflow at `44c5be34fa`. It runs on the newest main a few times a day and blocks nothing. It names no tool, machine or session of whoever runs this development (`AGENTS.md` 1; `tests/boundary/tests/outward.rs`).
3. **`temper forge fetch` on macOS** (4.3): Homebrew's bottles on ghcr.io at addresses fixed by the pins' digests, Homebrew as a URL and nothing more.
4. **What the story needs at run time, and no more:** the `/proc/self/fd` pipe check may simply be Linux's; smith's rg and git paths; the test's `parent::Guard` and its `base64 -w0`.
5. **The story green on the runner.**

**Steps 6 to 8 (the suites, a real Mac, the browser demo) are not this plan's.** A test that would take real work goes on its tree's Linux-only list.

## Context

- **Nothing has run on a Mac for smith, jig or temper.** skein's focused suite reached 6,020 tests green on macos-26 on 10-23, and its code is still there (skein-shell's darwin module).
- **Two assumptions are unverified:**
  - that ghcr.io answers with exactly one redirect (4.3);
  - that the bottle binaries run standalone: Homebrew marks them `any_skip_relocation` with no formula dependencies.
  
  The first run shows. Design for finding out quickly, not for handling every case.
- **No builder runs macOS.** A task can only prove step 5 by the workflow on GitHub, after landing. So slice the work so that the workflow lands early, and failures from its runs come back as small follow-up tasks. A task's acceptance on the Linux builders is that Linux stays green and the macOS code compiles for the target where it can.
- **Reading the workflow's results into after-landing** is separate work, not this plan's. The repository is public, so results can be read with no token.
- **skein and jig are shared with hearth:** keep changes there small, and name them in `plan.md`.
- **Budgets and timeouts are the user's** (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Size

Keep it small: a few tasks, not a large plan. One cut is at most 90 minutes. Release early.
