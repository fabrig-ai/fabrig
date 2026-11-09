# playground: status

2026-11-09. Where skein's code lags or departs from [playground.md](playground.md).

Section 6.2's *What a difference says* has no known gap: `replay`'s
`Differs` names the line's input by `log::input_text`, none on the last
line, and a `Why::Said` the log's digest, the replay's and its last 8
lines, each cut past 200 bytes (`skein-playground`'s `replay.rs`;
`a_said_difference_shows_its_last_lines`,
`last_lines_are_cut_and_drop_the_oldest` and skein-ui-web's
`the_text_is_the_designs`).

- **Departs:** 2: `skein-playground`'s manifest names two crates past
  those 2 and README.md, 3 give it: `skein-shell`, for the `Host` that
  `skein-sim-world` re-exports, and `skein-sim`, which none of its code
  uses; no reason is recorded.
- **Departs:** 6.1: the log the code writes and reads is version 2,
  and a version 1 is refused (`skein-ui-web`'s `log/text.rs` and
  `log/reader.rs`): an exchange's input adds, after its body and before
  its headers, `cache=<browser|no-store> keep=<count>` and a
  `name=<bytes>` for each header it keeps, and an answered report a
  `header=<name bytes>:<value bytes>` for each header kept. They carry
  the bridge's `Exchange` `cache` and `keep`; the section gives version
  1 without them, and no reason is recorded.
