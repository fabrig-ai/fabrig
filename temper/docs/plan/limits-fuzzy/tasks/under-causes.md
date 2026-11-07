+++
id = "under-causes"
part = "under"
kind = "feature"
needs = ["public-wire", "one-process-sweeps", "engine-sweeps"]
files = [
  "temper/tests/real-loop/src/**",
  "skein/crates/skein-json/**",
  "skein/crates/skein-lib/src/bytes.rs",
  "skein/crates/skein-llm/**",
  "smith/crates/smith-protocol-llm/**",
  "temper/crates/temper-service/src/**",
  "temper/docs/plan/limits-fuzzy/figures/under-causes.md",
]
effort = "high"
minutes = 130
+++

# What the worlds share below them

## Goal

The causes below the worlds: what the three sweep tasks left under `## Left`, and what the real loop's remaining causes offer cheaply. Rank them by what they'd take out of temper's fuzzy suite, fix those that clear the bars below, and write down the rest with their figures. It may well find that little clears them: an honest *no* with its figures is a good ending, and nothing here gets weakened to find a gain.

The two bars:

- **A cause the sweep tasks left:** fix the largest, and stop when the next is under about 2% of a sweep.
- **A real-loop cause:** fix it only if the change is small and takes more than about 2 CPU s out of the suite, which is about 3.5% of that world.

Acceptance:

- `temper/docs/plan/limits-fuzzy/figures/under-causes.md` holds, in `notes/measuring.md`'s form:
  - each candidate with its share, its estimated CPU s out of the suite and the arithmetic behind that, and whether it was fixed or why not;
  - instructions and native CPU before and after each fix, for the sweeps its cause was named under and for the real-loop stories below, twice per arm for those;
  - what you stopped at.
- Each product change has its own focused test, cheap, in the package that holds it, and changes no behaviour.
- `heavy cargo nextest run` with each package you changed, and with the same packages `--profile fuzzy`, passes. So do those of `temper-real-loop-world`, `temper-engine-domain-world`, `temper-one-process-world` and `temper-codecs-world`, with the same tests by name as your base.
- `heavy cargo check -p <dependent> --tests` passes for each direct dependent of a changed public item, in every tree.

## Inputs

- `temper/docs/plan/limits-fuzzy/notes/common.md`, `temper/docs/plan/limits-fuzzy/notes/measuring.md`, including its real-loop section, and `temper/docs/plan/limits-fuzzy/notes/real-loop.md`.
- `## Left` and `## Stopped at` in `temper/docs/plan/limits-fuzzy/figures/public-wire.md`, `temper/docs/plan/limits-fuzzy/figures/one-process-sweeps.md` and `temper/docs/plan/limits-fuzzy/figures/engine-sweeps.md`. Those are your first list.
- `skein/docs/foundation/testing-strategy.md`, 8; `skein/docs/design/json.md`, 3.4 and 7, for what the parser and its string scan decided.

## Owns and may change

Now that the three sweep tasks have merged, a cause may sit in any crate under the worlds. Your `files` are where the known causes sit. A cause elsewhere under the worlds is yours too, at its bars: `temper/crates/**`, `jig/crates/**`, `smith/crates/**`, `skein/crates/**` and the kits in `temper/testing/**`, `jig/testing/**` and `skein/testing/**`.

Stay out of `temper/tests/engine/domain/**`, `temper/tests/one-process/**` and `temper/tests/codecs/**`, whose tasks have stopped: a call site there is a one-line edit, only where your signature change forces it. `notes/common.md`, *Beside this plan*, says what other drafts hold, and a cause in one of those files takes the smallest hunk that does it.

**Profile these three real-loop stories,** by the names in `notes/real-loop.md`, twice each, under raised patience, on your base. Use the sweep tasks' own *after* for the sweeps. Trace self-cost leaves to their callers: that's what `notes/real-loop.md` couldn't say.

Where to look first. Reading shows these; the profile decides whether any is the cause:

- **Who parses what, how often.** The footprint is in `skein-json`, and the callers decide what can go: `openai::Json::from_bytes` in `skein/crates/skein-llm/src/translate.rs`, `part` and `replay`, and `dialect.rs`, per block; a schema or configuration parsed again in the real loop's own fixtures (`temper/tests/real-loop/src/fixtures.rs`, `own_defaults`; `landing.rs`, `admit`, whose twelve admission checks stay). An input parsed twice, with nothing between them that could change it, can go. An independent input's validation stays. `replay` in `translate.rs` writes a small document, then parses what it wrote.
- **`observed_prompt`** in `smith/crates/smith-protocol-llm/src/component.rs` measures the whole prompt with `skein_llm::client::measure`, to refuse before it observes, and the send path measures it again. Handing the admitted measurement to the send path may be small now that the request lets a protocol change freely. It must still refuse before observing.
- **`skein_lib::bytes::find_from`:** its one-byte needle loops over `iter().enumerate().skip(from)`, and `TwoWay::find` runs unoptimized. `skein/docs/foundation/testing-strategy.md`, 8, prefers the library's own searches, which run optimized in unoptimized builds.
- **`skein_json::string::after_plain`** walks a slice pattern byte by byte (R 1.96%, U 2.50%, self). `skein/docs/design/json.md`, 7, chose that shape. A word-at-a-time scan would need a figure to beat it, in the strict subset (`skein/docs/foundation/programming-model.md`, 10).

Options, each decided:

- **A change that's small and clears its bar, but what its design section says would change:** it isn't small. Leave the code, and put it in your figures with what the design would have to say. `close` takes it to the user.
- **The writing pass's second measurement:** a trial saved 0.76% of U and was discarded. Don't try it again unless your profile shows the reason has changed, and then say what did.
- **Allocation's header layout, `core`'s checks and `next_policy_deadline`'s scan:** the checks and the layout stay. What could go is a copy or a scan's caller, if the profile shows one, at the same bars.
- **A dev-profile override for a hot leaf crate:** never here. It's the user's. If a profile shows a cost no test can avoid, say so in your report.
- **A real-loop story past its call's bound under callgrind,** as `notes/measuring.md` says: raise it locally and say which.

## Steps

1. Read the three figures files' `## Left`, profile the three stories on your base, and rank the candidates with their arithmetic. Write that table first.
2. Fix the largest that clears its bar, with its focused test, re-profile, repeat. Restore `PATIENCE` before every commit.
3. `cargo fmt`; `python3 -I -B <tree>/scripts/old_forms.py` for each tree you touched; `heavy cargo clippy -p PKG [-p PKG ...] --all-targets -- -D warnings`, naming every package you changed; the runs in the acceptance; commit and push.

## Report

Your ranked table; each fix, what it saved, and the rule its test bites on; what you left and why; each dependent checked; and your implementation and check minutes apart.
