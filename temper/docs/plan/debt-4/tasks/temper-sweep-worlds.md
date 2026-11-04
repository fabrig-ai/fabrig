+++
id = "temper-sweep-worlds"
part = "temper-forms"
kind = "migration"
needs = []
files = [
  "temper/tests/**",
]
effort = "medium"
minutes = 35
+++

# temper D003, D004: temper's worlds use `len_u64`, `usize_of` and `env.with`

## Goal

temper's worlds spell their widenings with skein-lib's helpers and hand their child steps their limits with `env.with`. The one `Env` rewrap on a `skein_shell::Now` is `temper-env-now`'s and stays. Both items stay open: `temper-sweep-paid` deletes them.

Acceptance:

- With `OWN="temper/tests"`, the three searches of `temper/docs/plan/debt-4/notes/temper-forms.md` read nothing, but for the third's one file below.
- The third search lists exactly `temper/tests/real-loop/src/github.rs` (1), on a `Now`, which `temper-env-now` owns; that file stays untouched here.
- `heavy cargo nextest run -p temper-engine-domain-world -p temper-worker-checkout-world -p temper-client-world -p temper-codecs-world -p temper-engine-protocol-forge-world -p temper-one-process-world -p temper-worker-domain-world -p temper-worker-protocol-git-world -p temper-system-world -p temper-real-loop-world` passes, every count above zero; no assertion of an existing test changes.
- `heavy cargo clippy -p temper-engine-domain-world -p temper-worker-checkout-world -p temper-client-world -p temper-codecs-world -p temper-engine-protocol-forge-world -p temper-one-process-world -p temper-worker-domain-world -p temper-worker-protocol-git-world -p temper-system-world -p temper-real-loop-world --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/temper-forms.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D003-widening-try-from.md` and `temper/docs/development/debt/D004-env-with.md`, whole.
- `skein/docs/design/lib.md`, 6 (the widenings) and 9 (`env.with`); `skein-lib/src/length.rs`, `skein-lib/src/env.rs`.

At base, `$OWN`'s sites by the note's searches:

| Package | `u64::try_from(..len())` | `"u32 fits usize"` | `Env` rewraps | Files |
|---|---|---|---|---|
| `temper-engine-domain-world` | 3 | 7 | 3 | 7 |
| `temper-worker-checkout-world` | 3 | 0 | 0 | 2 |
| `temper-client-world` | 2 | 0 | 1 | 3 |
| `temper-codecs-world` | 2 | 0 | 0 | 2 |
| `temper-engine-protocol-forge-world` | 1 | 0 | 2 | 2 |
| `temper-real-loop-world` | 1 | 0 | 1 | 2 |
| `temper-one-process-world` | 1 | 0 | 0 | 1 |
| `temper-worker-domain-world` | 1 | 0 | 0 | 1 |
| `temper-worker-protocol-git-world` | 1 | 0 | 0 | 1 |
| `temper-system-world` | 0 | 0 | 1 | 1 |

## Owns and may change

- Owns: `$OWN`'s Rust sources, for the forms the note names and nothing else.
- Not this one: `temper/tests/real-loop/src/github.rs`.
- Overlap: `tests/engine/domain/tests/suite/forge_routes/goal_tool.rs` holds 6 sites and `tests/client/src/front.rs` one; `fabrig-scale` changes both. Sweep them all the same; a one-line merge is fab's.
- The real-loop world's tests may need what your slot lacks; `heavy cargo check -p temper-real-loop-world --tests` stands for its run if so, and your report says which.
- Pre-approved: the note's `Pre-approved fallout`, each with its default.

## Steps

1. Sweep `$OWN` by the note's table, package by package, and read what the first two rows did not take.
2. Run the three searches; then what the note's `What runs` names.
3. Commit (`D003, D004: <packages> use skein-lib's helpers`, saying why), and push.

## Report

The counts before and after; each site the note's fallout touched; each `#[expect]` you added;
any site you left and why.

## Answers

### 2026-11-04: Should the sum in temper/tests/one-process/src/public/load.rs:77 stay with an acceptance-search exemption (recommended), or should this task replace its conversion with skein_lib::len_u64 despite the note's What stays rule? (debt-4/temper-sweep-worlds-1, foreman debt-4/temper-sweep-worlds/foreman-1)

Keep the sum unchanged and report it as an exemption from the first acceptance search.
`temper/docs/plan/debt-4/notes/temper-forms.md`, “What stays”: “A sum or difference of lengths” stays, explicitly including `u64::try_from(size_of::<T>() + x.len())`.
`temper/docs/plan/debt-4/notes/common.md`: “Acceptance searches are narrow: code that legitimately uses the same word for something else is exempted, not changed; your spec names the exemptions it found, and your report names any other you meet.”
The expression in `temper/tests/one-process/src/public/load.rs:77` converts an arithmetic sum and falls under that exclusion.
The next run leaves this expression unchanged, names this additional exemption in its report, completes the remaining sweep, runs the required checks, commits and pushes.
