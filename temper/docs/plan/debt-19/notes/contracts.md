# debt-19: contracts

The shapes that cross tasks, binding where a spec cites them.

## 1. The benchmarks' `Fault` (smith D011)

`smith/benchmarks/src/fault.rs`, re-exported from `smith_bench`. Ordinary Rust, not step code.

```rust
/// What kind of failure a harness step met; the fault's detail says which.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultKind {
    /// A file, directory or process call failed.
    Io,
    /// A record, document or stream did not encode or decode.
    Format,
    /// An authored input, a pin, a configuration or an argument was refused.
    Input,
    /// What was read disagrees with what was pinned or written: a digest, an identity, a committed section.
    Mismatch,
    /// A size, time or count bound was passed, or its arithmetic overflowed.
    Bound,
    /// A login is absent, lapsed or changed.
    Credential,
    /// An agent's event stream or its collection failed.
    Stream,
    /// A preparation step refused the attempt before inference.
    Setup(SetupStep),
}

/// One failure of the harness: its kind and its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: FaultKind,
    pub detail: String,
}
```

- `Display` prints `detail` and nothing else, so every string a result file, a report or a test reads is the same, byte for
  byte, as the string it replaces. **The kind reaches no stored file:** `End::HarnessError { what }` and
  `End::Refused { setup }` keep their `String`s, and no result or summary format changes.
- `impl std::error::Error`, and `From` for what `?` carries today by `.map_err(|error| error.to_string())`: `std::io::Error` (`Io`), `serde_json::Error`,
  `toml` errors and `std::string::FromUtf8Error` (`Format`), `Refusal` (`Input`, its `Display`). There is **no** `From<String>` or
  `From<&str>`: a site that writes words names its kind.
- `SetupStep` stays where it is. `isolate::Setup` and `agent::HarnessFault` go: a preparation refusal is a `Fault` of kind
  `Setup(step)`, and a stream fault a `Fault` of kind `Stream`.
- `task::Refusal` stays, as what a document's reader returns: its `file` and `key` are read by its callers and its tests,
  and it reaches a `Fault` by `From`.

## 2. The io worlds' schedule digest (skein D032)

The three `io-*` tasks each prove that no world's schedule moved, with throwaway code, untracked and gone before the push. That departs from no rule
(`AGENTS.md`, 4), and the task lacks no tool: this section is the tool's spec.

- `skein/tests/io/tests/probe.rs`, untracked: cargo finds it without a tracked edit, since the package's manifest has no `[[test]]`.
- For each world, configuration and seed the suites use (`CALM`, `CHAOS`, `SEEDS`, `0..64`, `0..150`), it prints one line: the world's name, the seed and
  `skein_io::digest::digest(format!("{:?}", (trace, logs, marks, held_back, iterations)).as_bytes())`. `either` returns only `Seen`: digest that, since the same `Proc` runs through `World`.
- Run it at your base and again at your tip, through `heavy`, and `diff` the two outputs. **They are the same, or you stop.** A line that differs is a `Question:`, naming the world and the seed: never retime a test, rewrite an assertion or change a seed.
- A task that changes what a digest can read (a removed field, say) reads the same values by their new path, so the lines stay comparable.
