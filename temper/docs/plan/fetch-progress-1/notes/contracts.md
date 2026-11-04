# fetch-progress-1: the contracts

The shapes that cross parts, binding for every task that cites them.

## 1. skein-ui's fraction (`skein/docs/design/ui.md`, 3.2 and 5)

```rust
// skein_ui, exported beside `Spans`
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Fraction { pub done: u64, pub total: u64 }

impl Builder {
    pub fn fraction(&mut self, fraction: Fraction); // on a progress; done <= total; total > 0
    pub fn run(&mut self, run: Run, bytes: &[u8]);  // on a text node or a progress
}
// `Node.fraction` and `Attribute.fraction`: Option<Fraction>
```

- A progress's words are **its own runs:** `b.open(Role::Progress); b.name(..); b.run(Run::Plain, b"42%"); b.fraction(..); b.close();`, never `b.text(..)` inside it. `close` asserts a fraction has runs.
- The web page writes `aria-valuetext` from those runs, `aria-valuemin`, `aria-valuemax`, `aria-valuenow` and `data-progress` from the fraction (`skein/docs/design/ui-web.md`, section 6).
- `skein-ui.css` sets `--skein-ui-progress` from `data-progress` (`skein/docs/design/ui-web.md`, section 12); an owner's rule reads `var(--skein-ui-progress, 0%)`.

## 2. The fetch's line (`temper/docs/design/protocol/setup.md`, 6.6)

On `temper forge fetch`'s stdout, exactly:

```text
fetched forgejo 1048576 of 122057320 bytes
fetched runner 1048576 bytes
```

```rust
// temper_forge_setup::fetch
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fetched {
    pub binary: skein_forge::forgejo::pin::Binary,
    pub done: u64,
    pub total: Option<u64>,
}
impl Fetched {
    /// The line above, its line feed included.
    pub fn line(&self) -> Box<[u8]>;
    /// `forgejo` or `runner`, in this line and on the pipe.
    pub fn word(&self) -> &'static [u8];
    /// The line above, its line feed optional. None for anything else, a zero total, or done past total.
    pub fn read(line: &[u8]) -> Option<Fetched>;
}
```

## 3. The pipe's line (`temper/docs/design/protocol/setup.md`, 7.1)

```text
fetched 3 forgejo 1048576 122057320
fetched 3 runner 1048576 none
```

- `temper_box::pipe::fetched(forge: u16, fetched: Fetched) -> Box<[u8]>` writes it, its line feed included.
- The longest is 64 bytes, `box.line_bytes`'s least, which is why the word is `runner` and not `forgejo-runner`.
- `temper-service`'s link reads it with its own decoder, as it reads every pipe line, into §4's type.

## 4. The engine's row (`temper/docs/design/protocol/forge.md`, 2.8)

```rust
// temper_engine_domain_forge, beside SetUpStep
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SetUpBinary { Forgejo, Runner }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SetUpFetched { pub binary: SetUpBinary, pub done: u64, pub total: Option<u64> }

// Event, beside Connection
Fetched { forge: u16, fetched: SetUpFetched },
// Row, beside `failed`
pub fetched: Option<SetUpFetched>,
```

`Connection::SettingUp { step }` does not change: the progress sits beside the connection, as `failed` does, so none of its forty-odd literals move.

## 5. The wire (`temper/docs/design/client/README.md`, 4.6)

In `client-wire-v1.schema`, in place, `CLIENT_VERSION` unchanged:

```text
enum SetUpBinary { forgejo, runner }
record SetUpFetched { binary: SetUpBinary done: u64 total: option u64 }
record SettingUp { step: option SetUpStep fetched: option SetUpFetched }
```

## 6. The client's row (`temper/docs/design/client/forge.md`, 5.4)

```rust
// temper_client_domain::forge::forges
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binary { Forgejo, Runner }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fetched { pub binary: Binary, pub done: u64, pub total: Option<u64> }
// Row, beside `connection`: Some only while the connection is setting up at Step::Fetch,
// and never with a zero total or done past total.
pub fetched: Option<Fetched>,

// temper_client_domain::forge::words
/// "42%", or with no size "41 MB": `done * 100 / total` and `done / 1000000`, rounded down.
pub fn fetched_figure(fetched: Fetched) -> Box<[u8]>;
/// "Fetching Forgejo", "Fetching its runner": the progress node's name.
pub fn fetching_name(fetched: Fetched) -> &'static [u8];
/// "Setting up: fetching Forgejo · 42%": the row's words, as today it says the step.
pub fn forge_connection(row: &Row) -> Box<[u8]>;
/// The same with no figure, "Setting up: fetching Forgejo": the browser's detail beside its bar.
pub fn forge_connection_bare(row: &Row) -> Box<[u8]>;
```
