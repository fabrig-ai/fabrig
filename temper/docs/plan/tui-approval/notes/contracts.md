# tui-approval: the binding calls

2026-11-07. The shapes that cross this plan's parts. Each task builds its half to this note; neither side waits for the other's task to learn a name. The design sections are `jig/docs/design/protocol/client.md`, 8 (*A sign-in approved by a browser*) and 17.1, `jig/docs/design/client/domain/client.md`, 12, and temper's `client/README.md` and `client/browser.md`, 2.3 and 2.4; this note fixes their names in code.

## 1. What it is

jig's third built-in way of signing in: **a sign-in approved by a signed-in browser, for that browser's own party.** A signed-out client asks; a browser signed in by its cookie reads the ask by its code and approves or refuses it; the client takes it with its secret, and only the take makes a sign-in. Everything before the take is the clients' component's, in memory, as the printed link's state is: nothing is committed, kept or written, and a restart forgets every approval.

- **jig's people child and core are untouched.** At the take the component asks people `Whose { sign_in }` for the approver's sign-in (as the OpenID provider does, `jig-protocol-engine-clients/src/provider.rs`), and with the `Signer`'s identity sends the core's `SignIn { identity, held: None }`, as the printed link and a service do. So the sign-in made is the approver's party's alone, counted, full and expiring as any (people.md, 3); an approver whose sign-in ended before the take makes nothing.
- **A browser approves, nothing else.** The transport says whether an exchange's credential is its own, a browser's cookie (2.2). Reading and deciding an approval are refused `not_a_browser` for any other exchange, a bearer's among them, so `temper ctl` and any agent driving it cannot approve.
- **Enabled by configuration,** `engine.clients.approval`, beside `link`, and only where the clients' listener is on loopback, which the application's startup checks as it checks `link` (section 4).

## 2. The wire (`jig-client-wire 19`, in place)

### 2.1 Records

In `jig/crates/jig-client-protocol-wire/schema/client-v19.schema`, after `SignOutAnswer`, in this order; nothing existing changes:

```
record ApprovalAsk {}
record ApprovalAsked { secret: secret 32 code: fixed 8 expires: u64 }
enum ApprovalAskAnswer { asked: ApprovalAsked, refused, not_ready, busy }
record ApprovalRead { code: fixed 8 }
record ApprovalWaiting { asked: u64 }
enum ApprovalState { waiting: ApprovalWaiting, approved, taken, refused, expired, unknown, not_a_browser, signed_out, not_ready, busy }
record ApprovalDecide { code: fixed 8 approve: bool }
enum ApprovalDecided { approved, refused, used, expired, unknown, not_a_browser, signed_out, not_ready, busy }
record ApprovalSignIn { secret: secret 64 }
record ApprovalFull { sign_ins: u32 earliest: u64 until: u64 }
enum ApprovalSignInAnswer { signed_in: SignedIn, waiting, refused, expired, used, unknown, not_ready, busy, full: ApprovalFull }
```

- `expires`, `asked`, `earliest` and `until` are wall times in nanoseconds, as `SignedIn.expires` is.
- `ApprovalAsked.secret` is exactly 32 random bytes; `ApprovalSignIn.secret` takes up to 64, so a made-up secret takes the unknown path, as `LinkSignIn`'s does.
- `code` is the eight upper-case letters of 2.3, never the shown form.

### 2.2 The approval's code

A hand-written module of the wire crate, `jig/crates/jig-client-protocol-wire/src/approval.rs`, step code, exported from its `lib.rs`; both ends and temper's address codec use it:

```rust
/// The code's alphabet: twenty consonants, no vowel, none like another
/// or a digit (protocol/client.md, 8).
pub const CODE_ALPHABET: [u8; 20] = *b"BCDFGHJKLMNPQRSTVWXZ";
/// How many letters a code has.
pub const CODE_LETTERS: usize = 8;
/// The most bytes `ApprovalCode::parse` reads.
pub const CODE_TYPED_BYTES: usize = 32;

/// An approval's code: eight letters of the alphabet, upper case.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ApprovalCode([u8; 8]);

impl ApprovalCode {
    /// Typed or carried bytes: letters of the alphabet in either case,
    /// hyphens and spaces anywhere ignored, exactly eight letters left;
    /// none past `CODE_TYPED_BYTES` or for any other byte.
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self>;
    /// From eight indexes into the alphabet, as the engine draws them;
    /// none for an index past 19.
    #[must_use]
    pub fn from_indexes(indexes: [u8; 8]) -> Option<Self>;
    /// The eight letters, as the wire's `code` carries them.
    #[must_use]
    pub const fn letters(&self) -> &[u8; 8];
    /// As shown and in an address: four letters, a hyphen, four, `WDJB-MJHT`.
    #[must_use]
    pub fn shown(&self) -> [u8; 9];
}
```

The code is no secret: it is shown, and carried in the page's address. The secret is.

## 3. The engine (`jig-protocol-engine-clients`)

### 3.1 What crosses its boundary

- `FromTransport::Exchange { exchange: Token, credential: Option<Secret>, browser: bool, message: Message }`: `browser` is true only when the transport carried the credential itself, as a browser's cookie. The HTTP engine end sets it for a credential read from the cookie, never for a bearer, even beside the browser header. Every other constructor (test kits, joints) passes `false`, or `true` for a client it plays as a browser.
- `Message` gains `ApprovalAsk(Box<[u8]>)`, `ApprovalRead(Box<[u8]>)`, `ApprovalDecide(Box<[u8]>)` and `SignInApproval(Box<[u8]>)`, each the record of 2.1.
- `Settings` gains `approval: bool`, false wherever a literal builds one today.
- Constants of its new module `approval.rs`: `WAITING: u32 = 4` approvals live at once; `SPAN: Duration` of 10 minutes; `PAST: u32 = 8` ended approvals remembered by their secret's digest and their code, each as taken, refused or expired.

### 3.2 What it answers

| Message | Answer | When |
|---|---|---|
| `ApprovalAsk` | `refused` | approval not enabled |
| | `not_ready` | the engine stops |
| | `busy` | `WAITING` approvals live, once those past their span are moved to the past |
| | `asked { secret, code, expires }` | else: 32 fresh bytes, a code no live approval holds, `expires` the wall now plus `SPAN`. The component keeps the secret's digest, never the secret |
| `ApprovalRead`, `ApprovalDecide` | `not_a_browser` | `browser` false, before anything is read |
| | `signed_out` | no credential, or one whose digest or saved expiry fails, as any exchange's |
| | `not_ready`, `busy` | the engine stops; no room among the component's exchanges |
| | `unknown` | no live or remembered approval has the code, or approval not enabled |
| read | `waiting { asked }` | live, not decided |
| | `approved`, `taken`, `refused`, `expired` | as it stands or ended |
| decide | `approved` / `refused` | live and waiting: spent by this first decision; approved names the deciding sign-in's number as its approver |
| | `used`, `expired` | decided already (approved, taken or refused); past its span |
| `SignInApproval` | `unknown` | no live or remembered approval has the secret's digest, compared in constant time, or approval not enabled |
| | `waiting` | live and not yet decided |
| | `refused`, `expired`, `used` | refused; past its span; taken already |
| | `busy` | the same secret's take in flight; no room among the exchanges |
| | `not_ready` | the engine stops |
| | `signed_in { token, party, expires }` | approved: `Whose` of the approver, then the core's `SignIn` with that identity and no held sign-in; the credential drawn and its digest kept as any sign-in's, and handed out in this answer alone. The approval is then taken |
| | `full { sign_ins, earliest, until }` | the core answers full: the approval stays approved, `until` its own expiry |
| | `refused` | the approver's sign-in has ended (`Signer(None)`), or the core refuses: the approval is spent |

Not ready, busy and full spend nothing: the approval keeps its state and its expiry. An approval approved and never taken expires at its span with nothing made. `next_deadline` includes the earliest live approval's end, and `fire` moves it to the past as expired.

## 4. Configuration

- **jig:** `engine.clients.approval`, an optional boolean, false when absent, in `jig-protocol-config`'s `Clients { .., approval: bool }`; `config::clients::settings` copies it to `Settings.approval`. The reference service's startup refuses it on a listener off loopback as it refuses `link` (`What::Plaintext` at `engine.clients.approval`).
- **temper:** its templates (`temper init`'s `temper.json` and `bare.json`, so `temper box`'s too) and `temper/docs/examples/owned.json` set `"approval": true` beside `"link": true`; temper's service passes it on and refuses it off loopback as it does `link`.

## 5. The HTTP mapping

| Route | Message |
|---|---|
| `POST <prefix>/v1/approval/ask` | `ApprovalAsk` |
| `POST <prefix>/v1/approval/read` | `ApprovalRead` |
| `POST <prefix>/v1/approval/decide` | `ApprovalDecide` |
| `POST <prefix>/v1/sign-in/approval` | `SignInApproval` |

The guards hold for each as for every route (`browser.md`, 12.2). The engine end sets `browser` from the credential's source (3.1). A browser's `signed_in` on `sign-in/approval` is answered as a browser's link sign-in is: the cookie set, no token in the body. The client end maps `jig-client-protocol-link`'s messages of the same names to the same paths.

## 6. The client (`jig-client-protocol-link`, re-exported by `jig-client`)

### 6.1 Types

In `jig-client-protocol-link`'s boundary; `jig-client` re-exports each, and `ApprovalCode` from the wire:

```rust
pub enum With {
    // as today, and:
    /// An approval's secret, from `Approval::Asked`: the take (client.md, 12).
    Approval { secret: Secret },
}
pub enum Signing {
    // as today, and:
    /// Why an approval's take installed nothing.
    Approval(Taking),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taking { Waiting, Refused, Expired, Used, Unknown }

/// An approval's answer to its host (protocol/client.md, 8).
#[derive(Debug, PartialEq, Eq)]
pub enum Approval {
    /// Asked: the secret is the host's to take with, never shown.
    Asked { secret: Secret, code: ApprovalCode, expires: Wall },
    NotAsked(NotAsked),
    Read { code: ApprovalCode, state: ApprovalState },
    Decided { code: ApprovalCode, decided: Decided },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotAsked { Refused, NotReady, Busy, Lost }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalState { Waiting { asked: Wall }, Approved, Taken, Refused, Expired, Unknown, NotABrowser, SignedOut, NotReady, Busy, Lost }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decided { Approved, Refused, Used, Expired, Unknown, NotABrowser, SignedOut, NotReady, Busy, Lost }
```

### 6.2 `jig-client`'s surface

- `Event::AskApproval`, `Event::ReadApproval { code: ApprovalCode }`, `Event::DecideApproval { code: ApprovalCode, approve: bool }`: each one exchange, the ask with no credential, a read and a decision with the credential held (none under `Transport`).
- `Request::Approval(Approval)`: each one's single answer, a failure to answer `Lost`. A read or decision answered `signed_out` also signs the client out, as any exchange's signed out does.
- `SignIn { with: Approval { secret } }`: the take, sent with no credential. Answered `signed_in`, the credential is installed as a link's is, and `link().credential` gives it. Answered `waiting`, the client sends the same take again by itself after `APPROVAL_AGAIN`, one second, until another answer, a next `SignIn` or a `SignOut`; not ready, busy and lost take the existing backoff. Every other answer ends it: `NotSignedIn::Approval(Taking)` for refused, expired, used and unknown, `NotSignedIn::Full { sign_ins, earliest, link_until }` with the approval's `until` for full. `NotSignedIn::Approval(Taking::Waiting)` is told while it waits.
- The client keeps the secret only while it takes, in memory, never in the small store; nothing logs or prints it (`Secret`'s `Debug` gives its length).

## 7. temper

- **The page's address:** `/terminal`, the code in its query as shown, `/terminal?code=WDJB-MJHT`, by temper's address codec in `temper-client-domain` (`client/browser.md`, 2.3). A `code` that doesn't parse (2.2) is dropped and the page asks for the code to be typed. The codec prints the shown form. The TUI makes its tab's address from the engine's address and this printed path alone.
- **`temper-client-domain`'s half for a native client** (the TUI's, in the follow-up plan): the root's own event to ask, which sends `AskApproval` and, on `Asked`, signs in at once with `With::Approval { secret }`, keeping only the code and the expiry for its host to draw; the outcomes are the link's (`not_signed_in`, signed in). Its names are the spec's of the task that builds it.
- **The approval page** is the web client's, a place of its own beside `Authorize`, outside the frame, in the browser alone; its states and words are `client/README.md`'s and `client/browser.md`, 2.4's.
