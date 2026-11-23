# vocabulary: status

2026-11-23. Where the tree's code lags or departs from [vocabulary.md](vocabulary.md).

- **Departs:** 1: `skein/docs/foundation/programming-model.md`, 10.3 puts generic items out
  of step code, but `skein-llm/src/wire/output.rs`'s `Output<P>` is one, as
  this section has it, for the reason it gives. 1: Anthropic's modules name
  nothing of `openai`, but `skein-llm/src/anthropic/request_tests.rs` spells
  `openai::Limits` and `openai::json`, reaching `wire` through `openai`'s
  re-exports, because the request of 2026-11-05 keeps the tests as they are.
