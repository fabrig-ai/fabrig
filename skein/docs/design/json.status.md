# json: status

2026-11-06. Where skein's code lags or departs from [json.md](json.md).

- **Not yet:**
  - 3.4, 7: `string::decode` takes a plain run a byte at a time, through
    `next`, `Buffer::room` and `Buffer::put`, and the text buffer is a
    `List<u8>`.
  - 6: the step rows and the sweep of plain runs read.
  - 6: the fuzz target.
- **Departs:**
  - 5.1: `skein/docs/foundation/programming-model.md`, 6.1 has the queues
    between components uncapped, but `skein-json/src/collector.rs`'s
    `demands`, which stages the tokenizer's one request within `up`, is
    `Queue::with_capacity(1)`, because a growing queue's four-entry floor
    takes `skein/tests/json/tests/suite/memory.rs`'s
    `COLLECTOR_PEAK_CEILING` past its 440 B, and the push cannot fail: the
    queue is empty at every entry, and the tokenizer states at most one
    request.
  - 5.3: one whole buffer kept as a document is `parse`, but
    `skein-forge-conformance/src/json.rs`, `Json::parse`, and
    `skein-throwaway-forgejo/src/instance.rs`, `value`, each append it to
    an `Intake` and pump by hand a `Collector` that keeps the whole value.
    Both are test kits; no debt item holds them.
