# figures-before: baseline measurement blocked before native execution

2026-11-06 UTC, base commit `150688094b66d261f210311968a6500a760176be`,
16 CPUs, AMD EPYC-Genoa Processor, Valgrind 3.24.0.
Test binaries were built with the unoptimized test profile.

`/usr/bin/time` is absent on the builder. The first native command exited
127 before starting libtest: `/usr/bin/bash: line 1: /usr/bin/time: No such
file or directory`. The required descendant-inclusive native CPU measurement
could not run. No story has a measured result, and no instruction counts or
causes can yet be reported. `PATIENCE` remains unchanged at 20 seconds.

The implementer's missing-tool rule requires a question; the private-package
download exception in `notes/common.md` covers Valgrind alone. The proposed
next step is to download and extract GNU time privately under
`target/limits-worlds/tool/`, without installing it on the builder, then resume
`notes/measuring.md`'s native and Callgrind measurements.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| `temper-real-loop-world`: `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` | before | — | — | — | Unobserved; measurement tool missing |
| `temper-real-loop-world`: `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` | before | — | — | — | Unobserved |
| `temper-real-loop-world`: `github_land::a_chats_change_lands_on_connected_github` | before | — | — | — | Unobserved |
| `temper-conformance-world`: `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` | before | — | — | — | Unobserved |
| `temper-one-process-world`: `public::every_inaccessible_identifier_is_one_unknown_byte_for_byte` | before | — | — | — | Unobserved |

## Left

All five native measurements, eight Callgrind runs (two per real-loop story),
inclusive and self function tables, self-instructions by crate, child spawn
counts, slowdown corrections and the ranked causes await the measurement tool.
No product or test code changed.
