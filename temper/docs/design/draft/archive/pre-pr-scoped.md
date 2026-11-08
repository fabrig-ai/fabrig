# pre-pr-scoped: a delivery's checks cover what it changes

`.temper/pre-pr` is the project's check at each delivery (`temper/docs/design/protocol/configuration.md`, 13.9; smith's `docs/design/domain/run.md`, 8.1). Today it runs `cargo fmt --all --check` and all five trees' `scripts/old_forms.py` on every delivery, a documents-only one included. On a real deployment's worker that took about 90 s: fmt about 70 s wall (26 s CPU) and the five scripts about 17 s. The worker's check bound is 60 s, so a goal's three design deliveries timed out, `exit=timed-out given_ms=60000`, with no output. A goal couldn't deliver its design first, as `temper/docs/design/draft/goal-as-designer.md` has it. The bound is a figure that stays. The pipeline we develop with checks only what an implementer changed, and this does the same. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **The failing test first,** where the repository's scripts are tested, as a table of rows:
   - a documents-only delivery runs no format check and no `old_forms.py`, and passes;
   - a delivery changing one crate's `.rs` file formats only the changed Rust files, counts old forms for that tree only, and passes;
   - a misformatted changed `.rs` file still fails;
   - a raised old-form count in a changed tree still fails;
   - a tree that passed passes again without rerunning, as the cache does today.
   Say what each row costs and keep it cheap, with no real `cargo` build in a test if a fake binary on `PATH` will hold it.

2. **`.temper/pre-pr` checks the delivery's increment:**
   - **The paths:** those the delivery adds over the checkout's `HEAD`: staged, unstaged and untracked, with a rename's new path. Earlier deliveries passed their own checks, so the increment is what's left.
   - **Format:** only the Rust files among those paths, under the workspace's rustfmt settings (`rustfmt --check` with the workspace's edition and config, or `cargo fmt --check -- <files>`, whichever honours them). No format run when no `.rs` path changed.
   - **`old_forms.py`:** only for the trees among those paths (`skein`, `smith`, `jig`, `temper`, `hearth`).
   - **The cache:** the existing `pre-pr-passed` check stays.
   Say in a comment why it's scoped. Measure a documents-only and a one-crate delivery on a real checkout, and say their times.

3. **The design says so:** `configuration.md` 13.9, and wherever else it says what `.temper/pre-pr` runs, in the same words.

4. **The comparisons' notes:** `temper/docs/design/draft/dogfood-hacks.md`, beside 2.9, says a replay at an older base carries this `.temper/pre-pr` in its seed commit, beside the `ci.yml`, so its deliveries are checked the same way.

**A cost to avoid:** the workspace holds generated codecs of over 100k lines, such as `jig/crates/jig-client-protocol-wire/src/generated/v19.rs`. Running rustfmt over the whole workspace to survey the format check's reach took over 3 GB of memory and was killed. Survey by reading `.temper/pre-pr`, `rustfmt.toml` and the workspace's manifests instead, and when a test needs a format run, use a small fixture. Say in your report whether the scoped check could ever be handed a generated file, and what it does then.

## Not in this plan

The worker's check bound, and any figure. Main's CI, which is the deployment's.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Failing tests first; no test reaches a real host. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** `walk = false`.
