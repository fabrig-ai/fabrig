# design: status

2026-11-14. Where the repository lags or departs from [design.md](design.md).

Section 3's `skein-fork` takes the `ring` profile; its documented process
boundary scopes its `unsafe` expectation. No gap remains in that addition.

Section 4's ban of the process's arguments and environment is in all three variants, and every
read in the four trees' packages is under a scoped expectation whose reason starts with what it is
or the round or milestone that removes it (temper's, the milestones C2 to C6 of
`temper/docs/design/draft/cli.execution.md`), but for the seven of the last departure below.

- **Not yet:** 5: `every_package_takes_a_lint_profile` (`tests/boundary/tests/lints.rs`), which
  `.config/gate/always.txt` selects on every change, is the check's one run in this repository.
  The user's request of 2026-11-04 has it run beside `tools/citations/citations.py`, in a step this
  repository doesn't hold: whether that step has it awaits the user (8). `.config/gate/README.md`, 8
  also names `tools/lints/test_lints.py`, which no test here runs, so a break of the check itself
  shows only where someone runs it.
  7: the count of catches: `tools/lints/panics.py`, its list `tools/lints/panics.txt` at main's
  figures, its tests, and its runs in the gate and in `fabrig-boundary`. [in flight: change 2674]
- **Departs:**
  - 1, 2.4: four profiles, where the request and its debt items name three, because `binary` held
    two regimes and one block for both would loosen the ring's shell or raise twelve crates. The
    user hasn't answered (8).
  - 2.3: `tools/lints/profiles/binary.toml`'s first line, and so the block of each of the 12
    packages on `binary`, cites section 2.2, `harness`'s; no reason is recorded.
  - 4: the request has each `clippy.toml` carry a true header, but `ring`'s says what a crate on the
    ring or one that plays the kernel may not use, and its copies in the five kits this section
    names are neither; no reason is recorded. Each kit under `testing/` has a copy, where the debt
    item proposed one `testing/clippy.toml`, because step crates sit beside the kits there and
    clippy would hand them the looser bans.
  - 4: seven expectations in skein and jig that predate the ban keep a reason that doesn't start
    with what the read is: `skein-shell`'s `listener_tests.rs` (two), `skein/tests/llm`'s
    `live.rs`, `skein/tests/ui-web`'s `harness_fixture.rs`, `jig-test`'s `tests/support/mod.rs`
    and `jig/tests/core`'s fuzzy `settings.rs` (two). The ban's sweep left the other trees' existing
    wording to their rounds (`skein/docs/foundation/cli.md`, 10).
