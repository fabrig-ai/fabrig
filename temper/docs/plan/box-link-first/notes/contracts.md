# box-link-first: the shapes that cross tasks

Binding for every task of this plan. The design is `temper/docs/design/protocol/setup.md`, 7.1 and 7.2, and `jig/docs/design/protocol/client.md`, section 8, *A secret given*.

## jig

```rust
// jig_protocol_engine_clients
pub struct Settings { /* ... */ pub link_secret: Option<Secret> }
pub const LINK_SECRET_BYTES: usize = 32;
```

## temper-service

```rust
// settings.rs
pub struct Seeds { /* ... */ pub link_secret: Option<Secret> }

// startup.rs
pub struct Named {
    /* ... */
    /// `engine.clients.link`: the deployment prints a sign-in link.
    pub link: bool,
    /// `application.clients.public`.
    pub public: Option<application::Origin>,
}

// clients.rs
pub use jig_protocol_engine_clients::LINK_SECRET_BYTES;
/// The URL a printed link's secret opens, as serve prints it, with no line feed (protocol/setup.md, 7.2).
pub fn first_link_url(address: SocketAddr, public: Option<&startup::application::Origin>, secret: &Secret) -> Box<[u8]>;
```

## serve's standard input

Raw bytes, no line feed, closed by the box. None read: no secret given. Exactly `LINK_SECRET_BYTES`: the secret. Anything else: `temper internal serve` refuses with `--box: stdin: expected the first link's secret`. Only under `--box`; a plain `temper serve` never reads its standard input.

## temper-box

```rust
// terminal.rs
pub const PRELUDE: &[u8] = b"Open this link in a browser on this machine.\n\n";
pub const FOOTER: &[u8] = b"\nIt signs you in as this temper's owner, once and for a short while.\nOpened again once spent, it prints a fresh one here.\n";
pub const FRESH: &[u8] = b"\n";              // on stderr, before a fresh link on stdout
pub const CTRL_C: &[u8] = b"Ctrl-C stops temper.\n";  // box.log's, never the terminal's

pub enum Place { Terminal, Log }
impl Fact { pub fn place(self) -> Place; }   // GroupFailed, Restarting and GivingUp: Terminal; every other: Log

// host.rs
/// The first link, where the box draws one (protocol/setup.md, 7.2). Debug shows neither field.
pub struct FirstLink {
    pub url: Box<[u8]>,   // no line feed
    pub secret: Secret,   // LINK_SECRET_BYTES long; the box does not check it
}
pub struct Settings {
    /* ... */
    /// Whole lines, each with its line feed and no time, that open the box's log, before `Logs:`.
    pub log_lines: Box<[Box<[u8]>]>,
    pub first_link: Option<FirstLink>,
    /// Whole lines, each with its line feed and no time, that follow the first block on stderr.
    pub after_block: Box<[u8]>,
}
```

`box.log` opens: `log_lines`, then `Logs: <data>/box/logs/`, then `Ctrl-C stops temper.`, then what happens. Each line begins `HH:MM:SSZ `.

## What a reader of the box's output may rely on

- **The first line on stdout, where a first link is drawn, is the link.** It is printed before serve starts, so it no longer says serve is up. Once it is read, `DATA/box/logs/*` are this run's: the last run's have moved to `logs.previous/`.
- **Serve is up once `DATA/box/logs/serve.log` holds `listening clients `.** That line comes at the moment the first link used to.
- **Stdout holds links alone,** each a whole line, never a blank one. A fresh link's blank line is on stderr.
- **Lifecycle is in `DATA/box/logs/box.log`:** `First run: ...`, `Logs: ...`, `Ctrl-C stops temper.`, `Starting <step>`, `<label>: starting|started|stopped`, `Stopping`, `Stopped`, each after `HH:MM:SSZ `.
- **The terminal keeps errors:** `<label>: failed, <WHY>`, `<WHY>; restarting in <n> s`, `<WHY>; giving up` and the box's refusals, each after `HH:MM:SSZ `, and a failure's tail.
