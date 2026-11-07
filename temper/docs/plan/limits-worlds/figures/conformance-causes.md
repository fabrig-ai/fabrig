# conformance-causes: what each forked replay still costs

2026-11-07 UTC, base `456f45b727`, 16 CPUs, AMD EPYC-Genoa Processor,
valgrind 3.24.0. Unoptimized test builds; simulated time; original patience.
Each sweep's Ir sums PROGRAM TOTALS from its parent and every child.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` | Before | 1 | 25,170,053,373 | 6.918 | 1 passed; 78 children |
| `fuzzy::shared::a_changed_judge_is_crashed_at_every_live_commit` | Before | 1 | 21,870,826,246 | 5.843 | 1 passed; 77 children |

## Reproducing the measurements

Build and find the binary and cwd:
`heavy cargo nextest list -p temper-conformance-world --test suite --message-format json`.
For each test above, run the unoptimized binary from its cwd:

```sh
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR=$PWD valgrind --tool=callgrind \
  --zero-before=skein_fork::process::child \
  --callgrind-out-file=<slot>/target/limits-worlds/conformance-causes/<arm>-<story>.%p.out \
  <binary-path> --exact <test-name> --nocapture'
```

Use the same command before and after. The child-entry reset excludes the
inherited prefix counters. Sum each file's `summary:` (PROGRAM TOTALS),
including the parent once: 79 files for CI repair and 78 for changed judge.
As in `temper/docs/plan/limits-worlds/figures/cuts-fork.md`, these summaries
conservatively retain inherited libtest overhead. Self and inclusive function
counts aggregate raw records by function name; even `calls=0` is a call edge.
Inclusive frames overlap. Forwarding and iterator wrappers are omitted from
the inclusive cause tables. Read annotations with
`callgrind_annotate --inclusive=no <file>` and `--inclusive=yes <file>`.

Native CPU uses bash time, which counts waited-for children (`/usr/bin/time`
is absent). The first and second columns in its output are added above:

```sh
heavy bash -lc 'cd <cwd>
TIMEFORMAT="native user %U system %S wall %R"
time CARGO_MANIFEST_DIR=$PWD <binary-path> --exact <test-name> --nocapture'
```

Native timing is a single run per arm; instructions are the primary comparison.
No budget, deadline, patience, build setting or cut was changed.

## First fix: keyed row projection

Commit `02e320c27a` replaces each changed row's contribution under its key and
retains unchanged naming and translated facts. Erasure removes the row's spend,
balances, writers and receipts; assembling current contributions retains shared
receipts. Cross-row projection ownership and proposal acceptances are resolved
against current rows. Cold observation, bulk writes and missing feeds rebuild
whole. The referee still receives a complete snapshot at each commit, and its
checks and commit boundaries are unchanged. This pays D034.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| CI repair | Keyed projection | 1 | 24,189,059,072 | 6.967 | 1 passed; 78 children |
| Changed judge | Keyed projection | 1 | 20,219,743,088 | 5.657 | 1 passed; 77 children |

The instruction savings are 3.90% and 7.55%. CI's single native CPU timing rose
slightly; it is not presented as a CPU gain. At this arm, CI's policy equality
still costs 3,135,675,258 Ir (12.96%); the projected adapter's borrowed-map work
costs 852,128,829 Ir (3.52%, its inclusive frame less projection and connector
translation). Those are the next causes, in that order.

## Second fix: immutable policy snapshots

Commit `2b0b663c86` gives the adapter and referee immutable, shared policy
values. Mutable access detaches the edited value. Every inspection still
refreshes the referee before processing the same observations; an adapter
without the optional snapshot retains the whole value comparison. Tests cover
both directions of edits and refresh after an external referee edit.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| CI repair | Policy snapshot | 1 | 21,050,322,762 | 6.359 | 1 passed; 78 children |
| Changed judge | Policy snapshot | 1 | 18,015,359,638 | 5.868 | 1 passed; 77 children |

This removes another 3,138,736,310 / 2,204,383,450 Ir, 12.98% / 10.90% of the
preceding arm. The two fixes together remove 16.37% / 17.63% of baseline Ir.
The single changed-judge CPU timing rose; no CPU gain is claimed for that arm.
The next common cause is reconstructing borrowed maps and resolving cross-row
facts: the adapter map alone costs 852 million / 1,012 million Ir in the
preceding arm. Keyed joins can read the changed durable rows directly.

## Third fix: direct durable lookup and keyed joins

Commit `0a8fa91711` consumes the whole row iterator only for a rebuild. Normal
commits look up their submitted keys directly; keyed histories, decisions and
amendment targets resolve the same cross-row facts in the same order. The
connector adapter reads accepted decisions from those current keyed summaries.
A focused test changes and erases an amendment while retaining its decision;
omitting the link fails whole-store equivalence at commit 2.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| CI repair | Direct feed | 1 | 19,849,743,944 | 6.555 | 1 passed; 78 children |
| Changed judge | Direct feed | 1 | 16,542,216,305 | 5.526 | 1 passed; 77 children |

This saves another 1,200,578,818 / 1,473,143,333 Ir, 5.70% / 8.18% of the
preceding arm. CI's single native timing rose slightly again. The remaining
adapter scans in effect evidence cost about 386 / 860 million inclusive Ir;
fixture construction for repeated public document watches is another cause
in CI repair. They are considered next, with independent full-walk comparisons
for indexed evidence and every public readback retained.

## Fourth fix: typed effect evidence lookups

Commit `04347dd69c` narrows entry and call discovery to their typed key ranges,
and looks up known calls, tasks and issues by key. Purpose, task, goal, phase
and origin predicates stay checked. Unit builds compare identities, outcomes,
issue evidence and the complete seen/held/settled accounting with full walks.
Inverting the entry match fails by relabeling a call as a procedure.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| CI repair | Indexed effect evidence | 1 | 19,437,631,801 | 6.779 | 1 passed; 78 children |
| Changed judge | Indexed effect evidence | 1 | 15,589,228,169 | 5.476 | 1 passed; 77 children |

This removes another 412,112,143 / 952,988,136 Ir, 2.08% / 5.76% of the
preceding arm. The native CI timing rose; instructions remain the comparison.
A compatibility check also holds the referee's existing Send and Sync traits;
the final arm uses atomic immutable policy sharing, with edits still detached.
