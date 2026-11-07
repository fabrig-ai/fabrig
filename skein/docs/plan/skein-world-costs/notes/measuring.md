# skein-world-costs: how a test is measured

Callgrind, on one test by name, in instruction counts: a loaded machine doesn't move them, while CPU seconds on these machines swing by a third or more between runs. Every figure this plan reports is a reading made this way.

## The six tests

| Package | Test | Profile |
|---|---|---|
| `jig-core-world` | a median one, chosen by `counting` | `default` |
| `jig-client-protocol-world` | `client::account_keys::a_key_ask_carries_its_value_into_the_secrets_store` | `default` |
| `jig-ops-world` | a median one, chosen by `counting` | `fuzzy` |
| `skein-forge-machine-world` | a median one, chosen by `counting` | `default` |
| `skein-llm-world` | a median one, chosen by `counting` | `default` |
| `skein-tls-world` | a median one, chosen by `counting` | `default` |

`counting` chooses from one run of the package's suite in that profile, by the times nextest prints with `--status-level all`: the test in the middle of the passing tests. It writes the six names at the head of `figures/counting.md`, and every later reading uses those names. Only `counting` runs a whole suite for this; a median need not be exact.

## One reading

From the slot's root:

```sh
heavy cargo nextest list -p PKG --profile PROFILE --message-format json
```

That builds the package, and gives each test binary's `binary-path` and `cwd`. Then run the test's binary by its exact name, in its `cwd`, as nextest would:

```sh
mkdir -p "$ROOT/target/world-costs"
cd "$CWD"
CARGO_MANIFEST_DIR="$CWD" heavy valgrind --tool=callgrind \
  --callgrind-out-file="$ROOT/target/world-costs/LABEL.out" \
  "$BINARY" --exact TEST --nocapture
```

- **Check the count:** the output must say one test ran and passed. A run that selected nothing proves nothing, and a failing test's counts are no figure.
- **A variable a test reads:** `skein_scratch::paths` names any it misses in its failure (`CARGO_BIN_EXE_<name>`, for one). Set it as nextest would.
- **Read it** with `callgrind_annotate --inclusive=yes "$ROOT/target/world-costs/LABEL.out"`, and `--tree=calling` for who calls what. `PROGRAM TOTALS` is the test's total. A cause's figure is its inclusive count, and its share is that over the total.
- **`valgrind` missing from your machine** is a tool the task lacks: `Question:`, as the implementer's text says. Don't install anything.
- **One reading a figure.** These tests run on fixed seeds, so a second reading of the same binary differs by a fraction of a percent. If two readings of one binary differ by more than 1%, say so: something in the test isn't fixed.
- **Delete `target/world-costs/`** once your figures are written: it's yours, and it isn't committed.

## What's written down

Write your figures to `skein/docs/plan/skein-world-costs/figures/<your task>.md`, a file of its own, so that tasks running side by side never meet in one, and commit it:

```markdown
# <task>: <what these readings measure>

<date -u>, base <short sha>, after <short sha>, <nproc> CPUs, <lscpu's model name>, valgrind <version>.

| Package | Test | Arm | Total Ir | Cause | Cause's inclusive Ir | Share |
|---|---|---|---:|---|---:|---:|
```

Name the machine by its CPU count and model only, never by its host name. `world-helpers` carries these figures into `skein/docs/development/world-costs.md`.
