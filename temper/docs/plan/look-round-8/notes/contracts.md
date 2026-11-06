# look-round-8: contracts

The run's binding calls, for `temper/docs/design/client/README.md`, 3.5 (*The configuring chat's page*; *temper's own way, beside the page*).

## 1. The stock page

- **The file:** `temper/docs/help/how-temper-builds-temper.md`, byte for byte `temper/docs/plan/look-round-8/notes/how-temper-builds-temper.md`.
- **The kind:** `temper_docs::Kind::Way`, word `way`, bound 8,192 (`temper/docs/design/protocol/documentation.md`, 2.2). Its template: each `## ` heading is `N. …`, numbered from 1.
- **The row:**

  ```rust
  /// The way the client draws beside "Configure temper" and the configuring agent reads.
  pub const HOW_TEMPER_BUILDS_TEMPER: Page = Page {
      name: "how-temper-builds-temper",
      kind: Kind::Way,
      title: "How temper builds temper",
      summary: "How fabrig builds temper, with temper: one way among many. Start from it, from something smaller, or from your own words.",
      text: include_str!("../../../docs/help/how-temper-builds-temper.md"),
  };
  ```

  `PAGES` holds it after the recipes, before the reference pages, as `HOW_TEMPER_BUILDS_TEMPER,`; its length goes from 59 to 60.
- **The view takes the `const`, never `PAGES` or `find`,** so that the bundle holds this page alone.

## 2. What the page decides

Each departs from the request's words, `temper/docs/design/draft/own-way.status.md`:

- **Section 1** is the flow's words, a numbered list. No flow, and no `figure`.
- **Section 3's second item** says "whose tests run along the way", where the request says "checked by CI and a review on the way": the reference's goal branches have neither (`temper/docs/design/draft/reference-code-production.md`, sections 6 and 10).
- **The table:** Workers' cells, "Several machines, several slots each" and "The deployment’s configuration" (`temper/docs/design/protocol/configuration.md`, section 2); QA's "Yours to set", "Later" (`temper/docs/design/domain/findings.md`, section 8, is not built). Each other row is a setting: Plans, a project's ways and its goal template; Models, a charter's models; Branches, a template's own branch; Landing, a project's landing rules; Questions, a project's questions (`temper/docs/design/protocol/configuration.md`, 13.2 and 13.9).
- **The line under the table:** "Each row is a setting, unless it says otherwise. Tell the chat what to keep, and what to change."
- **Section 6:** Merges, Agent runs and Workers, each "—", then "Figures to come." No date, no link and no address. The user fills them later, in this file alone.

## 3. The domain

In `temper-client-domain`:

```rust
// pages.rs, in Act
/// "See how it works": the stock reference beside Configure (client/README.md, 3.5).
ShowReference,

// pages/chats.rs, in Chats
/// The reference's mode beside Configure, once opened; the page's alone (client/README.md, 3.5).
pub reference: Option<Mode>,
```

- `Act::ShowReference`, at `Address::Configure`, sets `chats.reference = Some(Mode::Split)`. Anywhere else it does nothing. It pushes no `Request`.
- `Act::Mode { mode }`, at `Address::Configure` with `chats.reference` set, sets it to `Some(mode)`, and also pushes nothing.
- A new `Chats` starts it at `None`, so leaving the page or a reload closes the reference.

## 4. The tree

In `temper-client-view`. `look` names are `temper-` and the name in lower case with hyphens.

**The card,** inside `STARTER_WAYS`, in this order:

```
Region "Ways to begin"            STARTER_WAYS
  Slot                            STARTER_HEAD
    Heading 3 "Ways to begin"     (browser only)
    Button "Not sure yet? …"      STARTER_UNSURE
  List                            STARTERS (its three items, as now)
  Group "temper’s own way"        STARTER_OWN
    Slot                          STARTER_OWN_TEXT
      Paragraph                   STARTER_OWN_TITLE  "At scale, as temper builds itself"
      List "Its steps: designers cut a draft into parts, coding agents make them in parallel, the goal lands whole"
                                  STARTER_OWN_STEPS  Draft, The cut, Coding agents, Goal’s branch, main
    Slot                          STARTER_OWN_ACTS
      Button "See how it works"   Act::ShowReference, never disabled
      Button "Start from it"      Act::Starter { "Set PROJECT up the way temper builds itself, sized for this machine." }, disabled while pending
```

PROJECT is the name the page's question uses, its fallback included. The words are `Word`s: `ConfigureOwn`, `ConfigureOwnTitle`, `ConfigureOwnSteps`, a word per step, `ConfigureOwnSee`, `ConfigureOwnStart`, `ConfigureOwnSends`.

**With `reference` set,** the main carries `document::style(mode)` after `SESSION`, and the bar's right part holds, after the byline, the group `document::modes` draws: a chat's three buttons, named `Word::ReferenceGroup`, "How temper builds temper, beside the chat". The composer's hint is the document's (`session::hint(b, w, true)`). With `reference` `None`, the main's styles stay `[MAIN, SESSION]`, and the bar and hint stay as they are.

**The pane,** in a new `pages/reference.rs`, after the conversation region, drawn unless the mode is `Mode::Chat`:

```
Region "How temper builds temper"   DOC
  Slot                              DOC_INNER
    Paragraph                       DOC_PATH  "temper’s own way · a reference", "changes nothing"
    Heading 2 (the page's title, level as a chat's document's)
    Paragraph                       DOC_SUB   the page's summary
    Slot                            DOC_SEC   one for each `## ` section
      Slot                          DOC_FLOW on section 1, none on the others
        the section, by skein_ui::markdown::read(.., 1, b)
```

- As a chat's document, the terminal gets `Word::ToolSeparator` between the path's two runs.
- **Section 5 and section 6 end in a closing note:** the section's last paragraph is a `Paragraph` with `DOC_MUTED`, the rest read by the reader.
- **Section 6's opening bullets** are `Slot DOC_FIGURES`, then for each `- TERM: VALUE` a `Slot` holding `Role::Term` and `Role::Description`. A section is known by its heading's number, from the page itself, and the pane panics on neither form: a section with no such bullets or note is read whole.
- `document.rs` gains `style(mode: Mode) -> Style` and `modes(b, name, mode, words, bindings)`, which `mode` and `seg` then use, so a chat's look doesn't change.

## 5. The overlay, rule by rule

`card` is `own-way-card`, `pane` is `own-way-pane`. The overlay comes after `temper.css`, so at equal specificity a rule of its beats a phone rule. Landing one in place, drop the phone rule it beat, or 420 differs.

| Overlay rule (`#app > .temper-session` omitted) | Lands in | How |
|---|---|---|
| `.temper-starter-head` | card | as it is |
| `.temper-starter-head > h3` | card | as it is; `.temper-starter-ways > h3` goes |
| `.temper-starter-head > .temper-starter-unsure` | card | as it is; `.temper-starter-unsure`'s own margin and font size go to its values |
| `.temper-thread` (28, 8) | card | **narrowed** to the opening, `.temper-thread:has(> .temper-starter > .temper-starter-ways)`, at wide widths alone: as drawn it reaches every chat, and the message names the opening. The phone's `padding: 20px 16px` stays |
| `.temper-starter-ways` (30, 12) | card | edit in place; the phone's `margin-top: 32px` goes |
| `.temper-starters` (gap 10) | card | edit in place; the phone's `gap` goes |
| `.temper-starters button` (padding) | card | edit in place; the phone's `padding` goes |
| `.temper-starter-own`, `-text`, `-text > p`, `-title`, `-steps`, `-steps > li:not(:last-child)::after`, `-acts`, `-acts > button` and its hover | card | as they are |
| the `@media (max-width: 760px)` pair | card | as they are |
| `.temper-mode-split .temper-starters`, its `button`, and `.temper-starter-own-acts` | pane | as they are |
| `figure:has(> svg…)`, `figure > svg…` | none | dropped: there is no figure |
| `.temper-doc-inner th[scope=row]` | pane | **as `.temper-doc-inner td > strong`,** its declarations: skein's reader draws no row header, so the row's name is `**bold**` in a cell |
| `.temper-doc-inner td` | pane | as it is |
| `.temper-doc-figures`, `> div` | pane | as they are |
| `.temper-doc-figures dt`, `dd` | pane | **as `[role=term]`, `[role=definition]`,** their declarations: `Role::Term` and `Role::Description` draw those |
| `.temper-doc-inner p > span > a` | none | dropped: there is no link |

And one the overlay doesn't have, for the picture, in `pane`: `.temper-doc-flow ul { padding-left: 0; list-style: none; }`, the reader's form today, whose words hold their number, and `.temper-doc-flow ol { padding-left: 20px; }`, the form `chat-reading-d/numbered-lists` makes. If that task has landed on your base, write only the `ol` rule and say so in the report.

## 6. The agent's words

In `temper/crates/temper-service/src/stock/configuring.txt`, one paragraph, before its last:

> Asked to set a project up the way temper builds itself, first read it: read_docs, topic how-temper-builds-temper. Propose its settings sized for this deployment: the slots its workers have, the models its accounts reach. Say what the deployment cannot do yet and leave it out. In "How PROJECT works", keep a column of temper's own values beside the project's, so each difference is a sentence the person can change.
