# look-ask-ways: contracts

The shapes that cross this plan's tasks, binding as written. The design
is `temper/docs/design/client/README.md`, 3.5 (*Ask temper's page*), 5.13
(*A way's acts*) and 5.16; the look is the request `ask-temper-ways`
(`notes/common.md`).

## 1. The opening's state (`temper-client-domain`, task `opening-state`)

On `temper_client_domain::forge::stock_ways::StockWays`, beside what is
there:

```rust
/// The index into `tabs()` of the tab shown: the current one as the page opens (client/README.md, 5.13).
pub fn shown(&self) -> Option<u32>;
/// The stop of the shown tab's path whose card is open.
pub fn open(&self) -> Option<u32>;
/// Whether a "Use this" or "Keep it" is out: the act shows, disabled.
pub const fn sending(&self) -> bool;
```

- `act(index)` keeps its rules but one: it no longer answers `None` while
  an act is out; `sending()` says that instead, and `send_act` still sends
  nothing then.
- `shown()` is `None` while the state is not `Shown`. On the first shown
  document it is the current tab's index (the way the panel's `current`
  names, else the "Yours" tab). On a later document the shown tab is kept
  by its way, `Tab::index` (`None` for "Yours"), at its new place in
  `tabs()`, while that tab still stands; else it is the current tab again.
- `open()` is `None` at first and after a tab is switched; it is kept
  across documents while the shown tab's path has the same stops (each
  stop's actor and words), else `None`.

In `temper_client_domain::pages::Act`:

```rust
/// Show a tab of Ask temper's ways in place (client/README.md, 3.5).
WayTab { project: u32, tab: u32 },
/// Open, move or close a stop's card under the shown path.
WayStop { project: u32, stop: u32 },
```

- `WayTab`: `tab` indexes `tabs()`; it shows that tab and closes the card.
  Past the tabs, or for another project, nothing.
- `WayStop`: `stop` indexes the shown tab's `path.stops()`; a closed stop
  opens, the open one closes, another moves the card there. A stop whose
  actor is `bar`, a stop past the path, or another project: nothing.
- Neither sends anything, and neither is in an address.

In `temper_client_domain::pages::needs::Toasted`:

```rust
/// A way's act answered: its name and the revision it made (client/README.md, 5.13).
Way { name: Box<[u8]>, revision: u32, kept: bool },
```

said with the toasts' own `say`, when the change sent by "Use this"
(`kept: false`) or "Keep it" (`kept: true`) is answered
`Done(Outcome::SettingsChanged { revision })`; `name` is the way's
`name` in the panel the act was sent from. The view's words for it
(section 4) are `opening-state`'s, written with the arm the compiler
asks for in `temper-client-view`.

## 2. A path's drawing (`temper-client-view::paths`, task `path-look`)

```rust
pub enum Mark { Person, Agent, Code, Bar }          // as today
pub struct Stop<'a> { pub mark: Mark, pub words: &'a [u8] } // as today

/// A path whose stops are pressed (client/README.md, 5.16, *Pressed*).
pub struct Pressed<'a> {
    /// The drawing node's binding: a press on a stop reports
    /// `skein_ui::Act::Drawing { data }`, `data` the stop's index as one byte.
    pub drawing: skein_ui::Binding,
    /// Each stop's fallback button, `None` for a bar; as long as the stops.
    pub stops: &'a [Option<skein_ui::Binding>],
    /// The stop whose card is open.
    pub open: Option<u32>,
}

pub fn path(
    builder: &mut Builder,
    label: &[u8],
    stops: &[Stop<'_>],
    pressed: Option<&Pressed<'_>>,
) -> Result<(), Problem>;

/// The stop a press names, from a drawing act's data.
pub fn pressed_stop(data: &[u8]) -> Option<u32>;
```

- **The node:** a drawing of `KIND` named `label`, carrying the token
  `WAYS_PATH` (`temper-ways-path`); bound to `pressed.drawing` when pressed.
  Its fallback is a list named `label`, a stop an item: words and actor in
  words as today ("produce, an agent"); pressed, each stop but a bar a
  button of those words, bound to its `stops` entry, `Expanded` while it
  is `open`.
- **Its data** grows by two bytes after the count: a flags byte (bit 0,
  pressed) and the open stop's index (255 for none). `DATA`, `MARKS` and
  `SCENE_BYTES` are recomputed and stay the module's bounds.
- **Its layout** is `client/README.md`, 5.16, *Its layout* and *Pressed*:
  across from 560 pixels wide, down the page under it; the last bar the
  landing branch's end, an earlier bar a goal's branch in a stop's place;
  a pressed stop a group (`WAYS_STOP`) holding its hit area (`WAYS_HIT`,
  carrying the act), its halo when open (`WAYS_HALO`), its mark and its
  words (`WAYS_OPEN` beside `WAYS_WORDS` when open); the caret
  (`WAYS_CARET`) under the open stop, across alone.
- **Tokens,** appended in this order after the last of `look.rs`, each
  name in `NAMES`: `WAYS_PATH` `temper-ways-path`, `WAYS_RISE`
  `temper-ways-rise`, `WAYS_STOP` `temper-ways-stop`, `WAYS_HIT`
  `temper-ways-hit`, `WAYS_HALO` `temper-ways-halo`, `WAYS_OPEN`
  `temper-ways-open`, `WAYS_LANDING` `temper-ways-landing`, `WAYS_END`
  `temper-ways-end`, `WAYS_CARET` `temper-ways-caret`. The bar's token's
  name becomes the request's, `temper-ways-main` (the constant
  `WAYS_MAIN`, in `WAYS_BAR`'s place and number).
- **Replies** (`views.rs`) call `path(..., None)`: the same look, nothing
  pressed.

## 3. The opening's tree (`temper-client-view`, task `opening`)

Inside the starter slot (`STARTER`, `temper-starter`) of `Ask { project
}`, in this order, and nothing else of today's opening:

| Node | Role, name | Tokens (classes) | Words, act |
|---|---|---|---|
| the lede | paragraph | `STARTER_LEDE`, `WAYS_LEDE` (`temper-starter-lede temper-ways-lede`) | "Pick a way for PROJECT, or ask temper to build your own." |
| the ways | region "Ways of working" | `STARTER_WAYS`, `WAYS` (`temper-starter-ways temper-ways`) | |
| the tabs | tabs "Ways of working" | `WAYS_TABS` | |
| a tab | tab, no name, `Selected` when shown | none | its text: the way's name, then on the current tab a text "now" (`WAYS_NOW`); `Act::WayTab { project, tab }` |
| the axis | paragraph "From more of you to more of your agents" | `WAYS_AXIS` | texts "more of you" and "more of your agents" |
| the panel | group named by the way's name | `WAYS_PANEL` | the shown tab's alone |
| its line | paragraph | `WAYS_SAYS` | the way's `line`; none for "Yours" |
| its path | `paths::path`, label "The way a change goes" | section 2 | `Pressed` with `drawing` and each stop's `Act::WayStop { project, stop }`, `open` the domain's |
| the card | region named by its heading, while a stop is open | `WAYS_ZOOM` | below |
| the foot | slot | `WAYS_FOOT` | |
| the legend | paragraph "An agent, a person, code; no merge tool above the dotted line" | `WAYS_LEGEND` | texts "agent", "person", "code", "no merge tool above", each with `WAYS_KIND_AGENT`, `WAYS_KIND_PERSON`, `WAYS_KIND_CODE`, `WAYS_KIND_LINE` |
| the acts | slot, only with an act | `WAYS_ACTS` | a button "Use this" or "Keep it" (`WAYS_USE`), `Disabled` while `sending()`: `Act::UseWay { project, way }` or `Act::KeepWay { project }` |
| the refusal | alert, while `refused()` | `WAYS_REFUSED` | jig's words with the forge's reason, as Settings says one under its row |

- **The card** is the open stop's `card` from the read: a slot
  (`WAYS_ZOOM_HEAD`) holding a level-3 heading, the stop's words with its
  first letter upper-cased ("Review"); a text, the card's `who`; and,
  where the stop opens a charter, a gate or a rule, a link "Open NAME ›",
  NAME the card's `name`, to `Settings { project }`. Then a group of the
  card's `facts`, each a slot of a term (its label) and a description (its
  value), as `views.rs`'s card draws them.
- **The terminal** prints the same tree: the tabs a line each, the current
  one's words "NAME, now"; the path's fallback a stop a line; the card,
  the legend and the act as text and controls.
- **Before and without the read:** while `StockWays::state()` is
  `Loading`, the ways' region holds no node; when it is `Ended`, a
  paragraph (`WAYS_SAYS`) "The ways can't be read now.".
- **The rail, top bar and composer** stay as today; the composer's hint is
  the chat's, `Word::ChatSendHint`.

## 4. Numbers

- **Words:** `opening-state` takes 4300 and 4301 (`ToastWayUsed`,
  `ToastWayKept`); `opening` takes 4302 up, in its words' order.
- **Tokens:** `path-look`'s are section 2's, in that order; `opening`'s are
  appended after them, in section 3's order: `WAYS_LEDE`, `WAYS`,
  `WAYS_TABS`, `WAYS_NOW`, `WAYS_AXIS`, `WAYS_PANEL`, `WAYS_SAYS`,
  `WAYS_ZOOM`, `WAYS_ZOOM_HEAD`, `WAYS_FOOT`, `WAYS_LEGEND`,
  `WAYS_KIND_AGENT`, `WAYS_KIND_PERSON`, `WAYS_KIND_CODE`,
  `WAYS_KIND_LINE`, `WAYS_ACTS`, `WAYS_USE`, `WAYS_REFUSED`, their names
  `temper-ways-lede`, `temper-ways`, `temper-ways-tabs`, `temper-ways-now`,
  `temper-ways-axis`, `temper-ways-panel`, `temper-ways-says`,
  `temper-ways-zoom`, `temper-ways-zoom-head`, `temper-ways-foot`,
  `temper-ways-legend`, `temper-ways-kind-agent`, `temper-ways-kind-person`,
  `temper-ways-kind-code`, `temper-ways-kind-line`, `temper-ways-acts`,
  `temper-ways-use`, `temper-ways-refused`.
