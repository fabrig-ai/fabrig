# ui-p5a-phone: contracts

The shapes that cross this plan's tasks. Binding: a task built to them
needs no other task's code but what its `needs` names.

## The phone's tabs (`temper-client-domain`)

`temper/crates/temper-client-domain/src/pages/rail.rs`, beside `Parts`
(`temper/docs/design/client/README.md`, 5.3, *On a phone*):

```rust
/// The phone's tab bar, one pure function over the rail's parts
/// (temper/docs/design/client/README.md, 5.3, On a phone). "Home" always
/// shows, first; the others in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "one switch per tab")]
pub struct Tabs {
    /// "Needs", while the rail's "Needs you" shows.
    pub needs: bool,
    /// Its count, drawn after "Needs" while more than zero, and "+" while cut.
    pub count: u32,
    pub cut: bool,
    /// "Work", while the rail's "Work" shows.
    pub work: bool,
    /// "Ask", while the rail's "Ask temper" shows.
    pub ask: bool,
    /// "Deployment", To begin's cut: while the rail's "Work" does not show.
    pub deployment: bool,
}

#[must_use]
pub fn tabs(parts: &Parts) -> Tabs;
```

Its rules, the design's table: `needs = parts.needs_you`, `count =
parts.count`, `cut = parts.cut`, `work = parts.work`, `ask = parts.ask`,
`deployment = !parts.work`. So outside To begin's cut: Home, Needs, Work and
Ask while asking; under it: Home, Ask while asking, Deployment.

## The tab bar and the Ask tab (`temper-client-view`)

- `frame::tabbar` draws `rail::tabs(&rail::parts(&rail::facts(..)))` and
  decides nothing else: "Home", "Needs" (`Word::NeedsTab`, "Needs";
  `Word::NeedsTabCount`, "Needs N"; `Word::NeedsTabCountCut`, "Needs N+",
  the count in `look::NAV_COUNT` as the rail's is), "Work", "Ask" (named
  "Ask temper", as today) and "Deployment", each current as the rail's
  entry is. The rail keeps "Needs you".
- While the tab bar's "Ask" is current (Ask temper's page, an Ask temper
  chat), the tab bar carries `look::TABBAR_ASK` (`temper-tabbar-ask`),
  which `temper.css` reads to keep it under a session's layout at 760 px and
  under, as it keeps `temper-tabbar-beginning`.

## Needs you's phone bar (`temper-client-view`)

- `pages/needs.rs`, while a card is open, draws first a slot
  `look::NEEDS_BAR` (`temper-needs-bar`): a link named "Needs you"
  (`Word::NeedsYou`), to Home where Done's "Home" goes; then the place,
  "1 of 2" (`Word::NeedsProgress` / `Word::NeedsProgressCut`, as the flow's
  head says it today), in a text styled `look::NEEDS_BAR_AT`
  (`temper-needs-bar-at`). The `.flow-head` slot, its dots and the
  `.next-up` line leave the view, with their styles and rules where nothing
  else reads them.
