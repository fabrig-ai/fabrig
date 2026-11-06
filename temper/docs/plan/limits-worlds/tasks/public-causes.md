+++
id = "public-causes"
part = "public"
kind = "feature"
needs = ["figures-before"]
files = [
  "temper/tests/one-process/**",
  "temper/docs/plan/limits-worlds/figures/public-causes.md",
]
effort = "high"
minutes = 90
+++

# Why one public walk costs seconds

## Goal

`public::every_inaccessible_identifier_is_one_unknown_byte_for_byte` is the focused suite's largest test at 4.7 CPU s, against the one-process story's 2 s guide (`temper/docs/design/domain/testing.md`, 6.1). Fix what the profile puts in `temper/tests/one-process/**`, largest first, until the next one's gain is under about 2% of the story. The one-process world's other tests share its fixtures, so a cause here may pay in them too. Everything this walk compares stays compared, byte for byte.

Acceptance:

- `temper/docs/plan/limits-worlds/figures/public-causes.md` holds, in `notes/measuring.md`'s form, instructions before and after for that test, for `public::every_owner_version_and_kind_answers_as_declared`, and for `m15::whole_run_and_renewal_scan_every_fake_issued_token_byte`.
- Each cause fixed says what it was and what it saved, and the file ends with what you stopped at.
- `heavy cargo nextest run -p temper-one-process-world` and the same `--profile fuzzy` pass, with the same tests by name.
- What each changed test checks, it still checks: your report says for each how you know, and for a shared setup, which assertion would have failed if the sharing broke it.

## Inputs

- `temper/docs/plan/limits-worlds/notes/common.md`, `temper/docs/plan/limits-worlds/notes/measuring.md`.
- `temper/docs/plan/limits-worlds/figures/figures-before.md`, this test's sections: your *before*, and where to look.
- `temper/docs/design/protocol/sign-in.md`, 2.5, for what the walks promise; `temper/docs/design/domain/testing.md`, 2.5 and 6.1.
- `temper/tests/one-process/src/public.rs`, `src/public/service.rs`, `src/public/identifiers.rs`, `src/public/owners.rs`.

## Owns and may change

Owns `temper/tests/one-process/**` and the figures file. A cause under this world is `engine-causes`' or another task's: name it under `## Left`, with its figures.

Where to look first, and what you decide. The profile decides whether any of these is the cause:

- **`public::walk` runs a whole deployment three times for this one test:** once tracked, under `every_owner_version_and_kind_answers_as_declared`'s walk, and then untracked, with `identifiers::PROXIED` false and true (`src/public.rs`). The first run also stands as a test of its own. If the deployment's cost is the largest share, the likely shape is one deployment for what the two untracked runs compare. Default: do that if both arms' compares stay byte for byte at one clock instant, and keep the first run unless you can show from the walk what it gives. If `identifiers::remember` is all that the later runs read from it, that's a cheaper way to get the identifiers. If `TRACKED`'s difference in the agent's delegation forces a deployment per value, keep the runs apart, and say so.
- **Fixtures rebuilt per run or re-encoded:** a fixture built once per test, not per case (`skein/docs/foundation/testing-strategy.md`, 8).
- **`Host::iterate` polls and scans in `src/public/service_checks.rs`:** a scan of what's been sent where a lookup or a position would do.

Options, each decided:

- **Sharing would merge two tests:** don't. A test past its guide isn't merged with another or named an exception (`temper/docs/design/domain/testing.md`, 6.1). Sharing within one test's body is fine.
- **The cause is in the service, its engine or a fake:** `## Left`.
- **The test stays past its guide after the fixes:** say so with its figures; `close` takes it to the user. Don't weaken it to get there.

## Steps

1. Profile the three tests on your base, as `notes/measuring.md` says, once each: simulated time, no raised patience.
2. Fix the largest, re-profile, repeat.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-one-process-world --all-targets -- -D warnings`; the runs in the acceptance; commit and push.

## Report

Instructions and CPU seconds before and after, each cause and what it saved, what you left and whose it is, and your implementation and check minutes apart.
