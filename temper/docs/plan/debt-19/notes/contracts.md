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
