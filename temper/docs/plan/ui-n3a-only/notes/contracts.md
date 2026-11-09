# ui-n3a-only: contracts

The shapes that cross this plan's tasks, binding (`notes/common.md`).
Design: `temper/docs/design/client/README.md`, 3.5, **Only you**, and
*A page's parts*.

## 1. The read: `temper_client_domain::pages::only_you`

A new module, `temper/crates/temper-client-domain/src/pages/only_you.rs`,
`pub mod only_you;` in `pages.rs` beside `pub mod needs;`.

```rust
//! Only you (temper/docs/design/client/README.md, 3.5, Only you).

/// Why a source needs its person.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    /// `AccountState::SignInAgain`.
    SignedOut,
    /// `AccountSource::expiring`, the source not signed out.
    Expiring,
}

/// An account's deciding source that the person alone can sign in again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    /// `AccountLine::number`.
    pub account: u32,
    /// `AccountSource::number`.
    pub source: u32,
    /// `AccountKind::SignIn` or `AccountKind::Program`, never another.
    pub kind: jig_client::contract::AccountKind,
    pub why: Why,
}

/// Only you's items, in the document's order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnlyYou {
    pub items: skein_lib::List<Item>,
    /// The `accounts` document was `cut`: the head says "N+".
    pub cut: bool,
}

/// The rule over one document, pure: `party` the person's (`me`'s
/// `party.number`), `playground` the playground's provider, whose account
/// is the deployment's and makes no item.
#[must_use]
pub fn items(document: &jig_client::contract::Accounts, party: u64, playground: Option<u32>) -> OnlyYou;

/// `items` over what the root holds: `root.pages().accounts_document(..)`,
/// `root.pages().me(..)`, `root.forge().playground_provider()`. With no
/// document or no `me`, no items and not cut.
#[must_use]
pub fn read(root: &crate::Domain) -> OnlyYou;
```

The rule (README 3.5, **Only you**, *Its read*): for each `AccountLine`
in `document.accounts`, in order, whose `holder` is
`AccountHolder::Party(party)` and whose `provider` is not `playground`
(a `None` provider is kept): its deciding source, `sources.get(0)`, as
`temper-client-view`'s `accounts::words::mark` reads it, makes an item
where its `kind` is `SignIn` or `Program`: `SignedOut` where `state` is
`AccountState::SignInAgain`, else `Expiring` where `expiring` is true,
else none. A later source makes none. `cut` is `document.cut`. The
list's capacity is the count of items, counted first, so it never
fails.

## 2. The page's parts: `pages::parts`

```rust
pub struct NeedsFacts {
    pub known: bool,
    pub open: u32,
    pub showing: u32,
    /// Only you's items, `only_you::read(root).items.len()`.
    pub only: u32,
}
pub struct NeedsParts {
    pub list: bool,
    pub done: bool,
    pub decided: bool,
    /// `facts.known && facts.only > 0`.
    pub only: bool,
}
```

`Default` gives `only: 0` and `only: false`. `parts::needs` stays a
`const fn`; `parts::needs_facts` fills `only`. No other fact or part
changes: `list`, `done` and `decided` keep their rules, and `open`
counts cards alone.

## 3. The view's region: `temper-client-view`'s `pages/only_you.rs`

```rust
//! Only you on Needs you (temper/docs/design/client/README.md, 3.5, Only you).
pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings);
```

`pages/needs.rs`'s `main` calls it once, as the line before `if
parts.decided {`: `if parts.only { super::only_you::main(b, root, w,
bindings); }`. It reads `pages::only_you::read(root)` and, for each item,
its `AccountLine` and `AccountSource` by number from
`root.pages().accounts_document(root.client())`, skipping an item whose
row or source is no longer there. Its act is `Act::Go { address:
Address::Accounts }`, bound as the run wait's `account_link`
(`temper-client-view/src/choice.rs:43`) binds it.

Its words, each a `Word` of `words.rs`:

| Word | Words |
|---|---|
| the head | "Only you · N", N in digits, "N+" where `cut` |
| the head's right | "work goes on around it" |
| the from line | the chip "Deployment" (`Word::RailDeployment`'s words), then "an LLM account" |
| signed out | "Your SOURCE is signed out. Sign in again?" |
| expiring | "Your SOURCE is expiring. Sign in again?" |
| the act | "Sign in again" |

SOURCE is the source as Accounts' row names it
(`pages/accounts/rows.rs:186`): a sign-in, `W::PlanSignIn` named with
its provider's plan ("ChatGPT sign-in"); a program's login,
`W::CodeLogin` named with its name ("Claude Code login claude-alice").

## 4. Accounts' expiring row

`pages/accounts/rows.rs`'s `notice`, for a source whose `expiring` is
true and whose state is not `SignInAgain` (a signed-out one keeps its
row as it is): a sign-in, the paragraph "Expiring: sign in again before
it ends." and `signin(..)`, its "Continue with PLAN"; a program's login,
the same paragraph, the command line
(`super::super::connect_model::command_line(root, w, NAME)`, NAME the
source's name) and "Check again" (`Operation::CheckProgram`). The tag
"Expiring" stays as `state` draws it. The new words are
`accounts/words.rs`'s.
