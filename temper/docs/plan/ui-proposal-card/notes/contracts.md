# ui-proposal-card: the run's binding shapes

The design: `temper/docs/design/client/README.md`, 5.13, *The card in a chat*; `skein/docs/design/ui-web.md`, 7, *A focus request*.

## A focus request comes into view (`focus-reveal`)

In `skein/crates/skein-ui-web/src/page.rs`, a new variant beside `Op::Focus`:

```rust
Reveal { node: NodeId },
```

`Page::apply` pushes it straight after the `Op::Focus` of a focus request, and only there: a focus moved by a removal or by keys
pushes none. `page/web.rs` draws it as `scrollIntoView` with `block` and `inline` both `nearest`.

## "Change something" (`change-something`)

`temper_client_domain::pages::Act::FocusComposer` goes. Its one producer is the card, so its place is taken by:

```rust
Act::ChangeProposal { proposal: c::ProposalKey },
```

routed as `FocusComposer` was. On `Chat`, public:

```rust
/// The proposal "Change something" was pressed on, until something is sent or it is decided (client/README.md, 5.13).
pub change_asked: Option<c::ProposalKey>,
```

- `Act::ChangeProposal { proposal }` sets it and moves `composer_focus` by one.
- It goes back to `None` where a message's ask goes out (`Chat::send`, `Sending::Message`, past its early returns), and in
  `Chat::proposals` when `task.proposals` no longer holds its key: that list is what any decision, from any place, changes.
- The view reads it in `session_composer`: `Some` says `Word::SettingsProposalChangeAsks`, "What should change?"; `None` says
  `Word::MessagePlaceholder` as today.
