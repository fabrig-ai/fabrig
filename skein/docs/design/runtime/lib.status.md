# lib: status

2026-11-19. Where skein's code lags or departs from [lib.md](lib.md).

- **Not yet:** 3: `Slab::get2_mut`.
- **Departs:** 12: `skein-lib/src/requests.rs`'s `Entry` holds its three lifecycles in
  `State`, `Keeping` and `Telling`, but the table's methods write most of their
  cells as assignments in their own arms, and `Entry::reachable` and the table's
  `pending` and `next_out` read `keeping` by `==` or `!=`, so a new `Keeping`
  breaks no build there, against `skein/docs/foundation/programming-model.md`, 5.4's one small
  handler per cell and exhaustive matches. The `debt-28` plan's task took the
  flags and options and left the idiom to its implementer, and that plan's item,
  which asked that the state follow 5.4, is deleted. Seen by reading. 12: that
  plan's request of 2026-11-06 has each of its items end with fewer lines than
  it started; the table's ended 20 lines of Rust up, within its task's cap.
