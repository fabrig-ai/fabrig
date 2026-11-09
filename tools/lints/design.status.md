# design: status

2026-11-09. Where the repository lags or departs from [design.md](design.md).

Section 3's `skein-fork` takes the `ring` profile; its documented process
boundary scopes its `unsafe` expectation. No gap remains in that addition.

- **Not yet:** 4: the variants list `std::env::var` alone, `step` none of the environment's functions,
  and no read says what it is.
  5: `every_package_takes_a_lint_profile` (`tests/boundary/tests/lints.rs`), which
  `.config/gate/always.txt` selects on every change, is the check's one run in this repository.
  The user's request of 2026-11-04 has it run beside `tools/citations/citations.py`, in a step this
  repository doesn't hold: whether that step has it awaits the user (7). `.config/gate/README.md`, 8
  also names `tools/lints/test_lints.py`, which no test here runs, so a break of the check itself
  shows only where someone runs it.
- **Departs:**
  - 1, 2.4: four profiles, where the request and its debt items name three, because `binary` held
    two regimes and one block for both would loosen the ring's shell or raise twelve crates. The
    user hasn't answered (7).
  - 2.3: `tools/lints/profiles/binary.toml`'s first line, and so the block of each of the 12
    packages on `binary`, cites section 2.2, `harness`'s; no reason is recorded.
  - 4: the request has each `clippy.toml` carry a true header, but `ring`'s says what a crate on the
    ring or one that plays the kernel may not use, and its copies in the five kits this section
    names are neither; no reason is recorded. Each kit under `testing/` has a copy, where the debt
    item proposed one `testing/clippy.toml`, because step crates sit beside the kits there and
    clippy would hand them the looser bans.
