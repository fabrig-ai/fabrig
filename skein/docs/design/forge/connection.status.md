# forge-connection: status

2026-11-15. Where skein's code lags or departs from [connection.md](connection.md).

Unlisted sections have no known gap. Sections 3.5 and 5.2's key checks
are present: access admission parses without signing, and a key refused
only at signing stops the forge before any request goes out on it.

- **Not yet:** 3.2: a raw body of unknown length (chunked, or ending at
  close) is read a byte per demand, `Read::Fill(1)`
  (`skein-forge-connection/src/connection.rs`, `raw_demand`) [in flight:
  change 3339].
- **Departs:** 10: the item the `debt-24` request of 2026-11-05 paid has the moved tests use
  the fake's own server, and the forge worlds share one io harness. `hand`'s
  tests keep their literal bytes, because they carry faults and identities no
  fake serves, and each world keeps its own arms, because a shared table ended
  131 lines up against 100 down, by that plan's owner. The `seam` feature holds
  `component/seam.rs`'s 108 lines and, with the crate's tests, `fixtures.rs`'s
  190, beside the step code.
  10: the `debt-26` request of 2026-11-06 has its leftovers' net lines fall, but
  they stand 7 up: their code fell 54 and their tests rose 61, 58 of those the
  gate on the seam's one world, which the same finding asks for.
