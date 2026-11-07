# account-email: the shapes that cross parts

Binding for every task of this plan. A shape here changes only by a question. The rules these shapes carry are `jig/docs/design/domain/accounts.md`, 3.7, and the sections each part below cites.

## 1. skein: the profile claims (`skein/docs/design/oauth.md`, 2.2 and 6.3)

```rust
// skein-oauth, re-exported from lib.rs
/// The profile claims a registration asks to read of an ID token.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Profile { pub email: bool, pub name: bool }

/// What an ID token said, of what was asked: words to show, no proof of anything.
#[derive(Clone, Default, PartialEq, Eq, Hash, Debug)]
pub struct ProfileClaims { pub email: Option<Box<[u8]>>, pub name: Option<Box<[u8]>> }

pub struct OpenId { pub issuer: Box<[u8]>, pub key_set: Option<Box<[u8]>>, pub profile: Profile }
// oauth::Request::Tokens gains `profile: ProfileClaims`.

// skein-oauth-accounts
Event::SignedIn { account: u32, generation: u64, missing: Box<[u8]>, subject: Option<Box<[u8]>>, profile: ProfileClaims }
/// A refresh whose record is kept and whose answer gave profile claims; before that generation's `Granted`.
Event::Refreshed { account: u32, generation: u64, profile: ProfileClaims }
```

## 2. smith: the status's email (`smith/docs/design/protocol/programs.md`, section 3)

```rust
// smith-protocol-program; `status()` and `Status` stay as they are
pub const EMAIL_BYTES: u32 = 255;
/// The email a usable status names, where it names one that reads.
pub fn email(code: u8, stdout: &[u8]) -> Option<Box<[u8]>>;

// smith-host-domain
ProgramCheck::Usable { email: Option<Box<[u8]>> }
```

The fake program's status names `you@example.com` while signed in, in `smith/tests/fake-claude`.

## 3. jig: the record (`jig/docs/design/domain/accounts.md`, 3.7; `jig/docs/design/protocol/store.md`, 4.4)

```rust
// jig-domain-accounts, src/facts.rs, re-exported from lib.rs. `Fact` is taken: it names a source's state.
pub const FACTS: u32 = 24;
pub const FACT_TEXT: u32 = 255;
pub const FACT_LABEL: u32 = 32;
pub const SHARE_WHOLE: u16 = 10_000;

pub const KIND_EMAIL: u16 = 1;     // KIND_NAME 2, KIND_PLAN 3, KIND_WORKSPACE 4, KIND_RENEWS 5, KIND_ENDS 6, KIND_USAGE 7
pub const ORIGIN_SIGN_IN: u16 = 1; // ORIGIN_REFRESH 2, ORIGIN_ANSWER 3, ORIGIN_LIMIT 4, ORIGIN_PROGRAM 5, ORIGIN_USAGE_READ 6
pub const WINDOW_OPEN: u16 = 1;    // WINDOW_WARNED 2, WINDOW_LIMITED 3

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Audience { Holder, Account }
/// A kind's audience; none for a kind this build does not register.
pub const fn audience(kind: u16) -> Option<Audience>;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AccountFact { pub kind: u16, pub label: Box<[u8]>, pub value: FactValue, pub seen: Wall, pub source: Option<u32>, pub origin: u16 }
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FactValue { Text(Box<[u8]>), Count(u64), Instant(Wall), Span(Duration), Share(u16), Window(FactWindow) }
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FactWindow { pub length: Option<Duration>, pub used: Option<u16>, pub resets: Option<Wall>, pub state: u16 }

impl AccountFact {
    /// Whether this build registers its kind, its origin and, for a window, its state.
    pub fn registered(&self) -> bool;
    /// Whether a reader may see it: registered, and of an account's kind or read by its holder.
    pub fn shown_to(&self, holder: bool) -> bool;
}

Stored::Facts { account: u32, facts: Box<[AccountFact]> }
Key::Facts { account: u32 }
impl Domain { pub fn account_facts(&self, account: u32) -> &[AccountFact]; }
```

The child keeps them in a map of its own, by account, never on `Account`, so that restore takes a facts row before its account's. An account with no facts has no record: its last fact gone erases it.

The store's schema, in `accounts-v2.schema`, in place:

```
record FactText { text: bytes 255 }
record FactCount { count: u64 }
record FactInstant { at: u64 }
record FactSpan { nanos: u64 }
record FactShare { basis: u16 }
record FactWindow { length: option u64 used: option u16 resets: option u64 state: u16 }
enum FactValue { text: FactText, count: FactCount, instant: FactInstant, span: FactSpan, share: FactShare, window: FactWindow }
record Fact { kind: u16 label: bytes 32 value: FactValue seen: u64 source: option u32 origin: u16 }
record Facts { account: u32 facts: list 24 Fact }
enum Value { next: Next, account: Account, source: Source, issued: Issued, facts: Facts }
```

Key: accounts' family, tag 5, then the account's number as a u64, as tag 1's. `RestoreRange::Accounts` walks a third interval, `Range::prefix(&[JIG, ACCOUNTS, 5])`, after its second. A share or a window's `used` past `SHARE_WHOLE` does not decode. A kind, origin or state no build registers decodes and is kept.

## 4. jig: a report (`jig/docs/design/domain/accounts.md`, 3.7; `jig/docs/design/protocol/llm.md`, 5.2)

```rust
// jig-domain-accounts, boundary.rs
/// What a provider's sign-in or refresh said; never a subject.
#[derive(Clone, Default, PartialEq, Eq, Hash, Debug)]
pub struct Said { pub email: Option<Box<[u8]>>, pub name: Option<Box<[u8]>> }

Event::SignedIn { source: u32, subject: Box<[u8]>, display: Box<[u8]>, declined: bool, said: Said }
FromComponent::Refreshed { account: u32, said: Said }   // `account` names the source, as FromComponent's others do
Event::ProgramSaid { name: Box<[u8]>, email: Box<[u8]> }
```

## 5. jig: a host's login (`jig/docs/design/protocol/channel.md`, 5.1)

```rust
// jig-domain-link; `LoginState` stays as it is
pub struct Login { pub name: .., pub state: LoginState, pub email: Option<Box<[u8]>> }
// jig-domain-host's sources.rs
Answer::Checked { state: LoginState, email: Option<Box<[u8]>> }
```

In `worker-wire-v9.schema`, in place: `record Login { name: text 32 state: LoginState email: option text 255 }`.

```rust
// jig-domain-fleet, once it has taken a host's whole logins: one per usable row naming an email
Request::ProgramEmail { name: Box<[u8]>, email: Box<[u8]> }
// jig-domain-core's route turns it into accounts::Event::ProgramSaid
```

## 6. jig: the client's line (`jig/docs/design/client/domain/contract.md`, 5.1)

```rust
// jig-domain-core, documents/accounts.rs and documents/deployment.rs
AccountLine { .., facts: List<accounts::AccountFact> }
Account     { .., facts: List<accounts::AccountFact> }   // the deployment's row

// jig-client-domain, contract/accounts.rs: the client's own vocabulary, as for its other kinds
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountFact { pub kind: u16, pub label: Box<[u8]>, pub value: AccountFactValue, pub seen: Wall, pub source: Option<u32>, pub origin: u16 }
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AccountFactValue { Text(Box<[u8]>), Count(u64), Instant(Wall), Span(Duration), Share(u16), Window(AccountWindow) }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AccountWindow { pub length: Option<Duration>, pub used: Option<u16>, pub resets: Option<Wall>, pub state: u16 }
pub const FACT_EMAIL: u16 = 1;
pub const FACT_NAME: u16 = 2;
impl AccountLine { pub fn email(&self) -> Option<&[u8]>; }
impl Account     { pub fn email(&self) -> Option<&[u8]>; }
```

In `client-v19-part1.schema`, in place, `AccountLine` and `Account` each gain `facts: list 24 AccountFact`, with `AccountFact` and its value shaped as section 3's schema, named `AccountFact`, `AccountFactValue`, `AccountWindow`, and their one-field records `AccountFactText`, `AccountFactCount`, `AccountFactInstant`, `AccountFactSpan`, `AccountFactShare`. A span on this wire is `duration`, as the schema's others are.

## 7. temper: who an account is (`temper/docs/design/client/README.md`, 5.5, *An account*)

```rust
// temper-client-view, accounts/words.rs
/// Who an account is: the email its provider gave, else what it was named.
pub(crate) fn who<'a>(email: Option<&'a [u8]>, named: &'a [u8]) -> &'a [u8];
```

For a signed-in source `named` is the display; for a program's, the source's name, else the display.
