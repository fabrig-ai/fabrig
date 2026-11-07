# wire-trace: deferred formatting of wire steps

2026-11-07 UTC, base ca05b26eec, after 02de455df1, 8 CPUs,
AMD EPYC-Genoa Processor, valgrind 3.24.0. Unoptimized default profile.

Package: `jig-client-protocol-world`. Test:
`client::account_keys::a_key_ask_carries_its_value_into_the_secrets_store`.
Each reading runs exactly that one test, passing, in its package directory.
Counts include the test process and use callgrind inclusive Ir.

| Package | Test | Arm | Total Ir | Cause | Cause's inclusive Ir | Share |
|---|---|---|---:|---|---:|---:|
| jig-client-protocol-world | account_keys test above | Before | 153,256,084 | Wire::take | 6,567,946 | 4.29% |
| jig-client-protocol-world | account_keys test above | Before | 153,256,084 | alloc::fmt::format under take (256 calls) | 6,346,129 | 4.14% |
| jig-client-protocol-world | account_keys test above | Before | 153,256,084 | Wire::answer | 16,606,637 | 10.84% |
| jig-client-protocol-world | account_keys test above | Before | 153,256,084 | alloc::fmt::format under answer (798 calls) | 12,549,691 | 8.19% |
| jig-client-protocol-world | account_keys test above | Before | 153,256,084 | alloc::fmt::format, whole process | 20,967,655 | 13.68% |
| jig-client-protocol-world | account_keys test above | After | 131,560,113 | Wire::take | 317,604 | 0.24% |
| jig-client-protocol-world | account_keys test above | After | 131,560,113 | alloc::fmt::format under take (0 calls) | 0 | 0.00% |
| jig-client-protocol-world | account_keys test above | After | 131,560,113 | Wire::answer | 2,473,187 | 1.88% |
| jig-client-protocol-world | account_keys test above | After | 131,560,113 | alloc::fmt::format under answer (0 calls) | 0 | 0.00% |
| jig-client-protocol-world | account_keys test above | After | 131,560,113 | alloc::fmt::format, whole process | 2,071,906 | 1.57% |

`answer` also spends 1,772,991 Ir (1.16%) collecting 584 drained fragments,
and 670,432 Ir (0.44%) dropping their temporary vectors. `write` spends
2,288,388 Ir (1.49%), including 2,255,796 Ir extending its byte queue.
These costs justify copying fragments through the queue's slices into the
trace store and examining the queue's extension path.

The test uses 21,695,971 fewer instructions (14.16%). `take` falls 95.16%
and `answer` 85.11%. Neither calls formatting, directly or through a
callee: the formatter's callers are the test's other fixtures and the
harness. Its largest remaining caller is `jig_conformance::core::naming::naming`,
1,319,599 Ir (1.00%); the protocol referee's domain function and closure
account for 717,895 Ir (0.55%). These fixture costs are outside this change.

Fragments now copy from both `VecDeque::as_slices` slices into the trace's
byte store, then append that stored fragment to the intake. No fragment
vector is collected or dropped. The two slice copies cost 445,428 Ir
(0.34%); `as_slices` costs 123,912 Ir (0.09%). Draining the queue without
collecting still costs 91,104 Ir (0.07%). The same 584 fragments reach the
intake, and `answer` still runs 4,090 times.

`write` keeps its extension path: 2,286,010 Ir after (1.74%), versus
2,288,388 Ir before (1.49%). An exploratory `.iter().copied()` extension
cost 2,541,817 Ir, so the final code retains the cheaper existing path.
The exploratory binary used 131,813,611 Ir; it is not the final after arm.

