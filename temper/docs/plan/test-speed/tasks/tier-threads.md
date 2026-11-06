+++
id = "tier-threads"
part = "tier-threads"
kind = "feature"
needs = []
files = [
  ".config/nextest.toml",
  "temper/crates/temper/src/fake_backends.rs",
  "temper/crates/temper/src/tests/**",
  "temper/crates/temper/tests/fake_backends/box_link.rs",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/domain/testing.status.md",
  "temper/docs/help/box.md",
  "temper/docs/plan/test-speed/figures/tier-threads.md",
]
effort = "high"
minutes = 150
+++

# `process-temper` runs three tests at a time

## Goal

`process-temper` runs `test-threads = 1` on 8 cores, using 0.54 of a core. Three threads could save about 300 s of wall per run, with no CPU change. No reason for the setting was found: `070bf648b5` restored the profile with it and says none, and `domain/testing.md` names none (check again, with `git log -S` on the line). The known blocker is a fake box's ports: its first run prefers 8367 to 8371 and writes them fixed, so a walk that restarts its box is refused (`AddressInUse` in tier run `94fdd7b1647d`) when another has taken them. Give the fake box the kernel's ports, then raise the threads.

Acceptance:

- **The ports, red first:** a test beside `crates/temper/src/init/tests.rs`'s or `box_link.rs`'s shows a fresh fake box writes no port of 8367 to 8371 while they're free. Seen failing, then `fake_backends.rs`'s first run asks for `[0; 5]` where it now asks for those five. Stays whatever the threads do: *if it flakes and the cause isn't quick, keep the port fix and leave the threads, and report it.*
- **`process-temper`** has `test-threads = 3`. The wall limit, the `global-timeout` and the budgets are as they were.
- **A test that can't share a machine** runs in a named test group in `.config/nextest.toml`, with a comment naming why, as `temper-real-loop` has. Expected: walks that sign in to ChatGPT, whose redirect comes back to the fixed port 1455 (`crates/temper/src/fake_backends.rs`, `ordinary_ports`; `tests/fake_backends/walk/sign_in.rs`), in a group `temper-sign-in-callback` of `max-threads = 1`. Look for the others: a fixed port, a shared path outside `Scratch`, the Forgejo binaries' cache (`support::forge_land::cache_forgejo_binaries`). Grouping is the last resort: a free port where one will do.
- **Proven, three whole runs at three threads, and one on a loaded builder:** no new flake, and no test's CPU worse. Compare each test's CPU to the one-thread run of the same commit, by `cpu-summary`'s figures, with the noise named; a test more than 25% worse on all three runs is worse. `figures/tier-threads.md` holds all of it: wall, CPU, peak load, each failure's name and log line, what contention did (it may ease or worsen).
- **Loaded** means another tier or heavy build running on the machine as the run starts: say what was running. A starved test is a warning, as `testing-strategy.md`, 8.1 has it, and its figures are reported.
- **If it flakes** and the cause isn't quick, `test-threads` goes back to 1, with the port fix staying, `testing.status.md`'s entry says why, and the report says what flaked.
- `setup.status.md`'s 7.5 entry goes; `testing.status.md`'s first 2.6 sentence goes when the threads stand. Its second is `walk-waits`'.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.5 (its last paragraph) and 8.2, *the clients' listener's port*.
- `temper/docs/design/domain/testing.md`, 2.6 (*Three at a time*).
- `skein/docs/foundation/testing-strategy.md`, 2.10 and 8.1.
- `temper/docs/plan/test-speed/notes/common.md`.

## Owns and may change

Owns the files in the header; in `.config/nextest.toml` only `[profile.process-temper]`, its overrides and `[test-groups]`.

`temper/docs/help/box.md` may change (a departure from the implementer's text): where it says what ports a fake box has, within its byte bound. Line 60 is `temper init --box`'s and stays.

Readers and neighbours, and what's allowed there:

- `temper/crates/temper/tests/support/**`, `tests/process.rs`, `tests/fake_backends/**`: a port it names, or a shared path, made its own. `support::free_addresses` and `support::own_listener_files` are the way. `tests/fake_backends/walk/{goal,away,status}.rs` are `walk-waits`' and `walk/sign_in.rs` is partly `sign-in-new-tab`'s: keep to the lines that bind.
- A help reference or golden the port change makes stale: regenerate by its own tool.
- `temper/tests/client-browser/**` reads the printed link, not the port. Its tier isn't this task's: leave it, and if it does name a port, report that under `## Left`.
- A committed journey's recording holds its old box's ports and replays as it stands. `add` and `again` are unaffected.

Options, each decided:

- If three threads fail but two hold, take two, and report the figures for each.
- If a first-run port of the kernel's can't serve something that needs it fixed (a registration, the fake GitHub's), give that one port its fixed preference back, and name it in the setup status.
- The other tier profiles' threads aren't this task's, nor is any `browser-*` profile.
- `temper init` and a person's `temper box` keep preferring 8367 to 8371.

This changes what a fake box's ports mean: run the tests of the packages that read them (`temper`, `temper-fake`, `temper-playground`, `temper-box`), and check by `rg` for readers of the five numbers.

## Steps

1. Look in git history and `domain/testing.md` for a reason; say what you found.
2. Measure the tier at the base, at one thread.
3. The port test, then the fix.
4. At three threads, on one slot: a run, its failures read and each cause repaired or grouped; then the three proving runs and the loaded one.
5. Write the figures.
6. `cargo fmt`, `old_forms.py`, clippy on the packages changed, the tests. Commit and push.

## Report

What history said; the ports fix; each group and why; the figures at one and at three threads; flakes, with their logs; whether the threads stand.
