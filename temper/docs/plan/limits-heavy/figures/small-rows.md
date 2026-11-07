# small-rows: figures

| Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---:|---:|---|---|
| `temper-client-tools::lib::tests::jig::turn::a_document_sized_turn_prints_escaped_tool_names_with_the_shipped_limits` | 6,995,481,808 | 6,832,067,781 | Input construction puts each NUL separately; most remaining work validates the printed JSON | Allocate the same zero bytes once; 2.3% fewer instructions |
| `temper-client-tools::lib::tests::jig::reads::a_document_at_its_byte_bound_prints_its_maximal_json_escapes` | 5,180,449,245 | 3,363,194,720 | Rendering hexadecimal paths builds a nibble array and calls the sized writer per digit, on both measuring and writing passes, including the digest's passes | Fill exact two-byte pairs in one final-length buffer; construct the same NUL input in bulk; 35.1% fewer instructions |
| `temper-service::lib::setup::tests::lifecycle_tests::setup_malformed_queries_are_bad_requests` | 1,033,151,661 | 140,501,808 | Sixteen service constructions cost 947,830,537 Ir | Build one service; retain all sixteen requests, their 400 and page/header checks, with the target in failures; 86.4% fewer instructions |
| `temper-service::lib::accounts::tests::expiry_and_native_paste_race_take_at_most_one_redirect` | 3,361,893,352 | 1,097,274,466 | Each schedule starts and restarts a forge service although the host only uses its store and values; settling runs 512 turns even after quiescence | Construct a durable store and values directly, and stop settling when the host and store queues are quiet; retain all 54 schedules; 67.4% fewer instructions |
| `temper-client-tools::lib::tests::jig::reads::binary_paths_print_fixed_width_lowercase_hex` | unknown | unknown | New focused regression, not profiled | Exact output and measured length for empty paths, every hex digit, leading zeroes and binary bytes |

## Measurement and decisions

The base is `1665bd6e5b`. All four before profiles preceded edits. Each
measurement ran one exact test alone through nextest, in the same unoptimized
build configuration on the same x86-64 builder, with this command:

```sh
NEXTEST_DOUBLE_SPAWN=0 \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind --callgrind-out-file=$PWD/target/limits-heavy/NAME.%p.cg" \
heavy cargo nextest run -p PACKAGE --lib -E 'test(=FULL_TEST_NAME)'
```

`lib` denotes the Cargo `--lib` target. Every invocation selected and passed
one test in the default profile. Counts are the test process's whole `Ir`
totals, excluding nextest's listing processes. None starts child processes.
No bound, schedule, query, build setting, budget or timeout changes.

The setup parser returns the malformed-query pages before taking setup state:
no row needs a fresh service. One shared construction costs 74,625,022 Ir
after; the sixteen visits cost 45,525,946 Ir. `status_at` shares the existing
status, page and header assertions while attaching each request's target.

In the race, `SignInWorld::new` costs 2,017,994,226 Ir before. The world's
only service accesses are its durable secrets store and account values;
the root's projects, forge records and restart do not enter the sign-in
assertions. Each schedule still constructs an independent host, store,
values, issuer, sockets and listeners, and still checks redirects,
exchanges, terminal events and cleanup. With just the fixture reduction,
the test costs 1,843,399,916 Ir. Its settling then costs 1,356,803,629 Ir:
each visit and delivered action runs 512 turns. Checking host work and both
store answer queues after each turn reduces the whole test further to the
table's after count. The existing 512-turn backstop stays.

The document's `hex` costs 4,812,066,695 Ir before and 3,159,465,571 after,
including JSON measurement and writing. Hex-pair filling alone brings the
whole document test to 3,528,640,895 Ir; bulk NUL construction supplies the
remaining reduction. `maximally_escaped_json_text` keeps its signature and
the same bytes. `document_at_byte_bound` still measures the entire encoded
document at the shipped bound, and the turn still counts every six-byte
NUL escape at its shipped limits.

The focused hex regression measures and writes the same expected bytes.
Replacing the low nibble with the high nibble makes it fail on
`0123456789abcdef` versus `0022446688aaccee`; restoring the low nibble passes.
Existing digest and document tests retain coverage of callers' JSON.

## Found and not fixed

- The turn's lossless JSON/duplicate-key validation costs 5,214,465,400 Ir
  on both sides, about 0.820 CPU s on the request's scale. Its underlying
  string decoder costs 4,891,609,446 Ir before. This assertion remains;
  removing it would weaken the test. A further repair belongs in the shared
  JSON reader/string decoder, outside this task's JSON-writing scope, and
  needs the decoder's own focused coverage.
- The unchanged occurrence scan costs 871,883,555 Ir, about 0.137 CPU s.
  An overlapping search using `bytes::find_from` was measured at
  878,343,149 Ir and reverted: constructing a search for each frequent
  match saves nothing here. It still counts overlapping occurrences of the
  same bytes. A faster search would need to retain a searcher across matches;
  even removing this scan entirely falls below the stopping threshold.
- JSON string measurement costs 1,503,745,836 Ir in the document and
  390,061,086 Ir in the turn; writing escaped strings costs 550,389,976 and
  348,028,667 Ir respectively. These counts include validation and writing
  the required bytes, including the digest's independently encoded subject.
  Eliminating all measurement would have a 0.235 CPU s ceiling across the
  two tests, but would lose validation and exact sizing. Further work would
  need bulk string classification or an encoder path for already validated
  hex, preserving refusal and byte-for-byte output. The remaining hex
  construction outside `Encoder::put_string` is 1,105,283,943 Ir in the
  document, a 0.128 CPU s ceiling on the original scale. These are inclusive
  costs and are not additive savings.
- The race's remaining settling costs 610,479,172 Ir across all 54
  schedules, a 0.163 CPU s ceiling on the original scale; it now stops at
  quiescence. Its independent issuer's RSA signing costs 315,239,382 Ir
  (0.084 CPU s). Each fresh durable store costs part of the combined
  342,533,129 Ir in `marked_store` (0.092 CPU s). Avoiding the required
  fresh states or issuer exchanges would weaken the schedules. A smaller
  general store fixture is shared-fixture work; retain that owner's changes
  if a merge meets them.
- Setup's remaining shared construction costs 74,625,022 Ir, a 0.079 CPU s
  ceiling on the original scale. Further general fixture work belongs to
  the shared-fixture task. No remaining test-local cause reaches about
  0.2 CPU s; the larger reader and writer paths are recorded above.

## Tests that went

None.
