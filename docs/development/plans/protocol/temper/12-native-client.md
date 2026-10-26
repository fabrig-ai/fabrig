# Session 12: the native client, and a chat answered by the real binary

temper gets its native client and its first end-to-end story
(protocol.md, section 12, steps 4 and 5). `temper-shell-terminal` is
made with three of its modules (client.md, section 2): `client`, the
native client, composing temper's client root and view root (11),
`jig-protocol-client-link` and `jig-shell-native`'s shell as one host of
skein's loop, copied from the native client of jig's client protocol
world (ordinary Rust); `route`, the client's protocol routing between
the link and the root (step code, total functions, exhaustive both ways,
no state); and `limits`, the client's limits and their sum (step code).
Then the process tier's first story: the real `temper serve` on a
scratch directory, skein's fake LLM peer on loopback, jig's fake person
in front of temper's native client, signing in by the link the binary
printed and asking a chat that names no repository, which is answered.
The terminal (`render`, `command`, `terminal`) is 16's; the forge
family's records in `route` are 15.1a's.

## Read first

- `client.md`, sections 2, 3.3, 3.4, 6.2, 6.6, 6.7 and 7; `domain/testing.md`,
  2.6; `protocol.md`, section 12, steps 4 and 5.
- jig's `docs/design/protocol/client.md`, sections 8, 10 and 11, at the
  pin.
- `decisions.md`, 13, 16 and 17; `code-organization.md`, sections 2, 4,
  5 and 6.
- `findings/jig-client.md`, sections 3 and 4, Discrepancies 1 to 5, 7
  and 15.
- jig at the pin: `tests/client-protocol/src/{native.rs (360 lines),
  front.rs:14-216 (Side), front/persistence.rs, family.rs,
  fixtures.rs:94-105}`; `crates/jig-protocol-client-link/src/{boundary.rs,
  limits.rs}`; `crates/jig-shell-native/src/{shell.rs, sign_in.rs,
  Cargo.toml, clippy.toml}` (its `[lints]`); `examples/ops/domain-client/src/{boundary.rs,
  store.rs, address.rs}` and `tests/ops/src/client/app.rs:143-260` (the
  root's `start`, `went`, `act`, `duty`); `testing/jig-test/tests/support/mod.rs:79-212`
  (the person host over stdout).
- `temper/10-startup-and-binary.md` (10.4's support); `temper/11-client.md`
  (11.1's root, 11.2's view); `temper/03-wire-codecs.md` (03.2's
  `FORGE`, `CLIENT_VERSION`).
- temper: `tests/system/src/composed.rs:102-127` (`report_script`, the
  chat's one completion).

## What the design asks

- **The native client** (client.md, 2): "`client`: the native client, the
  composition of the client root, the view root,
  `jig-protocol-client-link` and `jig-shell-native`'s shell as one host
  of skein's loop, copied from the native client of jig's client
  protocol world. Ordinary Rust. The worlds put jig's fake person in
  front of it where `temper open` puts the terminal."
- **The routing** (client.md, 2): "`route`: the client's protocol routing
  between the link and the client root: jig's records to and from the
  root as the link gives them, the forge family's records decoded and
  encoded with `temper-protocol-client-wire` (jig's `protocol/client.md`,
  11). Step code: small total functions, exhaustive both ways."
- **Its limits** (client.md, 2): "constants of `temper-shell-terminal`,
  taking jig's client limits from jig's small profile where jig names
  them. The client reads no configuration".
- **Each iteration** (client.md, 6.2): "`jig-shell-native` iterates; its
  loaded records start the link; its sign-in outcomes; the root starts
  once the client is signed in, by a token loaded or by the link's
  sign-in (3.4); the link's events through the routing into the root;
  the root's `fire` and `take`, the link's `fire`; the view built".
- **Signing in** (client.md, 7): `Shell::sign_in` with the printed link;
  "`jig-shell-native` writes the token into DIRECTORY before it reports
  the sign-in, and only then does the client start." A service's
  credential "is handed to the shell by the link, never to the domain,
  and printed once" (6.6).
- **The first end-to-end story** (protocol.md, 12, step 5): "the real
  binary on a scratch directory, skein's fake LLM peer on loopback, jig's
  native fake person through temper's client, signing in by the printed
  link and asking a chat that names no repository. A run with no
  repository has no workspace ..., so it needs no git and no forge; the
  configuration names one forge, which no call reaches."

## Choices this plan makes

- **temper's own native client** (decision 17): jig's is world code bound
  to jig's test application, with the fake person inside, panicking on
  sign-in paths (findings/jig-client.md, Discrepancies 1 to 3). temper's
  holds no person: it offers its tree and takes acts (`tree`, `act`,
  `go`), and the person, the terminal or a world drives it.
- **Started by a token loaded, or by the link** (Discrepancy 3): the
  caller says which at construction (`Start::Link(link)` or
  `Start::Token`); with `Start::Token` the root starts once
  `Shell::loaded` has returned, as the shell sets the loaded token on its
  connections in that same iteration (`shell.rs:242-244`). The terminal
  (16.4) learns that a token exists by reading it once (client.md, 6.1).
- **Sign-in outcomes are the caller's** (Discrepancy 2): `SignedIn`
  starts the root and `SignedOut` steps the root's `SignedOut`; `Again`,
  `Lost` and `Refused` go to the caller unchanged (`NativeClient::signing`),
  which retries after its backoff (16.3) or fails its test (12.2). The
  root's `SignIn { provider, back }` is a notice for the shell
  (client.md, 6.6: a terminal cannot follow a provider). temper
  registers no provider (client.md, 8).
- **`route` keeps no token map** in this session: only jig's client
  domain reads and watches (the forge's side reads and watches nothing,
  client.md, 4.2), so jig's read and watch tokens and the table's
  attempts reach the link unchanged; jig's `Side` remapped tokens
  (`front.rs:24-28, 53`) only because a family's reads shared the link's
  token space. A family readout or delivery that arrives anyway is a
  notice (`Unasked`). 15.1a adds the one bounded table the forge's
  requests need (its attempts in flight, for jig's refusal at
  admission), which is the "bounded token maps" of this plan's skeleton.
- **Where each of the link's requests goes** (`route::up`, exhaustive):
  `Write`, `Remove`, `Exchange`, `Abandon`, `OpenStream`, `CloseStream`
  to `Shell::down`; `Kept` to the root's start records; `Answer`,
  `Watch`, `Ended`, `Read` (jig's) and `Hello { family: None }` to the
  root; `ServiceCredential`, `Dropped`, `Fact`, `Hello { family: Some }`
  (15.1a routes the forge's to the root), a family `Read` and
  `FamilyDelivery` to the shell's notices. A family answer
  (`Answered::Family`) reaches the root as `answer: None` until 15.1a
  decodes it, as the root takes any answer that does not decode.
- **The small store's versions:** jig's `STORE_VERSION` and the table's
  `store::VERSION`; the forge's side keeps no record of its own
  (client.md, 3.4), so a `Saved::Family` found at start is dropped by the
  link and told as `Dropped`. A table entry holding a family's request is
  `Unread` until 15.1a restores it.
- **The client's limits** (Discrepancy 7: jig has no client profile):
  constants in `limits::native()`, each with its doc line, starting from
  jig's ops world's `tests/ops/src/client/app.rs:24-60` and jig's client
  protocol world's `fixtures.rs:94-105` (`connections_limits`), raised
  where a person's use needs it: `text` 4,096, `document_bytes` 1,048,576,
  `rows` 64, `watches` 16; `EXCHANGES` 8, `DEADLINE` 30 s. Each must fit
  what the engine's small profile serves (jig's `clients` limits); 12.2's
  story is the check. At jig's pin those raised values are refused
  (`rows` past the wire's ceiling of 32, and jig-view's
  `Limits::compose` refuses `text` 4,096), so each of the four rises from
  ops's value toward the target only as far as `link::Limits::new` and
  jig-view's `Limits::compose` accept, `rows` at most 32, its doc line
  naming the bound that stopped it (decisions.md, ruling 122).
- **The fake person is the process tier's** (`crates/temper/tests/support/person.rs`):
  a host owning the native client and `jig_fake_person::Person<temper_client_domain::Address>`,
  reading the binary's stdout for the link as jig's support's `Person`
  does (`support/mod.rs:103-212`), each iteration handing the person the
  client's tree (`TreeFace`) and its act back to the client.
- **The chat's script:** the person goes to jig's conversations page
  (`Address::Conversations { project: None }`), types "Hello" into "First
  words", presses "Start a conversation", and waits to see the answer
  (jig's `native.rs:17-66`, `answered_chat`); the fake LLM peer answers
  with one completion, smith's `finish` with `{"report": ANSWER}`
  (`tests/system/src/composed.rs:102-127`, `report_script(false)`, its
  cue empty).

## Rules that bind it

- "No increment of this plan: ... in `temper-shell-terminal`'s step
  modules (`route`, `render`, `command`, `limits`), uses `Vec`, `String`,
  `format!`, `std::`, a closure or a `loop`/`while`: the check is `grep
  -nE 'Vec<|String|format!|std::|\|[a-z_, ]*\||\bloop\b|\bwhile\b'
  crates/temper-shell-terminal/src/{route,render,command,limits}.rs ...`,
  which prints nothing" (code-organization.md, section 6). That pattern
  also matches `||` and an or-pattern of lowercase names: write such
  conditions as a `match` or nested `if`s.
- "**`temper-shell-terminal`** ... ordinary, precedent jig's
  `crates/jig-shell-native/clippy.toml`; its modules `route`, `render`,
  `command` and `limits` are step code" (code-organization.md, section 2).
- "**The process and live tiers run only where a Proof line or 'Done
  when' says,** once" (README, Rules); "No other
  variable is read by temper's code or tests" (code-organization.md, 1).

## Increments

### 12.1 `temper-shell-terminal`: the native client, its routing and its limits

Scope: the crate `crates/temper-shell-terminal` (ordinary Rust, its
`[lints]` and `clippy.toml` copied from jig's `crates/jig-shell-native`):
`route.rs` and `limits.rs` (step code, the sketch); `client.rs`
(ordinary Rust): `NativeClient`, `Settings`, `Start`, after jig's
`native.rs` and `front.rs`'s `Side` (`:14-216`), with temper's client
root (11.1, made with `FORGE` and, once 15.1a lands, `CLIENT_VERSION`)
and view root (11.2), no person, every link request through
`route::up` and every root request through `route::down`, the root's
`Show` stepping its `Went`, `SignOut` to `Shell::sign_out`, the link
started at `Shell::loaded` with `route::versions()`, the root started
as Choices say, at the inbox (client.md, 3.4), with offset 0 (universal
time, client.md, 11); `notices` and `signing` for the caller; `close`
(`Shell::close`, no new exchange).
Needs: 11.2's view root (the session's start); 03.2's `FORGE`.
Files: `crates/temper-shell-terminal/{Cargo.toml, clippy.toml,
src/{lib.rs, client.rs, route.rs, limits.rs, tests.rs,
tests/fixtures.rs}}`; the workspace `Cargo.toml` (member; entries
`temper-shell-terminal`, `jig-protocol-client-link`, `jig-shell-native`,
`temper-protocol-client-wire`); `Cargo.lock` (the pins' three commands
where a crate is new to it). Shared hotspot: the workspace `Cargo.toml`.
No existing type changes Shape.
Shape: the sketch's `route` and `limits` (compiled), and the
ordinary-Rust sketch of `client` below. `limits::link` is
`link::Limits::new(limits.client.jig, EXCHANGES, DEADLINE)`;
`worst_case` sums 11.1's root's, 11.2's view's, the link's
(`jig_protocol_client_link::worst_case`) and the shell's with checked
arithmetic.
Test shape: step tests of `route` and `limits` in `src/tests.rs`;
fixtures in `src/tests/fixtures.rs` (made here: `ATTEMPT`, `READ`,
`WATCH` tokens, `SERVICE: u64`, `CREDENTIAL: &[u8]`, `jig_record()`,
`table_record()`, `inbox_read()` (jig's `contract::Read` of the inbox)).
Tests: `a_jig_read_reaches_the_link_with_its_own_token`,
`a_jig_watch_event_reaches_the_root_unchanged`,
`a_table_record_saved_then_kept_comes_back_the_same`,
`exchanges_and_store_writes_go_to_jigs_shell` (rows: `Write`, `Remove`,
`Exchange`, `Abandon`, `OpenStream`, `CloseStream`),
`a_service_credential_is_a_notice_never_a_root_event`,
`a_family_readout_or_delivery_nobody_asked_for_is_a_notice`,
`the_native_limits_pass_each_worst_case`. Under 20 ms each. Reference
test, in full:

```rust
// crates/temper-shell-terminal/src/tests.rs
#[test]
fn a_service_credential_is_a_notice_never_a_root_event() {
    let credential = Secret::new(Box::from(fixtures::CREDENTIAL));
    let up = route::up(link::Request::ServiceCredential { party: fixtures::SERVICE, credential });
    let Up::Notice(Notice::Credential { party, credential }) = up else {
        panic!("a service's credential goes to the shell");
    };
    assert_eq!(party, fixtures::SERVICE);
    assert_eq!(credential.expose(), fixtures::CREDENTIAL);
}
```

Proof: unit, `cargo nextest run -p temper-shell-terminal`, run
once: jig's reads and watches pass unchanged both ways, a table record
round-trips through the small store's shape, the shell gets its
exchanges and writes, a service's credential and an unasked family
record never reach the root, and the native limits pass every worst case
(the seven tests above); under 0.5 s. The native client's composition
first runs in 12.2's process story (the process layer, 10.4's), which is
the highest layer that can run it before 16.3's machine world.
Don't: hold a person in the native client; panic on a sign-in outcome;
keep a token map in `route` (15.1a adds the one it needs); read an
environment variable.
Size: at most 600 lines.
Removes: nothing.

### 12.2 A chat answered by the real binary

Scope: the process tier's first story (protocol.md, 12, step 5;
domain/testing.md, 2.6, steps 1 and 2): `crates/temper/tests/support/person.rs`
(the person's host: the native client made from the printed link, as
Choices say, a `Script` of `jig_fake_person::Step`s, its tree handed to
the person each iteration, the person's `Act` through `NativeClient::act`,
its `Go` through `go`, a signing other than `SignedIn` failing the test
naming it, the client closed once the script is done); `support::run`
gains the person; `support::chat_script(answer)`; `process.rs` gains
the story's test. The process tier's configuration
(`support::configuration`) overrides the engine's documents as
configuration.md, 7 says ("The engine's documents fit temper's
client"): `engine.limits.overrides` `documents.rows` 2 and
`documents.text_bytes` 128, the native client's `rows` and `text`
(`temper-shell-terminal/src/limits.rs`); `documents.chips` and
`documents.lines` stay jig small's. `tests/configuration.rs` holds it to
the native client.
Needs: 10.4's process support.
Files: `crates/temper/{Cargo.toml (dev-dependencies
`temper-shell-terminal`, `temper-client-domain`, `jig-fake-person`,
`jig-view`, `jig-domain-client`, `jig-shell-native`, `jig-protocol-config`),
tests/{process.rs, configuration.rs (new), support/mod.rs,
support/person.rs}}`; `Cargo.lock` (the pins' three
commands where a crate is new to it). No existing type changes Shape.
Shape: the ordinary-Rust sketch of the person below.
Test shape: `tests/process.rs`:
`the_binary_answers_a_chat_from_a_person_signed_in_by_its_link`, aiming
at 30 s, at most 60 s; `tests/configuration.rs`:
`the_process_configurations_documents_fit_the_native_client` (the
configuration `support::configuration` writes, its `engine.limits`
parsed with `jig_protocol_config` as the service parses it: its
`core.documents.rows` at most `temper_shell_terminal::limits::native()`'s
`client.jig.rows`, its `text_bytes` at most `client.jig.text`), under
20 ms, in the default suite, not the process profile. Reference test, in full:

```rust
// crates/temper/tests/process.rs
#[test]
fn the_binary_answers_a_chat_from_a_person_signed_in_by_its_link() {
    let scratch = support::Scratch::new("chat");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let peer = support::fake_llm(Box::new([support::chat_script(support::ANSWER)]));
    let seen = support::run(
        scratch.path(),
        support::person::answered_chat(b"Hello", support::ANSWER),
        Some(peer),
        Duration::from_secs(60),
    );
    assert_eq!(seen.signed_in, 1, "signed in once, by the printed link");
    assert!(seen.finished, "the person saw the answer: {}", String::from_utf8_lossy(&seen.stderr));
    scratch.remove();
}
```

Proof: process, `scripts/proof-one-process.sh`, run once:
the real binary prints its link, jig's fake person signs in through
temper's native client by it, starts a chat that names no repository,
and sees the fake model's answer on jig's page (the test above, beside
10.4's); under 3 minutes for the profile.
Don't: give the person any link but the printed one; reach the forge
(no call does: the run has no workspace); add a person to the native
client; raise the native client's `rows` or `text` past what
`Limits::compose` accepts (ruling 122), or lower its `text` to make
room for more rows.
Size: test code; production 0 lines.
Removes: nothing.

## Interface sketches

`s12.rs` holds `route` and `limits`, compiled; its last module stands in
for 11.1's client root as session 11's sketch has it (its `boundary`,
`address` and `store` modules, abridged), and 11.1's landed code wins
where they differ. The native
client and the person are ordinary Rust, below it, not compiled.

```rust
//! Sketches of session 12: `temper-shell-terminal`'s step modules, `route`
//! (the client's protocol routing between `jig-protocol-client-link` and
//! the client root) and `limits` (the native client's limits). The native
//! client's composition (`client`) is ordinary Rust: its sketch is in the
//! session file only.
//!
//! Who owns what: the link owns exchanges, the stream and the small store's
//! encodings; the client root owns what a request means; `route` keeps no
//! state and decides nothing: each of the link's outputs goes to the root,
//! to jig-shell-native's shell or to the shell's notices, and each of the
//! root's to the link or to the shell. 11.1's types are stand-ins at the end,
//! named for their owner; its landed code wins where they differ.
#![expect(unused_variables, reason = "a sketch's bodies are unreachable; its parameters are its signature")]

/// `crates/temper-shell-terminal/src/route.rs` (12.1): small total
/// functions, exhaustive both ways, no `Vec`, `String`, `std::`, closure or
/// loop (code-organization.md, section 6).
pub mod route {
    use super::client;
    use jig_domain_client::contract::VersionRange;
    use jig_protocol_client_link as link;
    use skein_lib::Secret;

    /// Where one of the link's outputs goes.
    #[derive(Debug)]
    pub enum Up {
        /// One event for the client root, stepped at once.
        Root(client::Event),
        /// One request for jig-shell-native's `Shell::down`: an exchange, the
        /// stream, a small-store write or removal.
        Shell(link::Request),
        /// A small-store record restored at start, held for the root's
        /// `Event::Start`.
        Kept(client::store::Record),
        /// For the shell that runs the native client; never the domain.
        Notice(Notice),
    }

    /// What the native client hands its shell (the terminal, a test's
    /// person) and never the client root.
    #[derive(Debug)]
    pub enum Notice {
        /// A service's credential, shown once by the shell, never kept.
        Credential { party: u64, credential: Secret },
        /// A record of a version its owner does not read, dropped at start.
        Dropped { keeper: link::Keeper },
        /// A restored record no owner of this client reads.
        Unread { keeper: link::Keeper },
        /// The versions the engine serves for an application family (15.1a
        /// routes the forge's to the root).
        Served { family: u16, versions: VersionRange },
        /// A family readout or delivery the routing never asked for.
        Unasked,
        /// The link's own fact.
        Fact(link::Fact),
    }

    /// Where one of the client root's requests goes.
    #[derive(Debug)]
    pub enum Down {
        /// One event for the link, stepped at once.
        Link(link::Event),
        /// One duty of the shell's.
        Duty(Duty),
        /// Nothing: an answer to a shell's ask the native client never makes.
        Nothing,
    }

    /// The client root's requests the native client does itself.
    #[derive(Debug)]
    pub enum Duty {
        /// Go to `address`: the native client steps the root's `Went`.
        Show { address: client::Address, replace: bool },
        /// A provider's sign-in, which a terminal cannot follow (client.md, 6.6).
        SignIn { provider: u16, back: client::Address },
        /// jig-shell-native's sign-out exchange.
        SignOut,
        /// A fact of the root's, counted by the native client.
        Fact(client::Fact),
    }

    /// One of the link's outputs to its destination.
    #[must_use]
    pub fn up(request: link::Request) -> Up {
        unreachable!("sketch")
    }

    /// One of the client root's requests to its destination.
    #[must_use]
    pub fn down(request: client::Request) -> Down {
        unreachable!("sketch")
    }

    /// A root's record as the link saves it.
    #[must_use]
    pub fn saved(record: client::store::Record) -> link::Saved {
        unreachable!("sketch")
    }

    /// A restored record as the root takes it; none for a record no owner
    /// of this client reads (a family's own record; until 15.1a, a table
    /// entry holding a family's request).
    #[must_use]
    pub fn kept(saved: link::Saved) -> Option<client::store::Record> {
        unreachable!("sketch")
    }

    /// A root's store key as the link erases it.
    #[must_use]
    pub fn erased(key: client::store::Key) -> link::Erased {
        unreachable!("sketch")
    }

    /// The store versions the link's start takes: jig's and the table's.
    #[must_use]
    pub fn versions() -> [link::StoreVersion; 2] {
        unreachable!("sketch")
    }
}

/// `crates/temper-shell-terminal/src/limits.rs` (12.1): the native client's
/// limits, constants of the crate (client.md, section 2), and their sum.
pub mod limits {
    use super::client;
    use jig_protocol_client_link as link;
    use skein_lib::Duration;

    /// Exchanges in flight at once: jig-shell-native's and the link's.
    pub const EXCHANGES: u32 = 8;
    /// An exchange's deadline, the link's.
    pub const DEADLINE: Duration = Duration::from_secs(30);

    /// Every limit of the native client.
    #[derive(Clone, Copy, Debug)]
    pub struct Limits {
        /// The client root's (11.1's), jig's client limits among them.
        pub client: client::Limits,
        /// jig-shell-native's transport: exchanges, body bytes, HTTP.
        pub connections: jig_shell_native::Limits,
        /// io's, for the shell's sockets and files.
        pub io: skein_io::Limits,
    }

    /// The constants, each documented with what it bounds.
    #[must_use]
    pub fn native() -> Limits {
        unreachable!("sketch")
    }

    /// The link's limits from the client root's, or why they are unusable.
    pub fn link(limits: &Limits) -> Result<link::Limits, link::Unusable> {
        unreachable!("sketch")
    }

    /// The native client's memory: the root's, the view's, the link's and
    /// the shell's worst cases, summed with checked arithmetic.
    #[must_use]
    pub fn worst_case(limits: &Limits) -> Option<u64> {
        unreachable!("sketch")
    }
}

/// Stand-ins for 11.1's `temper_client_domain` (copied from `ops`'s client
/// root at the jig pin, the forge's side in place of `ops`'s two), as
/// session 11's sketch has them. 15.1 adds `Ask::Forge`, `Answered::Forge`,
/// `Action::Forge`, `Address::Forge` and `Event::Forge`.
pub mod client {
    use alloc::boxed::Box;
    use skein_lib::{List, RequestEnvelope, RequestKey, Token};

    /// The client root's limits; fields beyond jig's elided.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Limits {
        pub jig: jig_domain_client::Limits,
    }

    /// The root's address: jig's (15.1 adds the forge's).
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Address {
        Jig(jig_domain_client::address::Address),
    }

    /// A place the shell went.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Place {
        Address(Address),
        Missing,
    }

    /// One keyed request, tagged with its child (15.1 adds the forge's).
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Ask {
        Jig(Box<jig_domain_client::contract::Ask>),
    }

    /// A decoded terminal, tagged with its child (15.1 adds the forge's).
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Answered {
        Jig(jig_domain_client::contract::Answer),
    }

    /// A person's act, decoded by the view root (15.1 adds the forge's).
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Action {
        Jig(jig_domain_client::action::Act),
    }

    /// One event of the root's.
    #[derive(Debug)]
    pub enum Event {
        Start { address: Address, kept: List<store::Record>, offset: jig_domain_client::boundary::Offset },
        Went { place: Place },
        Act { action: Action },
        Answer { attempt: Token, envelope: RequestEnvelope, answer: Option<Answered> },
        Jig(jig_domain_client::Event),
        SignedOut,
    }

    /// jig's unkeyed outputs, passed through the root.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum JigOut {
        Read { read: Token, what: jig_domain_client::contract::Read },
        Open { watch: Token, subject: jig_domain_client::contract::Subject },
        Continue { watch: Token, subject: jig_domain_client::contract::Subject },
        Close { watch: Token },
        Answered { asked: Token, answer: jig_domain_client::contract::Answer },
        Unknown { asked: Token },
    }

    /// The root's facts.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Fact {
        Jig(jig_domain_client::facts::Fact),
        TableFull,
        PartsFull,
        InvalidRecord,
        InvalidAnswer,
        UnknownConnector,
        KeptAnswered,
    }

    /// One request of the root's; keyed sends come only from the table.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Request {
        Save { record: store::Record },
        Erase { key: store::Key },
        Send { attempt: Token, key: RequestKey, ask: Ask },
        Jig(JigOut),
        Show { address: Address, replace: bool },
        SignIn { provider: u16, back: Address },
        SignOut,
        Fact { fact: Fact },
    }

    /// The small store's records: jig's and the table's; the forge's side
    /// keeps none (client.md, 3.4).
    pub mod store {
        use skein_lib::{RequestKey, RequestRecord};

        /// The table's record version.
        pub const VERSION: u16 = 1;

        /// A versioned child or table record.
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum Record {
            Jig(jig_domain_client::store::Record),
            Table { version: u16, record: RequestRecord<super::Ask> },
        }

        /// A record's key, tagged with its owner.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Key {
            Jig(jig_domain_client::store::StoreKey),
            Table(RequestKey),
        }
    }
}
```

The native client, ordinary Rust, not compiled:

```rust
// crates/temper-shell-terminal/src/client.rs (ordinary Rust, not compiled)

/// What the shell gives the native client: the engine's address, the
/// client's directory's last component, the limits.
pub struct Settings { pub engine: skein_io::Addr, pub directory: Box<[u8]>, pub limits: limits::Limits }

/// How the client is signed in: by the printed link, or by the token the
/// directory holds.
pub enum Start { Link(Box<[u8]>), Token }

/// The native client: the client root, the view root, the link and
/// jig-shell-native's shell, as one host of skein's loop (client.md, 2).
pub struct NativeClient { /* shell, link, root, view, env, out, patches, kept, address, started, closing, notices, signing, seen */ }

/// What the native client counted, for its shell and tests.
pub struct Seen { pub signed_in: u32, pub patches: u64, pub facts: u32 }

impl NativeClient {
    /// The shell adopts `directory` (opened by the caller); a link is checked
    /// against `settings.engine` (`Shell::sign_in`) before anything is sent.
    pub fn new(settings: Settings, directory: Fd, user: u32, seed: u64, start: Start) -> Result<NativeClient, jig_shell_native::Unlinked>;
    /// The view's last tree, for a person or the terminal's renderer.
    pub fn tree(&self) -> &jig_view::Tree;
    /// A shell's act on a node; false when the view decodes it to nothing.
    pub fn act(&mut self, node: jig_view::NodeId, act: jig_view::Act) -> bool;
    /// The person went to `address` (the root's `Went`).
    pub fn go(&mut self, address: temper_client_domain::Address);
    /// A sign-in by a link again, after `Again` or `Lost` (16.3's backoff).
    pub fn sign_in(&mut self, link: &[u8]) -> Result<(), jig_shell_native::Unlinked>;
    /// A sign-in outcome the caller decides on: `Again`, `Lost`, `Refused`.
    pub fn signing(&mut self) -> Option<jig_shell_native::Signing>;
    /// What the shell is told and the domain never is (route's `Notice`), and
    /// the root's `SignIn` as `Notice`-like `SignInAsked`.
    pub fn notices(&mut self) -> Option<Shown>;
    /// Whether the root has started.
    pub fn started(&self) -> bool;
    pub fn seen(&self) -> &Seen;
    /// No new exchange; the small store's last write finished, then closed.
    pub fn close(&mut self);
}
/// What `notices` hands the shell.
pub enum Shown { Notice(route::Notice), SignInAsked }
impl skein_shell::Host for NativeClient { /* iterate (client.md, 6.2's order), queues, deadlines, is_empty, worst_case, operations: jig's native.rs:250-357 */ }
```

The process tier's person, ordinary Rust, not compiled:

```rust
// crates/temper/tests/support/person.rs (ordinary Rust, not compiled)

/// The person's finite script, after jig's `native.rs:13-66`.
pub struct Script { steps: Box<[jig_fake_person::Step<temper_client_domain::Address>]> }
/// Go to jig's conversations, type `words` into "First words", press "Start
/// a conversation", see `answer` within 20 s.
pub fn answered_chat(words: &[u8], answer: &[u8]) -> Script;

/// A host reading the binary's stdout for its link, then driving the native
/// client with the person (jig's `support/mod.rs:103-212`).
pub struct Person { /* client: Option<NativeClient>, person, stdout, root fd, queues */ }

// crates/temper/tests/support/mod.rs gains:
pub const ANSWER: &[u8] = b"Process answer";
/// One completion: smith's `finish` with `{"report": ANSWER}`, cue empty.
pub fn chat_script(answer: &[u8]) -> skein_fake_llm_domain::api::Script;
/// What a story's run showed.
pub struct Seen { pub signed_in: u32, pub finished: bool, pub stderr: Vec<u8> }
/// The binary, the person and the peer in skein's real world until the
/// person's script ends (then the binary is killed) or `deadline` passes.
pub fn run(directory: &Path, script: person::Script, peer: Option<skein_fake_peers::llm::Peer>, deadline: Duration) -> Seen;
```

## Questions to raise when reached

- **The client's limits against the engine's.** A document past the
  client's `document_bytes` or a text past its `text` refuses what the
  engine served. Default: the numbers of Choices; if 12.2's story or
  16.4's shows a refusal, raise them to the engine's small profile's
  `clients` bounds, with the reason in the report.
- **A `Dropped` record's owner.** jig's `protocol/client.md`, 10 says
  "its owner told"; the client root at the pin (`ops`'s) has no event
  for it. Default: a notice the native client counts; the root is not
  told.
- **jig's `Hello` for the family before 15.1a.** Default: a notice,
  ignored by the native client; 15.1a routes `FORGE`'s to the forge's
  side.
- **The engine not ready at sign-in.** `Again` comes back while the
  engine stops (jig's `protocol/client.md`, 8). Default: the process
  tier's person fails the test naming it; 16.3's terminal retries after
  its backoff.

## Done when

- The crate and the story pass: `cargo nextest run -p
  temper-shell-terminal` passes, and `scripts/proof-one-process.sh` passed once in 12.2's Proof, its log
  naming `the_binary_answers_a_chat_from_a_person_signed_in_by_its_link`.
- The step modules keep the subset: `grep -nE
  'Vec<|String|format!|std::|\|[a-z_, ]*\||\bloop\b|\bwhile\b'
  crates/temper-shell-terminal/src/{route,limits}.rs` prints nothing.
- No sign-in path panics in the native client: `git grep -n 'panic!' --
  crates/temper-shell-terminal/src/client.rs` prints nothing.
- The pins held: `grep -oE 'forgetic/(jig|smith|skein)\.git#[0-9a-f]+'
  Cargo.lock | sort -u` prints exactly the three pinned revisions.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log -1 --grep='^Step: temper-12\.2$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed`.
